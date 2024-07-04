mod apn;
mod dns;
mod tcp;

use atat::{asynch::AtatClient, AtatCmd};
use core::{str::from_utf8, sync::atomic::Ordering};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embedded_io::ErrorKind;
use embedded_nal_async::Ipv4Addr;

use crate::{
    commands::{
        gsm::SetMobileEquipmentError,
        tcpip::{
            BringUpWireless, ClientState, CloseConnection, ConfigureDomainNameServer,
            DeactivateGprsPdpContext, GetConnectionStatus, GetLocalIP, MultiIpValue,
            SelectDataTransmittingMode, SetManualRxGetMode, StartMultiIpConnection,
            StartTaskAndSetApn,
        },
    },
    device::{Handle, SOCKET_STATE_DROPPED, SOCKET_STATE_UNUSED, SOCKET_STATE_USED},
    DriverError, SimcomConfig, SimcomDevice, SimcomUrcChannel,
};

pub use apn::Apn;

use super::network::NetworkError;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SocketError {
    Atat(atat::Error),
    NoAvailableSockets,
    UnsupportedIpVersion,
    DnsError,
    DnsTimeout,
    UnableToConnect,
    ConnectTimeout,
    Closed,
    UnableToRead,
    ReadTimeout,
    WriteTimeout,
}

impl embedded_io::Error for SocketError {
    fn kind(&self) -> ErrorKind {
        match &self {
            SocketError::UnsupportedIpVersion => ErrorKind::Unsupported,
            SocketError::DnsTimeout => ErrorKind::TimedOut,
            SocketError::UnableToConnect => ErrorKind::ConnectionRefused,
            SocketError::ConnectTimeout => ErrorKind::TimedOut,
            SocketError::Closed => ErrorKind::ConnectionAborted,
            _ => ErrorKind::Other,
        }
    }
}

impl From<atat::Error> for SocketError {
    fn from(value: atat::Error) -> Self {
        SocketError::Atat(value)
    }
}

pub struct DataService<'buf, 'dev, 'sub, AtCl: AtatClient> {
    handle: &'dev Handle<'sub, AtCl>,
    urc_channel: &'buf SimcomUrcChannel,
    dns_lock: Mutex<NoopRawMutex, ()>,
    pub local_ip: Option<Ipv4Addr>,
}

impl<'buf, 'dev, 'sub, AtCl: AtatClient + 'static, Config: SimcomConfig>
    SimcomDevice<'buf, 'sub, AtCl, Config>
{
    pub async fn data(
        &'dev self,
        apn: Apn<'_>,
    ) -> Result<DataService<'buf, 'dev, 'sub, AtCl>, DriverError> {
        if self
            .data_service_taken
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let mut service = DataService::new(&self.handle, self.urc_channel);
            match service.setup(apn).await {
                Ok(_) => Ok(service),
                Err(e) => {
                    self.data_service_taken.store(false, Ordering::Relaxed);
                    Err(DriverError::Network(e))
                }
            }
        } else {
            Err(DriverError::AlreadyTaken)
        }
    }
}

impl<'buf, 'dev, 'sub, AtCl: AtatClient + 'static> DataService<'buf, 'dev, 'sub, AtCl> {
    fn new(handle: &'dev Handle<'sub, AtCl>, urc_channel: &'buf SimcomUrcChannel) -> Self {
        Self {
            handle,
            urc_channel,
            dns_lock: Mutex::new(()),
            local_ip: None,
        }
    }

    async fn setup(&mut self, apn: Apn<'_>) -> Result<(), NetworkError> {
        // According to the sim800 tcpip application note one should use the command group:
        // AT+CSTT, AT+CIICR and AT+CIFSR to start the task and activate the wireless connection.
        // See §2.1.1 in https://www.waveshare.com/w/upload/6/65/SIM800_Series_TCPIP_Application_Note_V1.02.pdf

        // AT+CIPSHUT
        self.send(&DeactivateGprsPdpContext).await?;

        // AT+CIPRXGET
        self.send(&SetManualRxGetMode).await?;

        // AT+CIPMUX
        self.send(&StartMultiIpConnection {
            n: MultiIpValue::MultiIpConnection,
        })
        .await?;

        // AT+CSTT
        // This implicitly activates the pdp context
        // so we should not manually call AT+CGACT
        // See git rev 3aa2787 for a version that connected using both.
        // It worked with tdc and telia, but not with onomondo.
        self.send(&StartTaskAndSetApn {
            apn: apn.apn,
            username: apn.username,
            password: apn.password,
        })
        .await?;

        // AT+CIICR
        self.send(&BringUpWireless).await?;

        // AT+CMEE
        self.send(&SetMobileEquipmentError {
            value: crate::commands::gsm::MobileEquipmentError::EnableVerbose,
        })
        .await?;

        // AT+CIFSR
        let ip = self.send(&GetLocalIP).await?.ip;
        self.local_ip = Some(from_utf8(ip.as_slice()).unwrap().parse().unwrap());

        // AT+CIPSTATUS
        for (id, state) in self.handle.socket_state.iter().enumerate() {
            let response = self.send(&GetConnectionStatus { id }).await?;
            let new_state = match response.state {
                ClientState::Initial => SOCKET_STATE_UNUSED,
                ClientState::Closed => SOCKET_STATE_UNUSED,
                ClientState::Connecting => SOCKET_STATE_USED,
                ClientState::Connected => SOCKET_STATE_USED,
                ClientState::Closing => SOCKET_STATE_USED,
                ClientState::RemoteClosing => SOCKET_STATE_USED,
            };
            state.store(new_state, Ordering::Release);
        }

        // AT+CIPQSEND
        // Enter quick send mode so that we get an URC when written data is buffered
        // instead of when it is received by the server
        // This changes the default "SEND OK" response into "DATA ACCEPT"
        self.send(&SelectDataTransmittingMode {
            mode: crate::commands::tcpip::DataTransmittingMode::QuickSendMode,
        })
        .await?;

        // AT+CDNSCFG
        self.send(&ConfigureDomainNameServer {
            pri_dns: "1.1.1.1",
            sec_dns: Some("1.0.0.1"),
        })
        .await?;

        Ok(())
    }

    async fn send<CMD: AtatCmd>(&mut self, cmd: &CMD) -> Result<CMD::Response, atat::Error> {
        let mut client = self.handle.client.lock().await;

        client.send(cmd).await
    }

    async fn close_dropped_sockets(&self) {
        for (id, state) in self.handle.socket_state.iter().enumerate() {
            if state.load(Ordering::Relaxed) == SOCKET_STATE_DROPPED {
                let mut client = self.handle.client.lock().await;

                // The close connection command does not return anything.
                // The actual transition from USED to UNUSED happens in URC handling,
                // as a "<id>, CLOSE OK" URC is sent when the connection is closed.
                match client.send(&CloseConnection { id }).await {
                    Ok(_) => {}
                    Err(atat::Error::CmeError(e)) if e == 3.into() || e == 100.into() => {
                        // CME Error seems to be returned if the connection is already closed
                        // Verify that it is actually the case
                        if let Ok(status) = client.send(&GetConnectionStatus { id }).await {
                            if status.state == ClientState::Closed {
                                warn!("[{}] Socket already closed", id);
                                state.store(SOCKET_STATE_UNUSED, Ordering::Release);
                            }
                        }
                    }
                    Err(e) => {
                        // If the close is not sent, we will simply retry later when `close_dropped_sockets()` is called again.
                        error!("[{}] Close request failed with error {}", id, e);
                    }
                }
            }
        }
    }
}

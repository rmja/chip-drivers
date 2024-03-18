mod apn;
mod dns;
mod tcp;

use atat::{asynch::AtatClient, AtatCmd};
use core::{str::from_utf8, sync::atomic::Ordering};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{with_timeout, Duration, Instant};
use embedded_io::ErrorKind;
use embedded_nal_async::Ipv4Addr;

use crate::{
    commands::{
        gprs::{self, ActivateOrDeactivatePDPContext},
        tcpip::{
            BringUpWireless, ClientState, CloseConnection, ConfigureDomainNameServer,
            DeactivateGprsPdpContext, GetConnectionStatus, GetLocalIP, MultiIpValue,
            SetManualRxGetMode, StartMultiIpConnection, StartTaskAndSetApn,
        },
        urc::Urc,
    },
    device::{Handle, SOCKET_STATE_DROPPED, SOCKET_STATE_UNUSED, SOCKET_STATE_USED},
    ContextId, DriverError, SimcomConfig, SimcomDevice, SimcomUrcChannel,
};

pub use apn::Apn;

use super::network::NetworkError;

const CONTEXT_ID: ContextId = ContextId(1);

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
        let state = self.get_pdp_context_state().await?;
        if state != gprs::PdpState::Deactivated {
            self.ensure_deactivated_pdp_context().await?;
        }

        self.send(&gprs::SetPDPContextDefinition {
            cid: CONTEXT_ID,
            pdp_type: "IP",
            apn: apn.apn,
        })
        .await?;

        self.send(&DeactivateGprsPdpContext).await?;

        self.send(&SetManualRxGetMode).await?;

        self.send(&StartMultiIpConnection {
            n: MultiIpValue::MultiIpConnection,
        })
        .await?;

        self.send(&StartTaskAndSetApn {
            apn: apn.apn,
            username: apn.username,
            password: apn.password,
        })
        .await?;

        self.send(&BringUpWireless).await?;

        let state = self.get_pdp_context_state().await?;
        if state == gprs::PdpState::Deactivated {
            self.send(&ActivateOrDeactivatePDPContext {
                cid: CONTEXT_ID,
                state: gprs::PdpState::Activated,
            })
            .await?;
        }

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

        let ip = self.send(&GetLocalIP).await?.ip;
        self.local_ip = Some(from_utf8(ip.as_slice()).unwrap().parse().unwrap());

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

    async fn get_pdp_context_state(&mut self) -> Result<gprs::PdpState, NetworkError> {
        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let subscription = self.urc_channel.subscribe().unwrap();

            client.send(&gprs::GetPDPContextStates).await?;

            subscription
        };

        let timeout_instant = Instant::now() + Duration::from_secs(20);
        while let Some(remaining) = timeout_instant.checked_duration_since(Instant::now()) {
            let urc = with_timeout(remaining, urc_subscription.next_message_pure())
                .await
                .map_err(|_| NetworkError::PdpStateTimeout)?;
            self.handle.drain_background_urcs();

            if let Urc::PdbState(state) = urc {
                if state.cid == CONTEXT_ID {
                    return Ok(state.state);
                }
            }
        }

        Err(NetworkError::PdpStateTimeout)
    }
    async fn ensure_deactivated_pdp_context(&mut self) -> Result<(), NetworkError> {
        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let subscription = self.urc_channel.subscribe().unwrap();

            client.send(&gprs::GetPDPContextStates).await?;

            subscription
        };

        let timeout_instant = Instant::now() + Duration::from_secs(20);
        while let Some(remaining) = timeout_instant.checked_duration_since(Instant::now()) {
            let urc = with_timeout(remaining, urc_subscription.next_message_pure())
                .await
                .map_err(|_| NetworkError::PdpStateTimeout)?;
            self.handle.drain_background_urcs();

            if let Urc::PdbState(state) = urc {
                if state.cid == CONTEXT_ID {
                    if state.state == gprs::PdpState::Deactivated {
                        return Ok(());
                    } else {
                        warn!("PDP context was found to be activated, deactivating...");

                        let mut client = self.handle.client.lock().await;
                        client
                            .send(&gprs::ActivateOrDeactivatePDPContext {
                                cid: CONTEXT_ID,
                                state: gprs::PdpState::Deactivated,
                            })
                            .await?;
                    }
                }
            }
        }

        Err(NetworkError::PdpStateTimeout)
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
                    Err(atat::Error::CmeError(e)) if e == 3.into() => {
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

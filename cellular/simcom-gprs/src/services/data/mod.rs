mod apn;
mod dns;
mod tcp;

use atat::{asynch::AtatClient, AtatUrcChannel};
use core::{str::from_utf8, sync::atomic::Ordering};
use embedded_io::ErrorKind;
use embedded_nal_async::Ipv4Addr;
use futures_intrusive::sync::LocalMutex;

use crate::{
    commands::{
        gprs,
        tcpip::{
            BringUpWireless, ClientState, CloseConnection, DeactivateGprsPdpContext,
            GetConnectionStatus, GetLocalIP, MultiIpValue, SetManualRxGetMode,
            StartMultiIpConnection, StartTaskAndSetApn,
        },
        urc::Urc,
    },
    device::{Handle, SOCKET_STATE_DROPPED, SOCKET_STATE_UNUSED, SOCKET_STATE_USED},
    ContextId, Device, DriverError,
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
    DnsTimeout,
    UnableToConnect,
    ConnectTimeout,
    MustReadBeforeWrite,
    Closed,
    ReadTimeout,
    WriteTimeout,
}

impl embedded_io::Error for SocketError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl From<atat::Error> for SocketError {
    fn from(value: atat::Error) -> Self {
        SocketError::Atat(value)
    }
}

pub struct DataService<'a, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> {
    handle: &'a Handle<AtCl>,
    urc_channel: &'a AtUrcCh,
    dns_lock: LocalMutex<()>,
    pub local_ip: Option<Ipv4Addr>,
}

impl<'a, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Device<AtCl, AtUrcCh> {
    pub async fn data(
        &'a self,
        apn: Apn<'_>,
    ) -> Result<DataService<'a, AtCl, AtUrcCh>, DriverError> {
        if self
            .data_service_taken
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let mut service = DataService::new(&self.handle, &self.urc_channel);

            service.setup(apn).await?;

            Ok(service)
        } else {
            Err(DriverError::AlreadyTaken)
        }
    }
}

impl<'a, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> DataService<'a, AtCl, AtUrcCh> {
    fn new(handle: &'a Handle<AtCl>, urc_channel: &'a AtUrcCh) -> Self {
        Self {
            handle,
            urc_channel,
            dns_lock: LocalMutex::new((), true),
            local_ip: None,
        }
    }

    async fn setup(&mut self, apn: Apn<'_>) -> Result<(), NetworkError> {
        let mut client = self.handle.client.lock().await;

        client
            .send(&gprs::SetPDPContextDefinition {
                cid: CONTEXT_ID,
                pdp_type: "IP",
                apn: apn.apn,
            })
            .await?;

        client.send(&DeactivateGprsPdpContext).await?;

        client.send(&SetManualRxGetMode).await?;

        client
            .send(&StartMultiIpConnection {
                n: MultiIpValue::MultiIpConnection,
            })
            .await?;

        client
            .send(&StartTaskAndSetApn {
                apn: apn.apn,
                username: apn.username,
                password: apn.password,
            })
            .await?;

        client.send(&BringUpWireless).await?;

        let ip = client.send(&GetLocalIP).await?.ip;
        self.local_ip = Some(from_utf8(ip.as_slice()).unwrap().parse().unwrap());

        for (id, state) in self.handle.socket_state.iter().enumerate() {
            let response = client.send(&GetConnectionStatus { id }).await?;
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

        Ok(())
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

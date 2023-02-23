mod apn;
mod dns;
mod tcp;

use core::{
    str::from_utf8,
    sync::atomic::Ordering,
};
use atat::asynch::AtatClient;
use embedded_io::ErrorKind;
use embedded_nal_async::Ipv4Addr;

use crate::{
    commands::{
        gprs,
        tcpip::{
            BringUpWireless, ClientState, CloseConnection, DeactivateGprsPdpContext,
            GetConnectionStatus, GetLocalIP, MultiIpValue, SetManualRxGetMode,
            StartMultiIpConnection, StartTaskAndSetApn,
        },
    },
    device::{Handle, SOCKET_STATE_UNUSED, SOCKET_STATE_USED, SOCKET_STATE_DROPPED},
    ContextId, Device, DriverError,
};

pub use apn::ApnInfo;

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

pub struct DataService<'a, AtCl: AtatClient> {
    handle: &'a Handle<AtCl>,
    pub local_ip: Option<Ipv4Addr>,
}

impl<'a, AtCl: AtatClient> Device<AtCl> {
    pub async fn data(
        &'a self,
        apn: &ApnInfo<'_>,
    ) -> Result<DataService<'a, AtCl>, DriverError> {
        if self
            .data_service_taken
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let mut service = DataService::new(&self.handle);

            service.setup(apn).await?;

            Ok(service)
        } else {
            Err(DriverError::AlreadyTaken)
        }
    }
}

impl<'a, AtCl: AtatClient> DataService<'a, AtCl> {
    fn new(handle: &'a Handle<AtCl>) -> Self {
        Self {
            handle,
            local_ip: None,
        }
    }

    async fn setup(&mut self, apn: &ApnInfo<'_>) -> Result<(), NetworkError> {
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
                if let Err(e) = client.send(&CloseConnection { id }).await {
                    // If the close is not sent, we will simply retry later when `close_dropped_sockets()` is called again.
                    error!("[{}] Close request failed with error {}", id, e);
                }
            }
        }
    }
}

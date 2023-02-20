mod apn;
mod dns;
mod tcp;

use core::{
    str::from_utf8,
    sync::atomic::{AtomicU8, Ordering},
};
use embedded_hal_async::delay::DelayUs;
use embedded_io::ErrorKind;
use embedded_nal_async::Ipv4Addr;

use crate::{
    atat_async::AtatClient,
    commands::{
        gprs,
        tcpip::{
            BringUpWireless, ClientState, CloseConnection, DeactivateGprsPdpContext,
            GetConnectionStatus, GetLocalIP, MultiIpValue, SetManualRxGetMode,
            StartMultiIpConnection, StartTaskAndSetApn,
        },
    },
    device::Handle,
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

pub(crate) type SocketState = AtomicU8;
pub(crate) const SOCKET_STATE_UNKNOWN: u8 = 0;
pub(crate) const SOCKET_STATE_UNUSED: u8 = 1;
pub(crate) const SOCKET_STATE_USED: u8 = 2;
pub(crate) const SOCKET_STATE_DROPPED: u8 = 3;

pub struct DataService<'a, AtCl: AtatClient, Delay: DelayUs> {
    handle: &'a Handle<AtCl>,
    delay: Delay,
    pub local_ip: Option<Ipv4Addr>,
}

impl<'a, AtCl: AtatClient, Delay: DelayUs + Clone> Device<AtCl, Delay> {
    pub async fn data(
        &'a self,
        apn: &ApnInfo<'_>,
    ) -> Result<DataService<'a, AtCl, Delay>, DriverError> {
        if self
            .data_service_taken
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let mut service = DataService::new(&self.handle, self.delay.clone());

            service.setup(apn).await?;

            Ok(service)
        } else {
            Err(DriverError::AlreadyTaken)
        }
    }
}

impl<'a, AtCl: AtatClient, Delay: DelayUs> DataService<'a, AtCl, Delay> {
    fn new(handle: &'a Handle<AtCl>, delay: Delay) -> Self {
        Self {
            handle,
            delay,
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

    fn take_socket_id(&self) -> Result<usize, SocketError> {
        for (id, state) in self.handle.socket_state.iter().enumerate() {
            if state
                .compare_exchange(
                    SOCKET_STATE_UNUSED,
                    SOCKET_STATE_USED,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return Ok(id);
            }
        }

        Err(SocketError::NoAvailableSockets)
    }

    async fn close_dropped_sockets(&self) {
        for (id, state) in self.handle.socket_state.iter().enumerate() {
            if state.load(Ordering::Relaxed) == SOCKET_STATE_DROPPED {
                let mut client = self.handle.client.lock().await;
                if client.send(&CloseConnection { id }).await.is_ok() {
                    state.store(SOCKET_STATE_UNUSED, Ordering::Release);
                }
            }
        }
    }
}

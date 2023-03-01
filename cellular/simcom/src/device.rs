use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use atat::asynch::AtatClient;
use embedded_io::asynch::Write;
use futures_intrusive::sync::LocalMutex;
use heapless::Vec;

use crate::{
    commands::{gsm, urc::Urc, v25ter, AT},
    services::{data::SocketError, network::Network},
    DriverError, PartNumber, SimcomDigester, MAX_SOCKETS,
};

pub(crate) type SocketState = AtomicU8;
pub(crate) const SOCKET_STATE_UNKNOWN: u8 = 0;
pub(crate) const SOCKET_STATE_UNUSED: u8 = 1;
pub(crate) const SOCKET_STATE_USED: u8 = 2;
pub(crate) const SOCKET_STATE_DROPPED: u8 = 3;

pub(crate) type ConnectedState = AtomicU8;
pub(crate) const CONNECTED_STATE_UNKNOWN: u8 = 0;
pub(crate) const CONNECTED_STATE_CONNECTED: u8 = 1;
pub(crate) const CONNECTED_STATE_FAILED: u8 = 2;

pub struct Handle<AtCl: AtatClient> {
    pub(crate) client: LocalMutex<AtCl>,
    pub(crate) socket_state: Vec<SocketState, MAX_SOCKETS>,
    pub(crate) connected_state: [ConnectedState; MAX_SOCKETS],
    pub(crate) is_flushed: [AtomicBool; MAX_SOCKETS],
    pub(crate) data_available: [AtomicBool; MAX_SOCKETS],
}

impl<AtCl: AtatClient> Handle<AtCl> {
    pub(crate) fn take_unused(&self) -> Result<usize, SocketError> {
        for id in 0..self.socket_state.len() {
            if self.try_take(id) {
                return Ok(id);
            }
        }
        Err(SocketError::NoAvailableSockets)
    }

    fn try_take(&self, id: usize) -> bool {
        if self.socket_state[id]
            .compare_exchange(
                SOCKET_STATE_UNUSED,
                SOCKET_STATE_USED,
                Ordering::AcqRel,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            self.connected_state[id].store(CONNECTED_STATE_UNKNOWN, Ordering::Relaxed);
            self.is_flushed[id].store(true, Ordering::Relaxed);
            self.data_available[id].store(false, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub(crate) fn handle_urc(&self, urc: &Urc) -> bool {
        match urc {
            Urc::ConnectOk(id) => {
                self.connected_state[*id].store(CONNECTED_STATE_CONNECTED, Ordering::Release);
                true
            }
            Urc::ConnectFail(id) => {
                self.connected_state[*id].store(CONNECTED_STATE_FAILED, Ordering::Release);
                true
            }
            Urc::SendOk(id) => {
                self.is_flushed[*id].store(true, Ordering::Release);
                true
            }
            Urc::Closed(id) => {
                warn!("[{}] Socket closed", *id);
                self.socket_state[*id].store(SOCKET_STATE_UNUSED, Ordering::Release);
                true
            }
            Urc::DataAvailable(id) => {
                debug!("[{}] Data available", *id);
                self.data_available[*id].store(true, Ordering::Release);
                true
            }
            urc => {
                error!("Uhandled URC: {:?}", urc);
                false
            }
        }
    }
}

pub struct Device<AtCl: AtatClient> {
    pub handle: Handle<AtCl>,
    pub(crate) part_number: Option<PartNumber>,
    pub network: Network,
    pub(crate) data_service_taken: AtomicBool,
}

impl<'a, Tx: Write, const RES_CAPACITY: usize, const URC_CAPACITY: usize>
    Device<atat::asynch::Client<'a, Tx, RES_CAPACITY, URC_CAPACITY>>
{
    /// Create a new device with a default AT client
    pub fn new<const INGRESS_BUF_SIZE: usize>(
        tx: Tx,
        buffers: &'a mut atat::Buffers<INGRESS_BUF_SIZE, RES_CAPACITY, URC_CAPACITY>,
    ) -> (
        atat::Ingress<'a, SimcomDigester, INGRESS_BUF_SIZE, RES_CAPACITY, URC_CAPACITY>,
        Self,
    ) {
        let (ingress, at_client) =
            buffers.split(tx, SimcomDigester::new(), atat::Config::default());

        let device = Self::with_at_client(at_client);
        (ingress, device)
    }
}

impl<AtCl: AtatClient> Device<AtCl> {
    /// Create a new device given an AT client
    pub fn with_at_client(at_client: AtCl) -> Self {
        let network = Network::new();
        // The actual state values, except for socket_state, are cleared
        // when a socket goes from [`SOCKET_STATE_UNUSED`] to [`SOCKET_STATE_USED`].
        Self {
            handle: Handle {
                client: LocalMutex::new(at_client, true),
                socket_state: Vec::new(),
                connected_state: Default::default(),
                is_flushed: Default::default(),
                data_available: Default::default(),
            },
            part_number: None,
            network,
            data_service_taken: AtomicBool::new(false),
        }
    }

    /// Setup the fundamentals for communicating with the modem
    pub async fn setup(&mut self) -> Result<(), DriverError> {
        self.is_alive(20).await?;

        let mut client = self.handle.client.lock().await;
        client.send(&v25ter::SetFactoryDefinedConfiguration).await?;

        client.send(&v25ter::Reset).await?;

        client
            .send(&v25ter::SetCommandEchoMode {
                mode: v25ter::CommandEchoMode::Disable,
            })
            .await?;

        client
            .send(&gsm::SetMobileEquipmentError {
                value: gsm::MobileEquipmentError::EnableNumeric,
            })
            .await?;

        client
            .send(&v25ter::SetFlowControl {
                from_modem: v25ter::FlowControl::Disabled,
                to_modem: Some(v25ter::FlowControl::Disabled),
            })
            .await?;

        let response = client.send(&gsm::GetManufacturerId).await?;
        if response.manufacturer != b"SIMCOM_Ltd" {
            return Err(DriverError::UnsupportedManufacturer);
        }

        let response = client.send(&gsm::GetModelId).await?;

        self.part_number = Some(match response.model.as_slice() {
            #[cfg(feature = "sim800")]
            b"SIMCOM_SIM800" => Ok(PartNumber::Sim800),
            #[cfg(feature = "sim900")]
            b"SIMCOM_SIM900" => Ok(PartNumber::Sim900),
            _ => Err(DriverError::UnsupportedModel),
        }?);

        info!("{} was setup", self.part_number.unwrap());

        let max_sockets = self.part_number.unwrap().max_sockets();
        for _ in 0..max_sockets {
            self.handle
                .socket_state
                .push(SocketState::new(SOCKET_STATE_UNKNOWN))
                .unwrap();
        }

        Ok(())
    }

    /// Check that the cellular module is alive.
    ///
    /// See if the cellular module is responding at the AT interface by poking
    /// it with "AT" up to `attempts` times, waiting 1 second for an "OK"
    /// response each time
    async fn is_alive(&mut self, attempts: u8) -> Result<(), DriverError> {
        let mut client = self.handle.client.lock().await;
        let mut error = DriverError::BaudDetection;
        for _ in 0..attempts {
            match client.send(&AT).await {
                Ok(_) => return Ok(()),
                Err(atat::Error::Timeout) => {}
                Err(e) => error = e.into(),
            };
        }
        Err(error)
    }
}

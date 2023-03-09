use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use atat::{
    asynch::{AtatClient, Client},
    AtatUrcChannel, Config, UrcSubscription,
};
use embedded_io::asynch::Write;
use futures_intrusive::sync::LocalMutex;
use heapless::Vec;

use crate::{
    commands::{gsm, urc::Urc, v25ter, AT},
    services::data::SocketError,
    DriverError, PartNumber, SimcomAtatBuffers, SimcomAtatIngress, SimcomAtatUrcChannel,
    SimcomDigester, MAX_SOCKETS,
};

pub(crate) const URC_CAPACITY: usize = 1 + 2 * (1 + MAX_SOCKETS); // A dns reply, and SEND OK and RXGET per socket + handle background
pub(crate) const URC_SUBSCRIBERS: usize = 2 + MAX_SOCKETS; // One for dns, one for handle background, and one for each socket

pub(crate) type SocketState = AtomicU8;
pub(crate) const SOCKET_STATE_UNKNOWN: u8 = 0;
pub(crate) const SOCKET_STATE_UNUSED: u8 = 1;
pub(crate) const SOCKET_STATE_USED: u8 = 2;
pub(crate) const SOCKET_STATE_DROPPED: u8 = 3;

pub struct Device<'a, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> {
    pub handle: Handle<'a, AtCl>,
    pub(crate) urc_channel: &'a AtUrcCh,
    pub(crate) part_number: Option<PartNumber>,
    pub(crate) data_service_taken: AtomicBool,
}

pub struct Handle<'a, AtCl: AtatClient> {
    pub(crate) client: LocalMutex<AtCl>,
    pub(crate) socket_state: Vec<SocketState, MAX_SOCKETS>,
    pub(crate) data_written: [AtomicBool; MAX_SOCKETS],
    pub(crate) data_available: [AtomicBool; MAX_SOCKETS],
    background_subscription: LocalMutex<UrcSubscription<'a, Urc>>,
}

impl<'a, W: Write, const INGRESS_BUF_SIZE: usize, const RES_CAPACITY: usize>
    Device<
        'a,
        Client<'a, W, INGRESS_BUF_SIZE, RES_CAPACITY>,
        SimcomAtatUrcChannel<INGRESS_BUF_SIZE>,
    >
{
    pub fn from_buffers(
        buffers: &'a SimcomAtatBuffers<INGRESS_BUF_SIZE, RES_CAPACITY>,
        tx: W,
    ) -> (
        SimcomAtatIngress<INGRESS_BUF_SIZE, RES_CAPACITY>,
        Device<
            'a,
            Client<'a, W, INGRESS_BUF_SIZE, RES_CAPACITY>,
            SimcomAtatUrcChannel<INGRESS_BUF_SIZE>,
        >,
    ) {
        let (ingress, client) = buffers.split(tx, SimcomDigester::new(), Config::new());

        (ingress, Device::new(client, &buffers.urc_channel))
    }
}

impl<'a, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Device<'a, AtCl, AtUrcCh> {
    /// Create a new device given an AT client
    pub fn new(client: AtCl, urc_channel: &'a AtUrcCh) -> Self {
        // The actual state values, except for socket_state, are cleared
        // when a socket goes from [`SOCKET_STATE_UNUSED`] to [`SOCKET_STATE_USED`].
        Self {
            handle: Handle {
                client: LocalMutex::new(client, true),
                socket_state: Vec::new(),
                data_written: Default::default(),
                data_available: Default::default(),
                background_subscription: LocalMutex::new(urc_channel.subscribe().unwrap(), false),
            },
            urc_channel,
            part_number: None,
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

impl<AtCl: AtatClient> Handle<'_, AtCl> {
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
            self.data_written[id].store(true, Ordering::Relaxed);
            self.data_available[id].store(false, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub(crate) fn drain_background_urcs(&self) {
        if let Some(mut subscription) = self.background_subscription.try_lock() {
            while let Some(urc) = subscription.try_next_message_pure() {
                self.handle_urc(urc);
            }
        }
    }

    fn handle_urc(&self, urc: Urc) {
        match urc {
            Urc::ConnectOk(_id) => {}
            Urc::ConnectFail(_id) => {}
            Urc::AlreadyConnect(id) => {
                error!("[{}] Already connected", id);
            }
            Urc::SendOk(id) => {
                debug!("[{}] Data was written", id);
                self.data_written[id].store(true, Ordering::Release);
            }
            Urc::Closed(id) => {
                warn!("[{}] Socket closed", id);
                self.socket_state[id].store(SOCKET_STATE_UNUSED, Ordering::Release);
            }
            Urc::IpLookup(result) => {
                debug!("Resolved IP for host {}", result.host);
            }
            Urc::DataAvailable(id) => {
                debug!("[{}] Data available to be read", id);
                self.data_available[id].store(true, Ordering::Release);
            }
            Urc::ReadData(result) => {
                debug!("[{}] Data read", result.id);
                self.data_available[result.id].store(result.pending_len > 0, Ordering::Release);
            }
        }
    }
}

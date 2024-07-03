use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use atat::{asynch::AtatClient, UrcSubscription};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex, pubsub::WaitResult};
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_io_async::Write;
use futures_intrusive::sync::LocalMutex;
use heapless::Vec;

use crate::{
    commands::{gsm, simcom::GetCcid, urc::Urc, v25ter, AT},
    services::data::SocketError,
    DriverError, FlowControl, PartNumber, SimcomClient, SimcomConfig, SimcomResponseSlot,
    SimcomUrcChannel, MAX_SOCKETS,
};

pub(crate) const URC_CAPACITY: usize = 1 + 3 * (1 + MAX_SOCKETS); // A dns reply, and (SEND OK + RXGET + CLOSED) per socket + background subscription
pub(crate) const URC_SUBSCRIBERS: usize = 2 + MAX_SOCKETS; // One for dns, one for background subscription, and one for each socket reply subscription

pub(crate) type SocketState = AtomicU8;
pub(crate) const SOCKET_STATE_UNKNOWN: u8 = 0;
pub(crate) const SOCKET_STATE_UNUSED: u8 = 1;
pub(crate) const SOCKET_STATE_USED: u8 = 2;
pub(crate) const SOCKET_STATE_DROPPED: u8 = 3;

pub struct SimcomDevice<'buf, 'sub, AtCl: AtatClient, Config: SimcomConfig> {
    pub handle: Handle<'sub, AtCl>,
    pub(crate) urc_channel: &'buf SimcomUrcChannel,
    pub(crate) part_number: Option<PartNumber>,
    pub(crate) data_service_taken: AtomicBool,
    config: Config,
}

pub struct Handle<'sub, AtCl: AtatClient> {
    pub(crate) client: LocalMutex<AtCl>,
    pub(crate) socket_state: Vec<SocketState, MAX_SOCKETS>,
    pub(crate) busy_writing: [AtomicBool; MAX_SOCKETS],
    pub(crate) data_available: [AtomicBool; MAX_SOCKETS],
    pub(crate) max_urc_len: usize,
    background_subscription:
        Mutex<NoopRawMutex, UrcSubscription<'sub, Urc, URC_CAPACITY, URC_SUBSCRIBERS>>,
}

impl<'buf, 'sub, W: Write, Config: SimcomConfig, const INGRESS_BUF_SIZE: usize>
    SimcomDevice<'buf, 'sub, SimcomClient<'sub, W, INGRESS_BUF_SIZE>, Config>
where
    'buf: 'sub,
{
    pub fn new(
        writer: W,
        res_slot: &'buf SimcomResponseSlot<INGRESS_BUF_SIZE>,
        buf: &'buf mut [u8],
        urc_channel: &'buf SimcomUrcChannel,
        config: Config,
    ) -> Self {
        let client = SimcomClient::new(writer, res_slot, buf, config.atat_config());
        Self::new_with_client(client, urc_channel, INGRESS_BUF_SIZE, config)
    }
}

impl<'buf, 'sub, AtCl: AtatClient, Config: SimcomConfig> SimcomDevice<'buf, 'sub, AtCl, Config>
where
    'buf: 'sub,
{
    /// Create a new device given an AT client
    pub fn new_with_client(
        client: AtCl,
        urc_channel: &'buf SimcomUrcChannel,
        max_urc_len: usize,
        config: Config,
    ) -> Self {
        // The actual state values, except for socket_state, are cleared
        // when a socket goes from [`SOCKET_STATE_UNUSED`] to [`SOCKET_STATE_USED`].
        Self {
            handle: Handle {
                client: LocalMutex::new(client, true),
                socket_state: Vec::new(),
                busy_writing: Default::default(),
                data_available: Default::default(),
                max_urc_len,
                background_subscription: Mutex::new(urc_channel.subscribe().unwrap()),
            },
            urc_channel,
            part_number: None,
            data_service_taken: AtomicBool::new(false),
            config,
        }
    }

    // Hardware reset
    pub async fn reset(&mut self) -> Result<(), DriverError> {
        let reset_pin = self.config.reset_pin();

        // SIM800 min. reset pulse length is 105ms
        // SIM900 min. reset pulse length is 50us
        reset_pin.set_low().unwrap();
        Timer::after(Duration::from_millis(150)).await;
        reset_pin.set_high().unwrap();

        // SIM800 post reset offline duration is 2.7s
        // SIM900 post reset offline duration is 1.2s
        Timer::after(Duration::from_secs(3)).await;

        Ok(())
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

        let (from_modem, to_modem) = match Config::FLOW_CONTROL {
            FlowControl::None => (v25ter::FlowControl::Disabled, v25ter::FlowControl::Disabled),
            FlowControl::RtsCts => (v25ter::FlowControl::RtsCts, v25ter::FlowControl::RtsCts),
        };

        client
            .send(&v25ter::SetFlowControl {
                from_modem,
                to_modem: Some(to_modem),
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

        let response = client.send(&gsm::GetSoftwareVersion).await?;

        info!(
            "{} with software version {} was setup",
            self.part_number.unwrap(),
            response.version.as_slice()
        );

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

    /// Get the sim card iccid
    pub async fn iccid(&self) -> Result<u128, DriverError> {
        let mut client = self.handle.client.lock().await;
        for _ in 0..10 {
            match client.send(&GetCcid).await {
                Ok(response) => {
                    let iccid =
                        core::str::from_utf8(&response.iccid).map_err(|_| atat::Error::Parse)?;
                    let iccid = iccid.parse::<u128>().map_err(|_| atat::Error::Parse)?;
                    return Ok(iccid);
                }
                Err(atat::Error::CmeError(atat::CmeError::SimNotInserted)) => {
                    // For Dandial (TDC) simcards it seems as if SimNotInserted may be returned several times when first requesting iccid
                }
                Err(e) => return Err(e.into()),
            }
            Timer::after_millis(500).await;
        }
        Err(DriverError::Atat(atat::Error::CmeError(
            atat::CmeError::SimNotInserted,
        )))
    }
}

impl<AtCl: AtatClient + 'static> Handle<'_, AtCl> {
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
            self.busy_writing[id].store(false, Ordering::Relaxed);
            self.data_available[id].store(false, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub(crate) fn drain_background_urcs(&self) {
        if let Ok(mut subscription) = self.background_subscription.try_lock() {
            while let Some(urc) = subscription.try_next_message() {
                match urc {
                    WaitResult::Message(urc) => self.handle_urc(urc),
                    WaitResult::Lagged(count) => error!("Lagged {} URC messages", count),
                }
            }
        }
    }

    fn handle_urc(&self, urc: Urc) {
        match urc {
            Urc::CallReady => {}
            Urc::SmsReady => {}
            Urc::PinStatus(_) => {}
            Urc::ConnectOk(_id) => {}
            Urc::ConnectFail(_id) => {}
            Urc::AlreadyConnect(id) => {
                error!("[{}] Already connected", id);
            }
            Urc::SendOk(id) => {
                debug!("[{}] Data written", id);
                self.busy_writing[id].store(false, Ordering::Release);
            }
            Urc::Closed(id) => {
                warn!("[{}] Socket closed", id);
                self.socket_state[id].store(SOCKET_STATE_UNUSED, Ordering::Release);
            }
            Urc::PdpDeact => info!("GPRS is disconnected by network"),
            Urc::PdbState(state) => {
                debug!("PDP state for context {} is {:?}", state.cid, state.state);
            }
            Urc::DnsResult(result) => {
                if let Ok(result) = result {
                    debug!("Resolved IP for host {}", result.host);
                } else {
                    warn!("Failed to resolve IP");
                }
            }
            Urc::DataAvailable(id) => {
                debug!("[{}] Data available to be read", id);
                self.data_available[id].store(true, Ordering::Release);
            }
            Urc::ReadData(result) => {
                debug!(
                    "[{}] Received {} bytes, there are {} pending bytes available",
                    result.id, result.data_len, result.pending_len
                );
                self.data_available[result.id].store(result.pending_len > 0, Ordering::Release);
            }
        }
    }
}

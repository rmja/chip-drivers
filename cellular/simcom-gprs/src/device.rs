use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use atat::{
    asynch::{AtatClient, Client},
    AtatUrcChannel, UrcSubscription,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;
use embedded_io::asynch::Write;
use futures_intrusive::sync::LocalMutex;
use heapless::Vec;

use crate::{
    commands::{gsm, urc::Urc, v25ter, AT},
    services::data::SocketError,
    DriverError, PartNumber, SimcomAtatBuffers, SimcomAtatIngress, SimcomAtatUrcChannel,
    SimcomDigester, MAX_SOCKETS,
};

pub(crate) const URC_CAPACITY: usize = 1 + 3 * (1 + MAX_SOCKETS); // A dns reply, and (SEND OK + RXGET + CLOSED) per socket + background subscription
pub(crate) const URC_SUBSCRIBERS: usize = 2 + MAX_SOCKETS; // One for dns, one for background subscription, and one for each socket reply subscription

pub(crate) type SocketState = AtomicU8;
pub(crate) const SOCKET_STATE_UNKNOWN: u8 = 0;
pub(crate) const SOCKET_STATE_UNUSED: u8 = 1;
pub(crate) const SOCKET_STATE_USED: u8 = 2;
pub(crate) const SOCKET_STATE_DROPPED: u8 = 3;

pub struct Device<'buf, 'sub, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>, Config: ModemConfig> {
    pub handle: Handle<'sub, AtCl>,
    pub(crate) urc_channel: &'buf AtUrcCh,
    pub(crate) part_number: Option<PartNumber>,
    pub(crate) data_service_taken: AtomicBool,
    config: Config,
}

pub trait ModemConfig {
    type ResetPin: OutputPin;

    const FLOW_CONTROL: FlowControl = FlowControl::None;

    fn reset_pin(&mut self) -> &mut Self::ResetPin;

    fn get_response_timeout(start: Instant, timeout: Duration) -> Instant {
        start + timeout
    }
}

pub enum FlowControl {
    /// No flow control is being used
    None,
    /// Hardware flow control
    RtsCts,
}

pub struct Handle<'sub, AtCl: AtatClient> {
    pub(crate) client: LocalMutex<AtCl>,
    pub(crate) socket_state: Vec<SocketState, MAX_SOCKETS>,
    pub(crate) data_written: [AtomicBool; MAX_SOCKETS],
    pub(crate) data_available: [AtomicBool; MAX_SOCKETS],
    pub(crate) max_urc_len: usize,
    background_subscription: Mutex<NoopRawMutex, UrcSubscription<'sub, Urc>>,
}

impl<'buf, 'sub, W: Write, Config: ModemConfig, const INGRESS_BUF_SIZE: usize>
    Device<'buf, 'sub, Client<'buf, W, INGRESS_BUF_SIZE>, SimcomAtatUrcChannel, Config>
where
    'buf: 'sub,
{
    pub fn from_buffers(
        buffers: &'buf SimcomAtatBuffers<INGRESS_BUF_SIZE>,
        tx: W,
        config: Config,
    ) -> (
        SimcomAtatIngress<INGRESS_BUF_SIZE>,
        Device<'buf, 'sub, Client<'buf, W, INGRESS_BUF_SIZE>, SimcomAtatUrcChannel, Config>,
    ) {
        let (ingress, client) = buffers.split(
            tx,
            SimcomDigester::new(),
            atat::Config::new().get_response_timeout(Config::get_response_timeout),
        );

        (
            ingress,
            Device::new(client, &buffers.urc_channel, INGRESS_BUF_SIZE, config),
        )
    }
}

impl<'buf, 'sub, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>, Config: ModemConfig>
    Device<'buf, 'sub, AtCl, AtUrcCh, Config>
where
    'buf: 'sub,
{
    /// Create a new device given an AT client
    pub fn new(
        client: AtCl,
        urc_channel: &'buf AtUrcCh,
        max_urc_len: usize,
        config: Config,
    ) -> Self {
        // The actual state values, except for socket_state, are cleared
        // when a socket goes from [`SOCKET_STATE_UNUSED`] to [`SOCKET_STATE_USED`].
        Self {
            handle: Handle {
                client: LocalMutex::new(client, true),
                socket_state: Vec::new(),
                data_written: Default::default(),
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
        if let Ok(mut subscription) = self.background_subscription.try_lock() {
            while let Some(urc) = subscription.try_next_message_pure() {
                self.handle_urc(urc);
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
                self.data_written[id].store(true, Ordering::Release);
            }
            Urc::Closed(id) => {
                warn!("[{}] Socket closed", id);
                self.socket_state[id].store(SOCKET_STATE_UNUSED, Ordering::Release);
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

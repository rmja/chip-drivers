use core::{sync::atomic::{AtomicBool, Ordering}, char::MAX};

use embedded_hal_async::delay::DelayUs;
use embedded_io::asynch::Write;
use embedded_time::Clock;
use futures_intrusive::sync::LocalMutex;
use heapless::Vec;

use crate::{
    atat_async::{self, AtatClient, Ingress},
    commands::{gsm, urc::Urc, v25ter, AT},
    services::{
        data::{SocketState, SOCKET_STATE_UNKNOWN, SOCKET_STATE_UNUSED},
        network::Network,
    },
    DriverError, PartNumber, SimcomDigester, MAX_SOCKETS,
};

pub struct Handle<AtCl: AtatClient> {
    pub(crate) client: LocalMutex<AtCl>,
    pub(crate) socket_state: Vec<SocketState, MAX_SOCKETS>,
    pub(crate) flushed: [AtomicBool; MAX_SOCKETS],
}

impl<AtCl: AtatClient> Handle<AtCl> {
    pub(crate) fn handle_urc(&self, urc: &Urc) -> bool {
        match urc {
            Urc::SendOk(id) => {
                self.flushed[*id].store(true, Ordering::Release);
                true
            }
            Urc::Closed(id) => {
                warn!("[{}] Socket closed", *id);
                self.socket_state[*id].store(SOCKET_STATE_UNUSED, Ordering::Release);
                true
            }
            Urc::DataAvailable(_id) => true, // Discard
            urc => {
                error!("Uhandled URC: {:?}", urc);
                false
            }
        }
    }
}

pub struct Device<AtCl: AtatClient, Delay: DelayUs + Clone> {
    pub handle: Handle<AtCl>,
    pub(crate) delay: Delay,
    pub(crate) part_number: Option<PartNumber>,
    pub network: Network<Delay>,
    pub(crate) data_service_taken: AtomicBool,
}

impl<
        'a,
        Tx: Write,
        Clk: Clock<T = u64>,
        Delay: DelayUs + Clone,
        const RES_CAPACITY: usize,
        const URC_CAPACITY: usize,
    > Device<atat_async::Client<'a, Tx, Clk, Delay, RES_CAPACITY, URC_CAPACITY>, Delay>
{
    /// Create a new device with a default AT client
    pub fn new<const INGRESS_BUF_SIZE: usize>(
        tx: Tx,
        buffers: &'a mut atat_async::Buffers<INGRESS_BUF_SIZE, RES_CAPACITY, URC_CAPACITY>,
        clock: &'a Clk,
        delay: Delay,
    ) -> (
        Ingress<'a, SimcomDigester, INGRESS_BUF_SIZE, RES_CAPACITY, URC_CAPACITY>,
        Self,
    ) {
        let (ingress, at_client) = buffers.split(
            tx,
            clock,
            delay.clone(),
            SimcomDigester::new(),
            atat_async::Config::default(),
        );

        let device = Self::with_at_client(at_client, delay);
        (ingress, device)
    }
}

impl<AtCl: AtatClient, Delay: DelayUs + Clone> Device<AtCl, Delay> {
    /// Create a new device given an AT client
    pub fn with_at_client(at_client: AtCl, delay: Delay) -> Self {
        let network = Network::new(delay.clone());
        const TRUE: AtomicBool = AtomicBool::new(true);
        Self {
            handle: Handle {
                client: LocalMutex::new(at_client, true),
                socket_state: Vec::new(),
                flushed: [TRUE; MAX_SOCKETS],
            },
            delay,
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

use core::marker::PhantomData;

use embassy_time::Instant;
use embedded_hal_async::{delay::DelayUs, spi};
use futures::Stream;
use futures_async_stream::stream;

use crate::{
    gpio::{Gpio, GpioOutput},
    regs::{
        ext::FreqoffCfg,
        pri::{
            AgcCfg3, AgcSyncBehaviourValue, FifoCfg, LengthConfigValue, Mdmcfg1, PktCfg0, PktCfg2,
            PktFormatValue, RfendCfg1, RxoffModeValue,
        },
        Iocfg,
    },
    ConfigPatch, Driver, Rssi, State, Strobe,
};

use super::ControllerError;

pub struct SerialController<'a, Spi, Delay, ResetPin, IrqGpio, IrqPin, const CHUNK_SIZE: usize = 16>
where
    Spi: spi::SpiDevice,
    Delay: DelayUs,
    ResetPin: embedded_hal::digital::OutputPin,
    IrqGpio: Gpio,
    IrqPin: embedded_hal_async::digital::Wait,
{
    driver: &'a mut Driver<Spi, Delay, ResetPin>,
    config: ConfigPatch<'a>,
    irq_gpio: PhantomData<IrqGpio>,
    irq_pin: &'a mut IrqPin,
    is_idle: bool,
}

#[derive(Debug)]
pub struct RxChunk<const CHUNK_SIZE: usize = 16> {
    /// The timestamp sampled when `fifo_thr` bytes has arrived in the CC1200 rx buffer.
    pub timestamp: Instant,
    /// The rssi sampled after `fifo_thr` bytes are in the rx buffer, that is, it corresponds to the rssi of the last byte.
    pub rssi: Option<Rssi>,
    /// The received bytes.
    pub bytes: [u8; CHUNK_SIZE],
}

impl<
        'a,
        Spi: spi::SpiDevice,
        Delay: DelayUs,
        ResetPin: embedded_hal::digital::OutputPin,
        IrqGpio: Gpio,
        IrqPin: embedded_hal_async::digital::Wait,
        const CHUNK_SIZE: usize,
    > SerialController<'a, Spi, Delay, ResetPin, IrqGpio, IrqPin, CHUNK_SIZE>
{
    /// Create a new serial controller
    pub fn new(
        driver: &'a mut Driver<Spi, Delay, ResetPin>,
        irq_pin: &'a mut IrqPin,
        config: ConfigPatch<'a>,
    ) -> Self {
        Self {
            driver,
            config,
            irq_gpio: PhantomData,
            irq_pin,
            is_idle: true,
        }
    }

    /// Initialize the chip by sending a configuration and entering idle state
    pub async fn init(&mut self) -> Result<(), ControllerError> {
        self.driver.write_patch(self.config).await?;

        // Do not freeze automatic gain control and rssi measurements
        let mut agccfg3 = self.config.get::<AgcCfg3>().unwrap_or_default();
        agccfg3.set_agc_sync_behaviour(AgcSyncBehaviourValue::NoAgcGainFreeze_000);
        self.driver.write_reg(agccfg3).await?;

        // Do not disable frequency offset compensation after sync word is detected
        let mut freqoffcfg = self.config.get::<FreqoffCfg>().unwrap_or_default();
        if freqoffcfg.foc_ki_factor() == 0 {
            freqoffcfg.set_foc_ki_factor(0b10); // Enable with loop gain factor = 1/64
            self.driver.write_reg(freqoffcfg).await?;
        }

        // FIFO must be enabled
        let mut mdmcfg1 = self.config.get::<Mdmcfg1>().unwrap_or_default();
        mdmcfg1.set_fifo_en(true);
        self.driver.write_reg(mdmcfg1).await?;

        // Packet mode must be Normal/FIFO mode
        let mut pktcfg2 = self.config.get::<PktCfg2>().unwrap_or_default();
        pktcfg2.set_pkt_format(PktFormatValue::NormalModeFifoMode);
        self.driver.write_reg(pktcfg2).await?;

        // Must re-enter RX when RX ends
        let mut rfendcfg1 = self.config.get::<RfendCfg1>().unwrap_or_default();
        rfendcfg1.set_rxoff_mode(RxoffModeValue::Rx);
        self.driver.write_reg(rfendcfg1).await?;

        self.idle().await?;

        Ok(())
    }

    /// Start and run receiver.
    /// Note that the receiver is _not_ stopped when the stream is dropped, so idle() must be called manually after the stream is dropped.
    pub async fn receive<'r>(
        &'r mut self,
    ) -> Result<
        impl Stream<Item = Result<RxChunk<CHUNK_SIZE>, ControllerError>> + 'r,
        ControllerError,
    >
    where
        'r: 'a,
    {
        assert!(self.is_idle);

        // Configure the fifo threshold to match the chunk size
        let mut fifo_cfg = self.config.get::<FifoCfg>().unwrap();
        fifo_cfg.set_bytes_in_rxfifo(CHUNK_SIZE as u8);
        self.driver.write_reg(fifo_cfg).await?;

        // Use infinite packet mode
        let mut pktcfg0 = self.config.get::<PktCfg0>().unwrap_or_default();
        pktcfg0.set_length_config(LengthConfigValue::InfinitePacketLengthMode);
        self.driver.write_reg(pktcfg0).await?;

        // Setup fifo pin
        // Asserted when sync word has been received
        let mut irq_iocfg = IrqGpio::Iocfg::default();
        irq_iocfg.set_gpio_cfg(GpioOutput::RXFIFO_THR);
        self.driver.write_reg(irq_iocfg).await?;

        // Flush RX buffer before we start the receiver
        // This can only be safely done if the chip is in IDLE state.
        self.driver.strobe(Strobe::SFRX).await?;

        // Start receiver - do not wait for calibration and settling.
        // If this is done while alraedy receiving this restart the demodulator to catch a new incoming packet.
        self.driver.strobe(Strobe::SRX).await?;

        self.is_idle = false;

        Ok(self.receive_stream())
    }

    #[stream(item = Result<RxChunk<CHUNK_SIZE>, ControllerError>)]
    async fn receive_stream<'r>(&'r mut self)
    where
        'r: 'a,
    {
        loop {
            self.irq_pin.wait_for_high().await.unwrap();
            let timestamp = Instant::now();

            let mut chunk_bytes = [0; CHUNK_SIZE];
            let rssi = unsafe {
                self.driver
                    .read_rssi_and_fifo_raw(&mut chunk_bytes)
                    .await
                    .unwrap()
            };

            match self.driver.last_status().unwrap().state() {
                State::RX => {
                    yield Ok(RxChunk {
                        timestamp,
                        rssi,
                        bytes: chunk_bytes,
                    });
                }
                State::CALIBRATE => {}
                State::SETTLING => {}
                State::RX_FIFO_ERROR => {
                    // Enter idle state
                    self.driver.strobe_until_idle(Strobe::SIDLE).await.unwrap();

                    yield Err(ControllerError::FifoOverflow);

                    // Re-start receiver
                    self.driver.strobe(Strobe::SFRX).await.unwrap();
                    self.driver.strobe(Strobe::SRX).await.unwrap();
                }
                State::TX_FIFO_ERROR => {
                    // This should really not be possible, but it seems to happen anyway
                    // Maybe because of chip calibration
                    // Enter idle state
                    self.driver.strobe_until_idle(Strobe::SIDLE).await.unwrap();

                    // Emit as fifo error
                    yield Err(ControllerError::FifoOverflow);

                    // Flush the tx fifo
                    self.driver.strobe(Strobe::SFTX).await.unwrap();

                    // Re-start receiver
                    self.driver.strobe(Strobe::SFRX).await.unwrap();
                    self.driver.strobe(Strobe::SRX).await.unwrap();
                }
                state => {
                    self.driver.strobe_until_idle(Strobe::SIDLE).await.unwrap();

                    yield Err(ControllerError::UnrecoverableChipState(state));

                    // Re-start receiver
                    self.driver.strobe(Strobe::SFRX).await.unwrap();
                    self.driver.strobe(Strobe::SRX).await.unwrap();
                }
            }
        }
    }

    /// Transition chip to idle state
    pub async fn idle(&mut self) -> Result<(), ControllerError> {
        self.driver.strobe_until_idle(Strobe::SIDLE).await?;
        self.is_idle = true;
        Ok(())
    }
}

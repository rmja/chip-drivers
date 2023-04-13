//! Default packet controller
//!
//! # Examples
//!
//! Typical receive sequence:
//! ```ignore
//! Method      SPI_TX      Description
//! init()      40...       Write primary registers
//!             6F00...     Write extended registers
//!             6F0A0002    Write freq calibration
//!             1142        Enable FIFO
//!             2600        Normal FIFO mode
//!             36          IDLE
//!
//! listen()    3A          Flush RX FIFO
//!             34          Start RX
//!
//! receive()   2840        Set infinite packet length mode
//!             34          Start RX, i.e. restart demodulator
//!             AFD700FF... Empty FIFO
//!             1D02        Set FIFO threshold to 3 bytes
//!             0306        Set IRQ output to SOF detected
//!
//!             -- Wait for IRQ, i.e. SOF to be detected --
//!
//!             0300        Set IRQ output to FIFO threshold
//!                         Wait for FIFO threshold to be reached
//!
//! read()      AFD700FF... Read FIFO
//!             1D0F        Set FIFO threshold to 16 bytes
//!
//! accept()    2EXX        Set packet length PKT_LEN
//!
//! get_rssi()  AF7100      Read RSSI
//!
//! read()      2800        Use fixed packet length mode (PKT_LEN is used)
//!             0346        Set IRQ output to EOF detected
//!                         Wait for EOF to be detected
//!             AFD700FF... Read FIFO
//! ```

use core::marker::PhantomData;

use crate::{
    driver::lo_divider,
    gpio::{Gpio, GpioOutput},
    regs::{
        ext::Freq2,
        pri::{
            FifoCfg, LengthConfigValue, Mdmcfg1, PktCfg0, PktCfg1, PktCfg2, PktFormatValue, PktLen,
            RfendCfg0, RfendCfg1, RxoffModeValue, TxoffModeValue,
        },
        Iocfg, Register,
    },
    ConfigPatch, Driver, Rssi, State, Strobe, RX_FIFO_SIZE, TX_FIFO_SIZE,
};
use embassy_time::Instant;
use embedded_hal_async::{delay::DelayUs, spi as async_spi};
use heapless::Vec;

use super::ControllerError;

pub struct PacketController<
    'a,
    Spi,
    SpiBus,
    Delay,
    ResetPin,
    IrqGpio,
    IrqPin,
    const WRITE_QUEUE_CAPACITY: usize,
> where
    Spi: async_spi::SpiDevice<Bus = SpiBus>,
    SpiBus: async_spi::SpiBus + 'static,
    Delay: DelayUs,
    ResetPin: embedded_hal::digital::OutputPin,
    IrqGpio: Gpio,
    IrqPin: embedded_hal_async::digital::Wait,
{
    driver: &'a mut Driver<Spi, Delay, ResetPin>,
    config: ConfigPatch<'a>,
    pktcfg0: PktCfg0,
    irq_iocfg: IrqGpio::Iocfg,
    irq_gpio: PhantomData<IrqGpio>,
    irq_pin: &'a mut IrqPin,
    pending_write_queue: Vec<u8, WRITE_QUEUE_CAPACITY>,
    written_to_txfifo: usize,
    is_idle: bool,
}

pub struct RxToken {
    pub timestamp: Instant,
    read_from_rxfifo: usize,
    frame_length: Option<usize>,
}

impl<'a, Spi, SpiBus, Delay, ResetPin, IrqGpio, IrqPin, const WRITE_QUEUE_CAPACITY: usize>
    PacketController<'a, Spi, SpiBus, Delay, ResetPin, IrqGpio, IrqPin, WRITE_QUEUE_CAPACITY>
where
    Spi: async_spi::SpiDevice<Bus = SpiBus>,
    SpiBus: async_spi::SpiBus + 'static,
    Delay: DelayUs,
    ResetPin: embedded_hal::digital::OutputPin,
    IrqGpio: Gpio,
    IrqPin: embedded_hal_async::digital::Wait,
{
    /// Create a new packet controller
    pub fn new(
        driver: &'a mut Driver<Spi, Delay, ResetPin>,
        irq_pin: &'a mut IrqPin,
        config: ConfigPatch<'a>,
    ) -> Self {
        Self {
            driver,
            config,
            pktcfg0: config.get::<PktCfg0>().unwrap(),
            irq_iocfg: config.get::<IrqGpio::Iocfg>().unwrap(),
            irq_gpio: PhantomData,
            irq_pin,
            pending_write_queue: Vec::new(),
            written_to_txfifo: 0,
            is_idle: true,
        }
    }

    /// Initialize the chip by sending a configuration and entering idle state
    pub async fn init(&mut self) -> Result<(), ControllerError> {
        self.driver.write_patch(self.config).await?;

        // FIFO must be enabled
        let mut mdmcfg1 = self.config.get::<Mdmcfg1>().unwrap_or_default();
        mdmcfg1.set_fifo_en(true);
        self.driver.write_reg(mdmcfg1).await?;

        // Packet mode must be Normal/FIFO mode
        let mut pktcfg2 = self.config.get::<PktCfg2>().unwrap_or_default();
        pktcfg2.set_pkt_format(PktFormatValue::NormalModeFifoMode);
        self.driver.write_reg(pktcfg2).await?;

        // Status byte must not be appended
        let mut pktcfg1 = self.config.get::<PktCfg1>().unwrap_or_default();
        pktcfg1.set_append_status(false);
        self.driver.write_reg(pktcfg1).await?;

        self.idle().await?;

        Ok(())
    }

    /// Set the frequency in Hz
    pub async fn set_frequency(&mut self, frequency: u32) -> Result<(), ControllerError> {
        let lo_div = lo_divider(frequency) as u32;
        let freq: [u8; 4] = (frequency * lo_div).to_be_bytes();
        let patch = ConfigPatch {
            first_address: Freq2::ADDRESS,
            values: &freq[1..],
        };
        self.driver.write_patch(patch).await?;
        Ok(())
    }

    /// Write bytes to the chip tx fifo
    /// Bytes that cannot fit in the tx fifo are buffered and written during transmission
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), ControllerError> {
        let write_now_length = usize::min(buffer.len(), TX_FIFO_SIZE - self.written_to_txfifo);
        let (write_now, write_later) = buffer.split_at(write_now_length);

        if !write_now.is_empty() {
            self.driver.write_fifo(write_now).await?;
            self.written_to_txfifo += write_now.len();
        }

        if !write_later.is_empty() {
            self.pending_write_queue
                .extend_from_slice(write_later)
                .map_err(|_| ControllerError::WriteCapacity)?;
        }

        Ok(())
    }

    /// Start transmission of previously written bytes
    pub async fn transmit(&mut self) -> Result<(), ControllerError> {
        assert_ne!(
            0, self.written_to_txfifo,
            "write() was not called prior to starting transmission"
        );

        let length = self.written_to_txfifo + self.pending_write_queue.len();

        // Setup fifo pin
        // Asserted when the TX FIFO is filled above threshold
        self.irq_iocfg = IrqGpio::Iocfg::default();
        self.irq_iocfg.set_gpio_cfg(GpioOutput::TXFIFO_THR);
        self.driver.write_reg(self.irq_iocfg).await?;

        // Set frame length configuration
        if length <= 256 {
            self.pktcfg0
                .set_length_config(LengthConfigValue::FixedPacketLengthMode);
        } else {
            self.pktcfg0
                .set_length_config(LengthConfigValue::InfinitePacketLengthMode);
        };
        self.driver.write_reg(self.pktcfg0).await?;

        // Set frame length.
        let pktlen = PktLen((length & 0xFF) as u8);
        self.driver.write_reg(pktlen).await?;

        // Start transmitter.
        self.driver.strobe(Strobe::STX).await?;

        // Do not wait for calibration and settling.

        let fifocfg = self.config.get::<FifoCfg>().unwrap();
        while !self.pending_write_queue.is_empty() {
            // Wait for fifo buffer to go below threshold.
            self.irq_pin.wait_for_low().await.unwrap();

            if self.pktcfg0.length_config() != LengthConfigValue::FixedPacketLengthMode
                && self.pending_write_queue.len() <= TX_FIFO_SIZE
            {
                // We are so far in the transmission that we can now transition from
                // infinite packet length mode to fixed packet length mode.
                self.pktcfg0
                    .set_length_config(LengthConfigValue::FixedPacketLengthMode);
                self.driver.write_reg(self.pktcfg0).await?;
            }

            let length = core::cmp::min(
                self.pending_write_queue.len(),
                fifocfg.bytes_in_rxfifo() as usize,
            );
            self.driver
                .write_fifo(&self.pending_write_queue[..length])
                .await?;
            self.pending_write_queue.remove(length);

            if self.driver.last_status().unwrap().state() == State::TX_FIFO_ERROR {
                // It seems that we came too late with the FIFO refill.
                // Flush TX buffer.
                self.driver.strobe(Strobe::SFTX).await?;

                self.pending_write_queue.clear();
                self.written_to_txfifo = 0;

                return Err(ControllerError::TxFifoUnderflow);
            }
        }

        // Wait for fifo buffer to go below threshold.
        self.irq_pin.wait_for_low().await.unwrap();

        // Re-define fifo pin to be de-asserted when idle (not rx nor tx).
        self.irq_iocfg = IrqGpio::Iocfg::default();
        self.irq_iocfg.set_gpio_cfg(GpioOutput::RX0TX1_CFG);
        self.driver.write_reg(self.irq_iocfg).await?;

        // Wait for transmission to complete.
        self.irq_pin.wait_for_low().await.unwrap();

        self.written_to_txfifo = 0;

        self.is_idle = match self
            .config
            .get::<RfendCfg0>()
            .unwrap_or_default()
            .txoff_mode()
        {
            TxoffModeValue::Idle => true,
            TxoffModeValue::Rx => false,
            _ => panic!("Unsupported state after tx completes"),
        };

        Ok(())
    }

    /// Start the receiver on the chip
    pub async fn listen(&mut self) -> Result<(), ControllerError> {
        assert!(self.is_idle);

        // Flush RX buffer before we start the receiver
        // This can only be safely done if the chip is in IDLE state.
        self.driver.strobe(Strobe::SFRX).await?;

        // Start receiver - do not wait for calibration and settling.
        // If this is done while alraedy receiving this restart the demodulator to catch a new incoming packet.
        self.driver.strobe(Strobe::SRX).await?;

        self.is_idle = false;

        Ok(())
    }

    /// Read the current rssi level
    pub async fn get_rssi(&mut self) -> Result<Rssi, ControllerError> {
        let rssi = self.driver.read_rssi().await?;
        Ok(rssi)
    }

    /// Start waiting for a packet to be detected
    /// This call completes when `min_frame_length` bytes have been received.
    pub async fn receive(&mut self, min_frame_length: usize) -> Result<RxToken, ControllerError> {
        assert!(min_frame_length > 0);
        assert!(min_frame_length <= RX_FIFO_SIZE);

        // Ensure infinite packet length mode.
        if self.pktcfg0.length_config() != LengthConfigValue::InfinitePacketLengthMode {
            self.pktcfg0
                .set_length_config(LengthConfigValue::InfinitePacketLengthMode);
            self.driver.write_reg(self.pktcfg0).await?;
        }

        // Start receiver - do not wait for calibration and settling.
        // If this is done while already receiving this restarts the demodulator to catch a new incoming packet.
        self.driver.strobe(Strobe::SRX).await?;

        // Make sure that the RX fifo is empty
        // We cannot send a FLUSH RX FIFO strobe as that cannot be sent while in RX.
        self.driver.empty_fifo().await?;

        // Configure the fifo threshold to be min_frame_length
        let mut fifo_cfg = self.config.get::<FifoCfg>().unwrap();
        fifo_cfg.set_bytes_in_rxfifo(min_frame_length as u8);
        self.driver.write_reg(fifo_cfg).await?;

        // Setup fifo pin
        // Asserted when sync word has been received
        self.irq_iocfg = IrqGpio::Iocfg::default();
        self.irq_iocfg.set_gpio_cfg(GpioOutput::PKT_SYNC_RXTX);
        self.driver.write_reg(self.irq_iocfg).await?;

        // Wait for SOF to be detected
        self.irq_pin.wait_for_high().await.unwrap();
        let timestamp = Instant::now();

        // Setup fifo pin
        // Asserted when fifo is above threshold and deasserted when drained below threshold.
        self.irq_iocfg = IrqGpio::Iocfg::default();
        self.irq_iocfg.set_gpio_cfg(GpioOutput::RXFIFO_THR);
        self.driver.write_reg(self.irq_iocfg).await?;

        // Wait for min_frame_length bytes to be received
        self.irq_pin.wait_for_high().await.unwrap();

        Ok(RxToken {
            timestamp,
            read_from_rxfifo: 0,
            frame_length: None,
        })
    }

    /// Read from the rx fifo for the frame currently being received.
    /// This call must be called regulary while a frame is being received to ensure that
    /// the rx fifo does not overflow
    pub async fn read(
        &mut self,
        token: &mut RxToken,
        buffer: &mut [u8],
    ) -> Result<usize, ControllerError> {
        // Determine if it is time to transition to fixed packet length mode
        if self.pktcfg0.length_config() == LengthConfigValue::InfinitePacketLengthMode && let Some(frame_length) = token.frame_length && token.read_from_rxfifo + RX_FIFO_SIZE >= frame_length {
                // We are now sufficently close to the end of the packet.
                // The remaining bytes that we need to receive can fit in the RX fifo.
                // Transition to fixed packet length mode.
                self.pktcfg0
                    .set_length_config(LengthConfigValue::FixedPacketLengthMode);
                self.driver
                    .write_reg(self.pktcfg0)
                    .await?;

                // Setup fifo pin
                // Asserted when end of packet is reached.
                self.irq_iocfg = IrqGpio::Iocfg::default();
                self.irq_iocfg.set_gpio_cfg(GpioOutput::PKT_SYNC_RXTX);
                self.irq_iocfg.set_gpio_inv(true);
                self.driver.write_reg(self.irq_iocfg).await?;
        }

        // Wait for the FIFO to reach threshold or for packet to be fully received
        self.irq_pin.wait_for_high().await.unwrap();

        // Read bytes available in the fifo
        let received = self.driver.read_fifo(buffer).await?;

        if self.driver.last_status().unwrap().state() == State::RX_FIFO_ERROR {
            return Err(ControllerError::RxFifoOverflow);
        }

        if received > 0 && token.read_from_rxfifo == 0 {
            // This is the first portion received

            // Configure the fifo threshold to the default
            let fifo_cfg = self.config.get::<FifoCfg>().unwrap();
            self.driver.write_reg(fifo_cfg).await?;
        }

        token.read_from_rxfifo += received;

        Ok(received)
    }

    /// Set the length of the frame being received.
    pub async fn accept(
        &mut self,
        token: &mut RxToken,
        frame_length: usize,
    ) -> Result<(), ControllerError> {
        assert_eq!(
            LengthConfigValue::InfinitePacketLengthMode,
            self.pktcfg0.length_config()
        );

        if frame_length > token.read_from_rxfifo {
            // Set the least significant byte of the frame length.
            let mut pktlen = PktLen::default();
            pktlen.set_packet_length((frame_length & 0xFF) as u8);
            self.driver.write_reg(pktlen).await?;

            token.frame_length = Some(frame_length);
        } else {
            // We have already read more bytes than the length of the frame
            // End the current frame asap and resume after receive.

            self.is_idle = match self
                .config
                .get::<RfendCfg1>()
                .unwrap_or_default()
                .rxoff_mode()
            {
                RxoffModeValue::Idle => true,
                RxoffModeValue::Rx => false,
                _ => panic!("Unsupported state after rx completes"),
            };

            if self.is_idle {
                self.driver.strobe_until_idle(Strobe::SIDLE).await?;
            }
        }

        Ok(())
    }

    /// Stop receiving by setting chip to idle.
    pub async fn idle(&mut self) -> Result<(), ControllerError> {
        self.driver.strobe_until_idle(Strobe::SIDLE).await?;

        self.is_idle = true;

        Ok(())
    }
}

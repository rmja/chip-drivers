//! Default packet controller
//!
//! # Examples
//!
//! Typical receive sequence:
//! Method      SPI_TX  Description
//! listen()    3A          Flush RX FIFO
//!             34          Start RX
//!
//! receive()   2840        Set infinite packet length mode
//!             34          Start RX, i.e. restart demodulator
//!             AFD700FF    Empty FIFO
//!             1D02        Set FIFO threshold to 3 bytes
//!             0306        Set IRQ output to SOF detected
//!                         Wait for SOF to be detected
//!             0300        Set IRQ output to FIFO threshold
//!                         Wait for FIFO threshold to be reached
//!
//! read()      AFD700FF    Read FIFO
//!             1D0F        Set FIFO threshold to 16 bytes
//!
//! accept()    2EXX        Set packet length PKT_LEN
//!
//! get_rssi()  AF7100      Read RSSI
//!
//! read()      2800        Use fixed packet length mode (PKT_LEN is used)
//!             0346        Set IRQ output to EOF detected
//!                         Wait for EOF to be detected
//!             AFD700FF    Read FIFO

use core::marker::PhantomData;

use crate::{
    gpio::{Gpio, GpioOutput},
    regs::{
        pri::{
            FifoCfg, LengthConfigValue, Mdmcfg1, PktCfg0, PktCfg2, PktFormatValue, PktLen,
            RfendCfg0, RfendCfg1, RxoffModeValue, TxoffModeValue,
        },
        Iocfg,
    },
    ConfigPatch, Driver, DriverError, Rssi, State, Strobe, RX_FIFO_SIZE, TX_FIFO_SIZE,
};
use alloc::vec::Vec;
use embedded_hal_async::spi;

use super::traits;

pub struct PacketController<'a, Spi, SpiBus, Delay, ResetPin, IrqGpio, IrqPin, Timestamp>
where
    Spi: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: embedded_hal_async::spi::SpiBus + 'static,
    Delay: embedded_hal_async::delay::DelayUs,
    ResetPin: embedded_hal::digital::OutputPin,
    IrqGpio: Gpio,
    IrqPin: traits::IrqPin<Timestamp>,
{
    pub driver: Driver<Spi, SpiBus, Delay, ResetPin>,
    pri_config: ConfigPatch<'a>,
    ext_config: ConfigPatch<'a>,
    pktcfg0: PktCfg0,
    irq_iocfg: IrqGpio::Iocfg,
    irq_gpio: PhantomData<IrqGpio>,
    irq_pin: IrqPin,
    timestamp: PhantomData<Timestamp>,
    pending_write_queue: Vec<u8>,
    written_to_txfifo: usize,
    is_idle: bool,
}

pub struct RxToken<Timestamp> {
    pub timestamp: Timestamp,
    read_from_rxfifo: usize,
    frame_length: Option<usize>,
}

impl<'a, Spi, SpiBus, Delay, ResetPin, IrqGpio, IrqPin, Timestamp>
    PacketController<'a, Spi, SpiBus, Delay, ResetPin, IrqGpio, IrqPin, Timestamp>
where
    Spi: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: embedded_hal_async::spi::SpiBus,
    Delay: embedded_hal_async::delay::DelayUs,
    ResetPin: embedded_hal::digital::OutputPin,
    IrqGpio: Gpio,
    IrqPin: traits::IrqPin<Timestamp>,
{
    type RxToken = RxToken<Timestamp>;
    type Error = DriverError<Spi::Error, Delay>;

    /// Create a new packet controller
    pub fn new(
        driver: Driver<Spi, SpiBus, Delay, ResetPin>,
        irq_pin: IrqPin,
        pri_config: ConfigPatch<'a>,
        ext_config: ConfigPatch<'a>,
    ) -> Self {
        Self {
            driver,
            pri_config,
            ext_config,
            pktcfg0: pri_config.get::<PktCfg0>().unwrap(),
            irq_iocfg: pri_config.get::<IrqGpio::Iocfg>().unwrap(),
            irq_gpio: PhantomData,
            irq_pin,
            timestamp: PhantomData,
            pending_write_queue: Vec::new(),
            written_to_txfifo: 0,
            is_idle: true,
        }
    }

    /// Initialize the chip by sending a configuration and entering idle state
    pub async fn init(&mut self) -> Result<(), Self::Error> {
        self.driver.write_patch(self.pri_config).await?;
        self.driver.write_patch(self.ext_config).await?;

        // FIFO must be enabled
        let mut mdmcfg1 = self.pri_config.get::<Mdmcfg1>().unwrap_or_default();
        mdmcfg1.set_fifo_en(true);
        self.driver.write_reg(mdmcfg1).await?;

        // Packet mode must be Normal/FIFO mode
        let mut pktcfg2 = self.pri_config.get::<PktCfg2>().unwrap_or_default();
        pktcfg2.set_pkt_format(PktFormatValue::NormalModeFifoMode);
        self.driver.write_reg(pktcfg2).await?;

        self.idle().await?;

        Ok(())
    }

    /// Write bytes to the chip tx fifo
    /// Bytes that cannot fit in the tx fifo are buffered and written during transmission
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        let write_now_length = usize::min(buffer.len(), TX_FIFO_SIZE - self.written_to_txfifo);
        let (write_now, write_later) = buffer.split_at(write_now_length);

        if !write_now.is_empty() {
            self.driver.write_fifo(write_now).await?;
            self.written_to_txfifo += write_now.len();
        }

        if !write_later.is_empty() {
            self.pending_write_queue.extend_from_slice(write_later);
        }

        Ok(())
    }

    /// Start transmission of previously written bytes
    pub async fn transmit(&mut self) -> Result<(), Self::Error> {
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

        let fifocfg = self.pri_config.get::<FifoCfg>().unwrap();
        while !self.pending_write_queue.is_empty() {
            // Wait for fifo buffer to go below threshold.
            self.irq_pin.wait_for_low().await;

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
            self.pending_write_queue.drain(0..length);

            if self.driver.last_status().unwrap().state() == State::TX_FIFO_ERROR {
                // It seems that we came too late with the FIFO refill.
                // Flush TX buffer.
                self.driver.strobe(Strobe::SFTX).await?;

                self.pending_write_queue.clear();
                self.written_to_txfifo = 0;

                return Err(DriverError::TxFifoUnderflow);
            }
        }

        // Wait for fifo buffer to go below threshold.
        self.irq_pin.wait_for_low().await;

        // Re-define fifo pin to be de-asserted when idle (not rx nor tx).
        self.irq_iocfg = IrqGpio::Iocfg::default();
        self.irq_iocfg.set_gpio_cfg(GpioOutput::RX0TX1_CFG);
        self.driver.write_reg(self.irq_iocfg).await?;

        // Wait for transmission to complete.
        self.irq_pin.wait_for_low().await;

        self.written_to_txfifo = 0;

        self.is_idle = match self.pri_config.get::<RfendCfg0>().unwrap().txoff_mode() {
            TxoffModeValue::Idle => true,
            TxoffModeValue::Rx => false,
            _ => panic!("Unsupported state after tx completes"),
        };

        Ok(())
    }

    /// Start the receiver on the chip
    pub async fn listen(&mut self) -> Result<(), Self::Error> {
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
    pub async fn get_rssi(&mut self) -> Result<Rssi, Self::Error> {
        self.driver.read_rssi().await
    }

    /// Start waiting for a packet to be detected
    /// This call completes when `min_frame_length` bytes have been received.
    pub async fn receive(&mut self, min_frame_length: usize) -> Result<Self::RxToken, Self::Error> {
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
        let mut fifo_cfg = self.pri_config.get::<FifoCfg>().unwrap();
        fifo_cfg.set_bytes_in_rxfifo(min_frame_length as u8);
        self.driver.write_reg(fifo_cfg).await?;

        // Setup fifo pin
        // Asserted when sync word has been received
        self.irq_iocfg = IrqGpio::Iocfg::default();
        self.irq_iocfg.set_gpio_cfg(GpioOutput::PKT_SYNC_RXTX);
        self.driver.write_reg(self.irq_iocfg).await?;

        // Wait for SOF to be detected
        let timestamp = self.irq_pin.wait_for_high().await;

        // Setup fifo pin
        // Asserted when fifo is above threshold and deasserted when drained below threshold.
        self.irq_iocfg = IrqGpio::Iocfg::default();
        self.irq_iocfg.set_gpio_cfg(GpioOutput::RXFIFO_THR);
        self.driver.write_reg(self.irq_iocfg).await?;

        // Wait for min_frame_length bytes to be received
        self.irq_pin.wait_for_high().await;

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
        token: &mut Self::RxToken,
        buffer: &mut [u8],
    ) -> Result<usize, Self::Error> {
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
        self.irq_pin.wait_for_high().await;

        // Read bytes available in the fifo
        let received = self.driver.read_fifo(buffer).await?;

        if self.driver.last_status().unwrap().state() == State::RX_FIFO_ERROR {
            return Err(DriverError::RxFifoOverflow);
        }

        if received > 0 && token.read_from_rxfifo == 0 {
            // This is the first portion received

            // Configure the fifo threshold to the default
            let fifo_cfg = self.pri_config.get::<FifoCfg>().unwrap();
            self.driver.write_reg(fifo_cfg).await?;
        }

        token.read_from_rxfifo += received;

        Ok(received)
    }

    /// Set the length of the frame being received.
    pub async fn accept(
        &mut self,
        token: &mut Self::RxToken,
        frame_length: usize,
    ) -> Result<(), Self::Error> {
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

            self.is_idle = match self.pri_config.get::<RfendCfg1>().unwrap().rxoff_mode() {
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
    pub async fn idle(&mut self) -> Result<(), Self::Error> {
        self.driver.strobe_until_idle(Strobe::SIDLE).await?;

        self.is_idle = true;

        Ok(())
    }
}

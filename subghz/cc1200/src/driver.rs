use crate::{
    opcode::{ExtReg, Opcode, Reg, Strobe, OPCODE_MAX},
    statusbyte::{State, StatusByte},
    traits, ConfigPatch, DriverError, PartNumber, Rssi, RX_FIFO_SIZE, TX_FIFO_SIZE,
};
use alloc::vec;
use embedded_hal_async::{delay, spi, spi_transaction};
use futures::{
    future::{self, Either},
    pin_mut,
};

pub struct Driver<Spi, SpiBus, Delay, Pins>
where
    Spi: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: spi::SpiBus,
    Delay: delay::DelayUs,
    Pins: traits::Pins,
{
    spi: Spi,
    delay: Delay,
    pins: Pins,
    last_status: Option<StatusByte>,
    pub rssi_offset: Rssi,
}

impl<Spi, SpiBus, Delay, Pins> Driver<Spi, SpiBus, Delay, Pins>
where
    Spi: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: spi::SpiBus,
    Delay: delay::DelayUs,
    Pins: traits::Pins,
{
    type Error = DriverError<Spi::Error, Delay>;

    pub fn new(spi: Spi, delay: Delay, pins: Pins) -> Self {
        Self {
            spi,
            pins,
            delay,
            last_status: None,
            rssi_offset: -99, // Default offset defined in users guide
        }
    }

    /// Initialize chip by releasing reset pin.
    pub async fn init(&mut self) -> Result<(), Self::Error> {
        self.pins.set_reset(); // Release chip reset pin.
        future::ready(()).await;
        Ok(())
    }

    /// Send a hardware reset to chip and wait for it to become available.
    pub async fn hw_reset(&mut self) -> Result<(), Self::Error> {
        // Reset chip.
        self.pins.clear_reset(); // Trigger chip reset pin.
        self.delay.delay_ms(2).await.map_err(DriverError::Delay)?;
        self.pins.set_reset(); // Release chip reset pin.

        // Wait for chip to become available.
        let delay = &mut self.delay;
        let stabilized = spi_transaction!(&mut self.spi, move |bus| async move {
            // Wait 1ms until the chip has had a chance to set the SO pin high.
            // We must unwrap as the transaction can only return `SpiBus::Error`.
            delay.delay_ms(1).await.unwrap();
            let stabilized = Self::wait_for_xtal(bus, delay).await?;
            Ok(stabilized)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = None;

        if stabilized {
            Ok(())
        } else {
            Err(DriverError::Timeout)
        }
    }

    /// Get the spi status returned by the last register read or strobe.
    /// Writing registers does not update status.
    pub fn last_status(&self) -> Option<StatusByte> {
        self.last_status
    }

    /// Read the chip part number.
    /// This action _does_ update `last_status`.
    pub async fn read_part_number(&mut self) -> Result<PartNumber, Self::Error> {
        match self.read_reg(ExtReg::PARTNUMBER).await? {
            0x20 => Ok(PartNumber::Cc1200),
            0x21 => Ok(PartNumber::Cc1201),
            _ => Err(DriverError::InvalidPartNumber),
        }
    }

    /// Read a single register value from chip.
    /// This action _does_ update `last_status`.
    pub async fn read_reg<R: Reg>(&mut self, reg: R) -> Result<u8, Self::Error> {
        let mut tx_buffer = [0; OPCODE_MAX + 1];
        let opcode_len = reg.get_read_opcode(false).assign(&mut tx_buffer);
        let tx = &tx_buffer[..opcode_len + 1];

        let mut rx_buffer = [0; OPCODE_MAX + 1];
        let rx = &mut rx_buffer[..tx.len()];

        self.spi.transfer(rx, tx).await.map_err(DriverError::Spi)?;
        self.last_status = Some(StatusByte(rx[0]));

        Ok(rx[rx.len() - 1])
    }

    /// Read a sequence of register values from chip.
    /// This action _does_ update `last_status`.
    pub async fn read_regs<R: Reg>(
        &mut self,
        first: R,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut opcode_tx_buffer = [0; OPCODE_MAX];
        let opcode_len = first
            .get_read_opcode(buffer.len() > 1)
            .assign(&mut opcode_tx_buffer);
        let opcode_tx = &opcode_tx_buffer[..opcode_len];

        let mut opcode_rx_buffer = [0; OPCODE_MAX];
        let opcode_rx = &mut opcode_rx_buffer[..opcode_tx.len()];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(opcode_rx, opcode_tx).await?;
            let status = StatusByte(opcode_rx[0]);
            bus.read(buffer).await?;
            Ok(status)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);

        Ok(())
    }

    /// Write a single register value to chip.
    /// This action _does not_ update `last_status`.
    pub async fn write_reg<R: Reg>(&mut self, reg: R, value: u8) -> Result<(), Self::Error> {
        let mut tx_buffer = [0; OPCODE_MAX + 1];
        let opcode_len = reg.get_write_opcode(false).assign(&mut tx_buffer);
        let tx = &mut tx_buffer[0..opcode_len + 1];
        tx[opcode_len] = value;

        self.spi.write(tx).await.map_err(DriverError::Spi)?;

        self.last_status = None;

        Ok(())
    }

    /// Write a sequence of register values to chip.
    /// This action _does not_ update `last_status`.
    pub async fn write_regs<R: Reg>(&mut self, first: R, values: &[u8]) -> Result<(), Self::Error> {
        let mut opcode_tx_buffer = [0; OPCODE_MAX];
        let opcode_len = first
            .get_write_opcode(values.len() > 1)
            .assign(&mut opcode_tx_buffer);
        let opcode_tx = &opcode_tx_buffer[..opcode_len];

        spi_transaction!(&mut self.spi, |bus| async {
            bus.write(opcode_tx).await?;
            bus.write(values).await?;
            Ok(())
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = None;

        Ok(())
    }

    /// Write a configuration patch to chip.
    /// This action _does not_ update `last_status`.
    pub async fn write_patch<'a, 'patch, R: Reg>(
        &'a mut self,
        patch: ConfigPatch<'patch, R>,
    ) -> Result<(), Self::Error>
    where
        'a: 'patch,
    {
        self.write_regs(patch.first, patch.values).await
    }

    /// Modify register values.
    /// This action _does_ update `last_status`.
    pub async fn modify_regs<R: Reg, F: FnOnce(&mut [u8])>(
        &mut self,
        first: R,
        count: usize,
        configure: F,
    ) -> Result<(), Self::Error> {
        let mut buf = vec![0; count];
        self.read_regs(first, &mut buf).await?;
        configure(&mut buf);
        self.write_regs(first, &buf).await?;
        Ok(())
    }

    /// Read the current RSSI level.
    /// This action _does_ update `last_status`.
    pub async fn read_rssi(&mut self) -> Result<Rssi, Self::Error> {
        let mut tx = [0; 3];
        assert_eq!(2, ExtReg::RSSI1.get_write_opcode(false).assign(&mut tx));

        let mut rx = [0; 3];

        self.spi
            .transfer(&mut rx, &tx)
            .await
            .map_err(DriverError::Spi)?;
        self.last_status = Some(StatusByte(rx[0]));

        self.map_rssi(rx[2])
    }

    /// Read from the RX fifo.
    /// This action _does_ update `last_status`.
    pub async fn read_fifo(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        assert!(buffer.len() <= RX_FIFO_SIZE);

        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::ReadFifoBurst.assign(&mut opcode_tx));

        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &opcode_tx).await?;
            let status = StatusByte(opcode_rx[0]);
            bus.read(buffer).await?;
            Ok(status)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);

        Ok(())
    }

    /// Read the RSSI and RX fifo in one transaction.
    /// This action _does_ update `last_status`.
    pub async fn read_rssi_and_fifo(&mut self, buffer: &mut [u8]) -> Result<Rssi, Self::Error> {
        assert!(buffer.len() <= RX_FIFO_SIZE);

        let mut tx = [0; 3 + 1];
        assert_eq!(
            2,
            Opcode::ReadExtSingle(ExtReg::RSSI1).assign(&mut tx[0..2])
        );
        // RSSI is returned intx[2]
        assert_eq!(1, Opcode::ReadFifoBurst.assign(&mut tx[3..4]));

        let mut rx = [0; 3 + 1];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut rx, &tx).await?;
            let status = StatusByte(rx[0]);
            bus.read(buffer).await?;
            Ok(status)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);

        self.map_rssi(rx[2])
    }

    /// Write to the TX fifo.
    /// This action _does_ update `last_status`.
    pub async fn write_fifo(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        assert!(buffer.len() <= TX_FIFO_SIZE);

        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::WriteFifoBurst.assign(&mut opcode_tx));

        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &opcode_tx).await?;
            let status = StatusByte(opcode_rx[0]);
            bus.write(buffer).await?;
            Ok(status)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);
        Ok(())
    }

    // Map the RSSI1 register field to an rssi value.
    fn map_rssi(&self, rssi1_value: u8) -> Result<Rssi, Self::Error> {
        let rssi = rssi1_value as i8;
        match rssi {
            -128 => Err(DriverError::InvalidRssi),
            rssi => Ok(rssi + self.rssi_offset),
        }
    }

    /// Strobe a command to the chip.
    /// This action _does_ update `last_status`.
    pub async fn strobe(&mut self, strobe: Strobe) -> Result<(), Self::Error> {
        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::Strobe(strobe).assign(&mut opcode_tx));
        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &opcode_tx).await?;

            if strobe == Strobe::SRES {
                // When SRES strobe is issued the CSn pin must be kept low until the SO pin goes low again.
                Self::miso_wait_low(bus).await?;
            }

            Ok(StatusByte(opcode_rx[0]))
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);
        Ok(())
    }

    /// Strobe a command to the chip, and continue to do so until `pred` is satisfied.
    /// This action _does_ update `last_status`.
    pub async fn strobe_until<Pred>(
        &mut self,
        strobe: Strobe,
        pred: Pred,
    ) -> Result<(), Self::Error>
    where
        Pred: Fn(StatusByte) -> bool,
    {
        assert_ne!(Strobe::SRES, strobe);

        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::Strobe(strobe).assign(&mut opcode_tx));
        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            let mut status;
            loop {
                bus.transfer(&mut opcode_rx, &opcode_tx).await?;
                status = StatusByte(opcode_rx[0]);

                if pred(status) {
                    break;
                }
            }
            Ok(status)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);
        Ok(())
    }

    /// Strobe a command to the chip, and continue to do so until the chip enters the IDLE state.
    /// This action _does_ update `last_status`.
    pub async fn strobe_until_idle(&mut self, strobe: Strobe) -> Result<(), Self::Error> {
        self.strobe_until(strobe, |status| status.state() == State::IDLE)
            .await
    }

    /// Wait for the xtal to stabilize.
    async fn wait_for_xtal(spi: &mut SpiBus, delay: &mut Delay) -> Result<bool, SpiBus::Error> {
        let rising_future = Self::miso_wait_low(spi);
        let timeout_future = delay.delay_ms(2_000);
        pin_mut!(rising_future);
        pin_mut!(timeout_future);

        // Wait for any of the two futures to complete.
        match future::select(rising_future, timeout_future).await {
            Either::Left((rising, _)) => {
                // Ensure no spi bus error
                rising?;

                // The xtal is stabilized
                Ok(true)
            }
            Either::Right((timeout, _)) => {
                // Ensure that the timeout result was ok
                timeout.unwrap();

                // We have timeout - the xtal did not stabilize in time
                Ok(false)
            }
        }
    }

    async fn miso_wait_low(bus: &mut SpiBus) -> Result<(), SpiBus::Error> {
        loop {
            let mut buffer = [0u8];
            bus.read(&mut buffer).await?;
            if buffer[0] & 1 == 0 {
                return Ok(());
            }
        }
    }
}

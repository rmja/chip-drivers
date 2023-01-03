use crate::{
    opcode::{Opcode, Strobe, OPCODE_MAX},
    regs::{
        self,
        ext::{self, Freqoff0, Freqoff1},
        Register, RegisterAddress,
    },
    statusbyte::{State, StatusByte},
    ConfigPatch, DriverError, PartNumber, Rssi, RX_FIFO_SIZE, TX_FIFO_SIZE,
};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::{delay, spi, spi_transaction};
use futures::{
    future::{self, Either},
    pin_mut,
};

const DEFAULT_RSSI_OFFSET: i8 = -99; // The default offset defined in the users guide

pub struct Driver<SpiDevice, SpiBus, Delay, ResetPin>
where
    SpiDevice: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: spi::SpiBus,
    Delay: delay::DelayUs,
    ResetPin: OutputPin,
{
    spi: SpiDevice,
    delay: Delay,
    reset_pin: Option<ResetPin>,
    last_status: Option<StatusByte>,
    rssi_offset: Option<Rssi>,
    freq_off: Option<i16>,
}

pub struct CalibrationValue<T> {
    pub desired: T,
    pub actual: T,
}

impl<Spi, SpiBus, Delay, ResetPin> Driver<Spi, SpiBus, Delay, ResetPin>
where
    Spi: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: spi::SpiBus,
    Delay: delay::DelayUs,
    ResetPin: OutputPin,
{
    type Error = DriverError<Spi::Error, Delay>;

    pub fn new(spi: Spi, delay: Delay, reset_pin: Option<ResetPin>) -> Self {
        Self {
            spi,
            delay,
            reset_pin,
            last_status: None,
            rssi_offset: None,
            freq_off: None,
        }
    }

    /// Initialize chip by releasing reset pin.
    pub async fn init(&mut self) -> Result<(), Self::Error> {
        if let Some(reset_pin) = self.reset_pin.as_mut() {
            reset_pin.set_high().unwrap(); // Release chip reset pin.
        }
        future::ready(()).await;
        Ok(())
    }

    /// Send a reset to chip and wait for it to become available.
    /// This action _does_ update `last_status`.
    pub async fn reset(&mut self) -> Result<(), Self::Error> {
        if let Some(reset_pin) = self.reset_pin.as_mut() {
            // Send reset chip sequence
            reset_pin.set_low().unwrap(); // Trigger chip reset pin.
            self.delay.delay_ms(2).await.map_err(DriverError::Delay)?;
            reset_pin.set_high().unwrap(); // Release chip reset pin.

            // The chip reset sequence was sent - wait for chip to become available.

            let delay = &mut self.delay;
            let status = spi_transaction!(&mut self.spi, move |bus| async move {
                // Wait 1ms until the chip has had a chance to set the SO pin high.
                // We must unwrap as the transaction can only return `SpiBus::Error`.
                delay.delay_ms(1).await.unwrap();
                Self::wait_for_xtal(bus, delay).await
            })
            .await
            .map_err(DriverError::Spi)?;
            self.last_status = status;

            if let Some(status) = status && status.chip_rdy() {
                Ok(())
            } else {
                Err(DriverError::Timeout)
            }
        } else {
            let delay = &mut self.delay;
            let status = spi_transaction!(&mut self.spi, move |bus| async move {
                bus.write(&[Opcode::Strobe(Strobe::SRES).as_u8()]).await?;

                // The chip reset sequence was sent - wait for chip to become available.
                // This must happen in the same spi transaction

                Self::wait_for_xtal(bus, delay).await
            })
            .await
            .map_err(DriverError::Spi)?;
            self.last_status = status;

            if let Some(status) = status && status.chip_rdy() {
                Ok(())
            } else {
                Err(DriverError::Timeout)
            }
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
        let partnumber = self.read_reg::<regs::ext::Partnumber>().await?;
        match partnumber.partnum() {
            0x20 => Ok(PartNumber::Cc1200),
            0x21 => Ok(PartNumber::Cc1201),
            _ => Err(DriverError::InvalidPartNumber),
        }
    }

    /// Read a single register value from chip.
    /// This action _does_ update `last_status`.
    pub async fn read_reg<R: Register>(&mut self) -> Result<R, Self::Error> {
        let mut tx_buffer = [0; OPCODE_MAX + 1];
        let opcode_len = Opcode::ReadSingle(R::ADDRESS).assign(&mut tx_buffer);
        let tx = &tx_buffer[..opcode_len + 1];

        let mut rx_buffer = [0; OPCODE_MAX + 1];
        let rx = &mut rx_buffer[..tx.len()];

        self.spi.transfer(rx, tx).await.map_err(DriverError::Spi)?;
        self.last_status = Some(StatusByte(rx[0]));

        Ok(R::from(rx[rx.len() - 1]))
    }

    /// Read a sequence of register values from chip.
    /// This action _does_ update `last_status`.
    pub async fn read_regs(
        &mut self,
        first: RegisterAddress,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut opcode_tx_buffer = [0; OPCODE_MAX];
        let opcode_len = Opcode::read(first, buffer.len() > 1).assign(&mut opcode_tx_buffer);
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
    pub async fn write_reg<R: Register>(&mut self, reg: R) -> Result<(), Self::Error> {
        let mut tx_buffer = [0; OPCODE_MAX + 1];
        let opcode_len = Opcode::WriteSingle(R::ADDRESS).assign(&mut tx_buffer);
        let tx = &mut tx_buffer[0..opcode_len + 1];
        tx[opcode_len] = reg.value();

        self.spi.write(tx).await.map_err(DriverError::Spi)?;

        self.last_status = None;

        Ok(())
    }

    /// Write a sequence of register values to chip.
    /// This action _does not_ update `last_status`.
    pub async fn write_regs(
        &mut self,
        first: RegisterAddress,
        values: &[u8],
    ) -> Result<(), Self::Error> {
        let mut opcode_tx_buffer = [0; OPCODE_MAX];
        let opcode_len = Opcode::write(first, values.len() > 1).assign(&mut opcode_tx_buffer);
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
    pub async fn write_patch<'patch>(
        &mut self,
        patch: ConfigPatch<'patch>,
    ) -> Result<(), Self::Error> {
        let (pri, ext) = patch.split();
        if let Some(pri) = pri {
            self.write_regs(pri.first_address, pri.values).await?;
        }

        if let Some(ext) = ext {
            self.write_regs(ext.first_address, ext.values).await?;

            if self.freq_off.is_some() && ext.get::<Freqoff1>().is_some()
                || ext.get::<Freqoff0>().is_some()
            {
                self.write_freq_off().await?;
            }
        }

        Ok(())
    }

    /// Read the current RSSI level.
    /// This action _does_ update `last_status`.
    pub async fn read_rssi(&mut self) -> Result<Rssi, Self::Error> {
        let mut tx = [0; 3];
        assert_eq!(2, Opcode::ReadSingle(ext::Rssi1::ADDRESS).assign(&mut tx));

        let mut rx = [0; 3];

        self.spi
            .transfer(&mut rx, &tx)
            .await
            .map_err(DriverError::Spi)?;
        self.last_status = Some(StatusByte(rx[0]));

        self.map_rssi(rx[2])
    }

    /// Read from the RX fifo by first reading the length and then read what is available.
    /// This action _does_ update `last_status`.
    pub async fn read_fifo(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let mut opcode_tx: [u8; 4] = [0; 4];
        assert_eq!(
            2,
            Opcode::ReadSingle(ext::NumRxbytes::ADDRESS).assign(&mut opcode_tx)
        );
        opcode_tx[3] = Opcode::ReadFifoBurst.as_u8();
        let mut opcode_rx = [0; 4];

        let (status, length) = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &opcode_tx).await?;
            let status = StatusByte(opcode_rx[0]);
            let available = opcode_rx[2] as usize;
            let length = usize::min(available, buffer.len());
            bus.read(&mut buffer[..length]).await?;
            Ok((status, length))
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);

        Ok(length)
    }

    /// Read from the RX fifo by explicitly reading a pre-known amount corresponding to the size of the buffer.
    /// This action _does_ update `last_status`.
    pub async fn read_fifo_raw(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        assert!(buffer.len() <= RX_FIFO_SIZE);

        const OPCODE_TX: [u8; 1] = [Opcode::ReadFifoBurst.as_u8()];
        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &OPCODE_TX).await?;
            let status = StatusByte(opcode_rx[0]);
            bus.read(buffer).await?;
            Ok(status)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);

        Ok(())
    }

    /// Empty the RX fifo.
    /// This action _does_ update `last_status`.
    pub async fn empty_fifo(&mut self) -> Result<(), Self::Error> {
        let mut opcode_tx: [u8; 4] = [0; 4];
        assert_eq!(
            2,
            Opcode::ReadSingle(ext::NumRxbytes::ADDRESS).assign(&mut opcode_tx)
        );
        opcode_tx[3] = Opcode::ReadFifoBurst.as_u8();
        let mut opcode_rx = [0; 4];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &opcode_tx).await?;
            let status = StatusByte(opcode_rx[0]);
            let mut available = opcode_rx[2] as usize;
            let zeros = &[0; 16];
            while available > zeros.len() {
                bus.write(zeros).await?;
                available -= zeros.len();
            }

            bus.write(&zeros[..available]).await?;

            Ok(status)
        })
        .await
        .map_err(DriverError::Spi)?;
        self.last_status = Some(status);

        Ok(())
    }

    /// Skip bytes in the RX fifo.
    /// This action _does_ update `last_status`.
    pub async fn skip_fifo(&mut self, length: usize) -> Result<(), Self::Error> {
        assert!(length <= RX_FIFO_SIZE);

        const OPCODE_TX: [u8; 1] = [Opcode::ReadFifoBurst.as_u8()];
        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &OPCODE_TX).await?;
            let status = StatusByte(opcode_rx[0]);

            let zeros = &[0; 16];
            let mut length = length;
            while length > zeros.len() {
                bus.write(zeros).await?;
                length -= zeros.len();
            }

            bus.write(&zeros[..length]).await?;

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
            Opcode::ReadSingle(ext::Rssi1::ADDRESS).assign(&mut tx[0..2])
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

        const OPCODE_TX: [u8; 1] = [Opcode::WriteFifoBurst.as_u8()];
        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &OPCODE_TX).await?;
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
            rssi => Ok(rssi + self.rssi_offset.unwrap_or(DEFAULT_RSSI_OFFSET)),
        }
    }

    /// Strobe a command to the chip.
    /// This action _does_ update `last_status`.
    pub async fn strobe(&mut self, strobe: Strobe) -> Result<(), Self::Error> {
        assert_ne!(Strobe::SRES, strobe);

        let opcode_tx = [Opcode::Strobe(strobe).as_u8()];
        let mut opcode_rx = [0];

        let status = spi_transaction!(&mut self.spi, |bus| async {
            bus.transfer(&mut opcode_rx, &opcode_tx).await?;
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

        let opcode_tx = [Opcode::Strobe(strobe).as_u8()];
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
    async fn wait_for_xtal(
        spi: &mut SpiBus,
        delay: &mut Delay,
    ) -> Result<Option<StatusByte>, SpiBus::Error> {
        let ready_future = Self::miso_wait_low(spi);
        let timeout_future = delay.delay_ms(2_000);
        pin_mut!(ready_future);
        pin_mut!(timeout_future);

        // Wait for any of the two futures to complete.
        match future::select(ready_future, timeout_future).await {
            Either::Left((status, _)) => {
                // The xtal is stabilized
                Ok(Some(status?))
            }
            Either::Right((timeout, _)) => {
                // Ensure that the timeout result was ok
                timeout.unwrap();

                // We have timeout - the xtal did not stabilize in time
                Ok(None)
            }
        }
    }

    async fn miso_wait_low(bus: &mut SpiBus) -> Result<StatusByte, SpiBus::Error> {
        const OPCODE_TX: [u8; 1] = [Opcode::Strobe(Strobe::SNOP).as_u8()];
        let mut opcode_rx = [0];

        loop {
            bus.transfer(&mut opcode_rx, &OPCODE_TX).await?;
            let status = StatusByte(opcode_rx[0]);
            if status.chip_rdy() {
                return Ok(status);
            }
        }
    }

    // Set the RSSI calibration
    pub async fn set_rssi_cal(
        &mut self,
        value: Option<CalibrationValue<i8>>,
    ) -> Result<(), Self::Error> {
        self.rssi_offset = value.map(|x| x.desired - x.actual);
        Ok(())
    }

    /// Set the frequency calibration
    ///
    /// # Example
    ///
    /// The desired frequency is 868.950MHz but the measured is 868.850MHz.
    /// Then call with `set_freq_cal(Some(CalibrationValue{ desired: 868950000, actual: 868850000 }))`.
    ///
    /// # Details
    ///
    /// From equation 28 in CC1200 we have
    ///
    /// ![f_VCO](https://latex.codecogs.com/png.latex?\color{White}f_{VCO}=\frac{FREQ}{2^{16}}f_{XOSC}+\frac{FREQOFF}{2^{18}}f_{XOSC})
    ///
    /// and from equation 27:
    ///
    /// ![f_RF](https://latex.codecogs.com/png.latex?\color{White}f_{RF}=\frac{f_{VCO}}{4})
    ///
    /// For the actual value `FREQOFF=0` and so we have:
    ///
    /// ![delta](https://latex.codecogs.com/png.latex?\color{White}f_{RFdesired}-f_{RFactual}=\frac{FREQOFF}{2^{20}}f_{XOSC})
    ///
    /// Solving for `FREQOFF` we have
    ///
    /// ![FREQOFF](https://latex.codecogs.com/png.latex?\color{White}FREQOFF=\frac{f_{RFdesired}-f_{RFactual}}{f_{XOSC}}2^{20}=\frac{f_{RFdesired}-f_{RF_actual}}{10000000}2^{18})
    pub async fn set_freq_cal(
        &mut self,
        value: Option<CalibrationValue<u32>>,
    ) -> Result<(), Self::Error> {
        self.freq_off = value.map(|x| {
            let desired = x.desired as i32;
            let actual = x.actual as i32;
            let delta = actual - desired;
            let freq_off = delta * 2i32.pow(18) / 10_000_000;
            freq_off as i16
        });

        self.write_freq_off().await
    }

    async fn write_freq_off(&mut self) -> Result<(), Self::Error> {
        let values = self.freq_off.unwrap_or_default().to_be_bytes();
        self.write_regs(Freqoff1::ADDRESS, &values).await
    }
}

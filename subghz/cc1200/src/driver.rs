use core::convert::Infallible;

use crate::{
    cmd::{BurstHeader, Response, SingleCommand, Strobe, StrobeCommand},
    regs::{
        self,
        ext::{self, Freqoff0, Freqoff1},
        Register, RegisterAddress,
    },
    statusbyte::{State, StatusByte},
    Config, ConfigPatch, DriverError, PartNumber, Rssi, RX_FIFO_SIZE, TX_FIFO_SIZE,
};
use embedded_hal::{
    digital::{self, OutputPin},
    spi::Operation,
};
use embedded_hal_async::{delay, spi};
use futures::{
    future::{self, Either},
    pin_mut,
};

const DEFAULT_RSSI_OFFSET: i16 = -99; // The default offset defined in the users guide

pub struct Driver<Spi, Delay, ResetPin = NoPin>
where
    Delay: delay::DelayNs,
    ResetPin: OutputPin,
{
    spi: Spi,
    delay: Delay,
    reset_pin: Option<ResetPin>,
    last_status: Option<StatusByte>,
    rssi_offset: Option<Rssi>,
    freq_off: Option<i16>,
}

pub struct NoPin;

impl digital::ErrorType for NoPin {
    type Error = Infallible;
}

impl digital::OutputPin for NoPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct CalibrationValue<T> {
    pub measured: T,
    pub desired: T,
}

impl<T> From<CalibrationValue<T>> for (T, T) {
    fn from(value: CalibrationValue<T>) -> (T, T) {
        (value.measured, value.desired)
    }
}

impl<T> From<(T, T)> for CalibrationValue<T> {
    fn from(value: (T, T)) -> CalibrationValue<T> {
        Self {
            measured: value.0,
            desired: value.1,
        }
    }
}

impl<Spi, Delay, ResetPin> Driver<Spi, Delay, ResetPin>
where
    Spi: spi::SpiDevice,
    Delay: delay::DelayNs,
    ResetPin: OutputPin,
{
    pub fn new(spi: Spi, delay: Delay) -> Self {
        Self {
            spi,
            delay,
            reset_pin: None,
            last_status: None,
            rssi_offset: Some(DEFAULT_RSSI_OFFSET),
            freq_off: None,
        }
    }

    pub fn new_with_reset(spi: Spi, delay: Delay, reset_pin: ResetPin) -> Self {
        Self {
            spi,
            delay,
            reset_pin: Some(reset_pin),
            last_status: None,
            rssi_offset: Some(DEFAULT_RSSI_OFFSET),
            freq_off: None,
        }
    }

    /// Initialize chip by releasing reset pin.
    pub async fn init(&mut self) -> Result<(), DriverError> {
        if let Some(reset_pin) = self.reset_pin.as_mut() {
            reset_pin.set_high().unwrap(); // Release chip reset pin.
        }
        future::ready(()).await;
        Ok(())
    }

    /// Send a reset to chip and wait for it to become available.
    pub async fn reset(&mut self) -> Result<(), DriverError> {
        if let Some(reset_pin) = self.reset_pin.as_mut() {
            // Send reset chip sequence
            reset_pin.set_low().unwrap(); // Trigger chip reset pin.
            self.delay.delay_ms(2).await;
            reset_pin.set_high().unwrap(); // Release chip reset pin.

            // The chip reset sequence was sent - wait for chip to become available.

            let status = Self::wait_for_xtal(&mut self.spi, &mut self.delay).await?;
            self.last_status = status;

            if let Some(status) = status {
                if status.chip_rdy() {
                    Ok(())
                } else {
                    Err(DriverError::Timeout)
                }
            } else {
                Err(DriverError::Timeout)
            }
        } else {
            const CMD: StrobeCommand = StrobeCommand::new(Strobe::SRES);
            self.spi.write(CMD.request.as_ref()).await?;
            let status = Self::wait_for_xtal(&mut self.spi, &mut self.delay).await?;
            self.last_status = status;

            if let Some(status) = status {
                if status.chip_rdy() {
                    Ok(())
                } else {
                    Err(DriverError::Timeout)
                }
            } else {
                Err(DriverError::Timeout)
            }
        }
    }

    /// Get the spi status returned by the last spi operation.
    pub fn last_status(&self) -> Option<StatusByte> {
        self.last_status
    }

    /// Read the chip part number.
    pub async fn read_part_number(&mut self) -> Result<PartNumber, DriverError> {
        let partnumber = self.read_reg::<regs::ext::Partnumber>().await?;
        match partnumber.partnum() {
            0x20 => Ok(PartNumber::Cc1200),
            0x21 => Ok(PartNumber::Cc1201),
            _ => Err(DriverError::InvalidPartNumber),
        }
    }

    /// Read a single register value from chip.
    pub async fn read_reg<R: Register>(&mut self) -> Result<R, DriverError> {
        let mut cmd = SingleCommand::read(R::ADDRESS);

        self.spi
            .transfer(cmd.response.as_mut(), cmd.request.as_ref())
            .await?;

        self.last_status = Some(cmd.response.status_byte());
        Ok(R::from(cmd.response.value()))
    }

    /// Read a sequence of register values from chip.
    pub async fn read_regs(
        &mut self,
        first: RegisterAddress,
        buffer: &mut [u8],
    ) -> Result<(), DriverError> {
        let mut header = BurstHeader::read(first);

        self.spi
            .transaction(&mut [
                Operation::Transfer(header.response.as_mut(), header.request.as_ref()),
                Operation::Read(buffer),
            ])
            .await?;

        self.last_status = Some(header.response.status_byte());
        Ok(())
    }

    /// Write a single register value to chip.
    pub async fn write_reg<R: Register>(&mut self, reg: R) -> Result<(), DriverError> {
        let mut cmd = SingleCommand::write(R::ADDRESS, reg.value());

        self.spi
            .transfer(cmd.response.as_mut(), cmd.request.as_ref())
            .await?;

        self.last_status = Some(cmd.response.status_byte());
        Ok(())
    }

    /// Write a sequence of register values to chip.
    pub async fn write_regs(
        &mut self,
        first: RegisterAddress,
        values: &[u8],
    ) -> Result<(), DriverError> {
        let mut header = BurstHeader::write(first);

        self.spi
            .transaction(&mut [
                Operation::Transfer(header.response.as_mut(), header.request.as_ref()),
                Operation::Write(values),
            ])
            .await?;

        self.last_status = Some(header.response.status_byte());
        Ok(())
    }

    /// Write a configuration patch to chip.
    pub async fn write_patch<'patch>(
        &mut self,
        patch: ConfigPatch<'patch>,
    ) -> Result<(), DriverError> {
        let (pri, ext) = patch.split_pri_ext();
        if !pri.is_empty() {
            self.write_regs(pri.first_address, pri.values).await?;
        }

        if !ext.is_empty() {
            self.write_regs(ext.first_address, ext.values).await?;

            if self.freq_off.is_some()
                && (ext.get::<Freqoff1>().is_some() || ext.get::<Freqoff0>().is_some())
            {
                self.write_freq_off().await?;
            }
        }

        Ok(())
    }

    /// Read entire configuration from chip.
    pub async fn read_config(&mut self) -> Result<Config, DriverError> {
        let mut config = Config([0; 105]);
        let pri_len = RegisterAddress::PRI_MAX.0 - RegisterAddress::PRI_MIN.0 + 1;
        let (pri_buf, ext_buf) = config.0.split_at_mut(pri_len as usize);
        self.read_regs(RegisterAddress::PRI_MIN, pri_buf).await?;
        self.read_regs(RegisterAddress::EXT_MIN, ext_buf).await?;
        Ok(config)
    }

    /// Read the current RSSI level.
    pub async fn read_rssi(&mut self) -> Result<Option<Rssi>, DriverError> {
        let rssi = self.read_reg::<ext::Rssi1>().await?.rssi_11_4();
        Ok(self.map_rssi(rssi))
    }

    /// Read from the RX fifo by first reading the length and then read what is available.
    pub async fn read_fifo(&mut self, buffer: &mut [u8]) -> Result<usize, DriverError> {
        let available = self.read_reg::<ext::NumRxbytes>().await?.rxbytes() as usize;
        let len = core::cmp::min(core::cmp::min(available, buffer.len()), RX_FIFO_SIZE);
        unsafe { self.read_fifo_raw(&mut buffer[..len]).await? }
        Ok(len)
    }

    /// Read from the RX fifo by explicitly reading a pre-known amount corresponding to a known number of items in the buffer.
    pub async unsafe fn read_fifo_raw(&mut self, buffer: &mut [u8]) -> Result<(), DriverError> {
        assert!(buffer.len() <= RX_FIFO_SIZE);

        let mut header = BurstHeader::read_fifo();

        self.spi
            .transaction(&mut [
                Operation::Transfer(&mut header.response.as_mut(), header.request.as_ref()),
                Operation::Read(buffer),
            ])
            .await?;

        self.last_status = Some(header.response.status_byte());
        Ok(())
    }

    /// Read from the RX fifo by explicitly reading a pre-known amount corresponding to a known number of items in the buffer.
    pub async unsafe fn read_rssi_and_fifo_raw(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<Option<Rssi>, DriverError> {
        let len = buffer.len();
        assert!(len <= RX_FIFO_SIZE);

        let mut tx_buf: [u8; 4 + RX_FIFO_SIZE] = [0; 4 + RX_FIFO_SIZE];
        let mut rx_buf = [0; 4 + RX_FIFO_SIZE];

        tx_buf[0..3].copy_from_slice(SingleCommand::read(ext::Rssi1::ADDRESS).request.as_ref());
        tx_buf[3..4].copy_from_slice(BurstHeader::read_fifo().request.as_ref());

        let tx = &tx_buf[..4 + len];
        let rx = &mut rx_buf[..4 + len];

        self.spi.transfer(rx, tx).await?;

        // The status byte is emitted twice by the chip as we send two opcodes in the same transfer
        self.last_status = Some(StatusByte(rx[3]));
        buffer.copy_from_slice(&rx[4..]);
        Ok(self.map_rssi(rx[2]))
    }

    /// Empty the RX fifo.
    pub async fn drain_fifo(&mut self) -> Result<usize, DriverError> {
        let mut available = self.read_reg::<ext::NumRxbytes>().await?.rxbytes() as usize;
        let discarded = available;
        if available > 0 {
            let mut tx_buf = [0; 1 + 16];
            let mut rx_buf = [0; 1 + 16];
            tx_buf[0..1].copy_from_slice(BurstHeader::read_fifo().request.as_ref());
            while available > 16 {
                self.spi.transfer(&mut rx_buf, &tx_buf).await?;
                available -= 16;
            }

            if available > 0 {
                self.spi
                    .transfer(&mut rx_buf[..1 + available], &tx_buf[..1 + available])
                    .await?;
            }

            self.last_status = Some(StatusByte(rx_buf[0]));
        }
        Ok(discarded)
    }

    /// Write to the TX fifo.
    pub async fn write_fifo(&mut self, buffer: &[u8]) -> Result<(), DriverError> {
        assert!(buffer.len() <= TX_FIFO_SIZE);

        let mut header = BurstHeader::write_fifo();

        self.spi
            .transaction(&mut [
                Operation::Transfer(header.response.as_mut(), header.request.as_ref()),
                Operation::Write(buffer),
            ])
            .await?;

        self.last_status = Some(header.response.status_byte());
        Ok(())
    }

    // Map the RSSI1 register field to an rssi value.
    fn map_rssi(&self, rssi1_value: u8) -> Option<Rssi> {
        let rssi = rssi1_value as i8;
        match rssi {
            -128 => None,
            rssi => Some(rssi as i16 + self.rssi_offset.unwrap_or_default()),
        }
    }

    /// Strobe a command to the chip.
    pub async fn strobe(&mut self, strobe: Strobe) -> Result<(), DriverError> {
        assert_ne!(Strobe::SRES, strobe);

        let mut cmd = StrobeCommand::new(strobe);

        self.spi
            .transfer(cmd.response.as_mut(), cmd.request.as_ref())
            .await?;

        self.last_status = Some(cmd.response.status_byte());
        Ok(())
    }

    /// Strobe a command to the chip, and continue to do so until `pred` is satisfied.
    pub async fn strobe_until<Pred>(
        &mut self,
        strobe: Strobe,
        pred: Pred,
    ) -> Result<(), DriverError>
    where
        Pred: Fn(StatusByte) -> bool,
    {
        assert_ne!(Strobe::SRES, strobe);

        let mut cmd = StrobeCommand::new(strobe);

        loop {
            self.spi
                .transfer(cmd.response.as_mut(), cmd.request.as_ref())
                .await?;
            let status = cmd.response.status_byte();
            if pred(status) {
                self.last_status = Some(status);
                return Ok(());
            }
        }
    }

    /// Strobe a command to the chip, and continue to do so until the chip enters the IDLE state.
    pub async fn strobe_until_idle(&mut self, strobe: Strobe) -> Result<(), DriverError> {
        self.strobe_until(strobe, |status| status.state() == State::IDLE)
            .await
    }

    /// Wait for the xtal to stabilize.
    async fn wait_for_xtal(
        spi: &mut Spi,
        delay: &mut Delay,
    ) -> Result<Option<StatusByte>, Spi::Error> {
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
            Either::Right(_) => {
                // We have timeout - the xtal did not stabilize in time
                Ok(None)
            }
        }
    }

    async fn miso_wait_low(spi: &mut Spi) -> Result<StatusByte, Spi::Error> {
        let mut cmd = StrobeCommand::new(Strobe::SNOP);

        loop {
            spi.transfer(cmd.response.as_mut(), cmd.request.as_ref())
                .await?;
            let status = cmd.response.status_byte();
            if status.chip_rdy() {
                return Ok(status);
            }
        }
    }

    // Set the RSSI calibration
    pub async fn set_rssi_cal(
        &mut self,
        value: Option<CalibrationValue<i8>>,
    ) -> Result<(), DriverError> {
        self.rssi_offset = value.map(|x| x.desired - x.measured).map(|x| x as i16);
        Ok(())
    }

    /// Set the frequency calibration
    ///
    /// # Example
    ///
    /// The desired frequency is 868.950MHz but the measured is 868.850MHz.
    /// Then call with `set_freq_cal(Some(CalibrationValue{ measured: 868850000, desired: 868950000 }))`.
    ///
    /// # Details
    ///
    /// From equation 28 in CC1200 we have
    ///
    /// ![f_VCO](https://latex.codecogs.com/png.latex?\color{White}f_{VCO}=\frac{FREQ}{2^{16}}f_{XOSC}+\frac{FREQOFF}{2^{18}}f_{XOSC})
    ///
    /// and from equation 27:
    ///
    /// ![f_RF](https://latex.codecogs.com/png.latex?\color{White}f_{RF}=\frac{f_{VCO}}{LO_{div}})
    ///
    /// For the measured value `FREQOFF=0` and so we have:
    ///
    /// ![delta](https://latex.codecogs.com/png.latex?\color{White}f_{RFdesired}-f_{RFactual}=\frac{FREQOFF}{2^{18}LO_{div}}f_{XOSC})
    ///
    /// Solving for `FREQOFF` we have
    ///
    /// ![FREQOFF](https://latex.codecogs.com/png.latex?\color{White}FREQOFF=\frac{f_{RFdesired}-f_{RFactual}}{f_{XOSC}}2^{18}LO_{div}=\frac{f_{RFdesired}-f_{RF_actual}}{40000000}2^{18}LO_{div})
    pub async fn set_frequency_cal(
        &mut self,
        value: Option<CalibrationValue<u32>>,
    ) -> Result<(), DriverError> {
        self.freq_off = value.map(|x| {
            let lo_div = lo_divider(x.desired) as i32;
            let measured = x.measured as i32;
            let desired = x.desired as i32;
            let delta = measured - desired;
            let freq_off = (delta * lo_div * 2i32.pow(18)) / 40_000_000;
            freq_off as i16
        });

        self.write_freq_off().await
    }

    async fn write_freq_off(&mut self) -> Result<(), DriverError> {
        let values = self.freq_off.unwrap_or_default().to_be_bytes();
        self.write_regs(Freqoff1::ADDRESS, &values).await
    }
}

pub(crate) fn lo_divider(frequency: u32) -> u8 {
    match frequency {
        820_000_000..=960_000_000 => 4,
        410_000_000..=480_000_000 => 8,
        273_300_000..=320_000_000 => 12,
        205_000_000..=240_000_000 => 16,
        164_000_000..=192_000_000 => 20,
        136_700_000..=160_000_000 => 24,
        _ => panic!("Invalid frequency select"),
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_async_mocks::{delay::MockDelay, spi::MockSpiDevice};

    use crate::regs::{ext::FreqoffCfg, pri::Iocfg2};

    use super::*;

    macro_rules! singleton {
        ($type:ty, $val:expr) => {{
            static STATIC_CELL: static_cell::StaticCell<$type> = static_cell::StaticCell::new();
            STATIC_CELL.init($val)
        }};
    }

    #[tokio::test]
    async fn read_reg_primary() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 2], [0x22, 0x33]),
                &[0x80 | 0x01, 0x00]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let value = driver.read_reg::<Iocfg2>().await.unwrap();

        // Then
        assert_eq!(0x22, driver.last_status.unwrap().0);
        assert_eq!(0x33, value.0);
    }

    #[tokio::test]
    async fn read_reg_extended() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 3], [0x22, 0x00, 0x33]),
                &[0x80 | 0x2F, 0x01, 0x00]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let value = driver.read_reg::<FreqoffCfg>().await.unwrap();

        // Then
        assert_eq!(0x22, driver.last_status.unwrap().0);
        assert_eq!(0x33, value.0);
    }

    #[tokio::test]
    async fn read_regs_primary() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 2],
            [
                Operation::Transfer(singleton!([u8; 1], [0x22]), &[0xC0 | 0x01]),
                Operation::Read(singleton!([u8; 2], [0x33, 0x44]))
            ]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let mut buf = [0; 2];
        driver.read_regs(Iocfg2::ADDRESS, &mut buf).await.unwrap();

        // Then
        assert_eq!(0x22, driver.last_status.unwrap().0);
        assert_eq!([0x33, 0x44].as_ref(), buf);
    }

    #[tokio::test]
    async fn read_regs_extended() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 2],
            [
                Operation::Transfer(singleton!([u8; 2], [0x22, 0x00]), &[0xC0 | 0x2F, 0x01]),
                Operation::Read(singleton!([u8; 2], [0x33, 0x44]))
            ]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let mut buf = [0; 2];
        driver
            .read_regs(FreqoffCfg::ADDRESS, &mut buf)
            .await
            .unwrap();

        // Then
        assert_eq!(0x22, driver.last_status.unwrap().0);
        assert_eq!([0x33, 0x44].as_ref(), buf);
    }

    #[tokio::test]
    async fn read_fifo_raw() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 2],
            [
                Operation::Transfer(singleton!([u8; 1], [0x22]), &[0xC0 | 0x3F]),
                Operation::Read(singleton!([u8; 2], [0x33, 0x44]))
            ]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let mut buf = [0; 2];
        unsafe { driver.read_fifo_raw(&mut buf).await.unwrap() };

        // Then
        assert_eq!(0x22, driver.last_status.unwrap().0);
        assert_eq!([0x33, 0x44].as_ref(), buf);
    }

    #[tokio::test]
    async fn read_rssi_and_fifo_raw() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 6], [0x00, 0x00, 0x11, 0x22, 0x33, 0x44]),
                &[0x80 | 0x2F, 0x71, 0x00, 0xC0 | 0x3F, 0x00, 0x00]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let mut buf = [0; 2];
        let rssi = unsafe {
            driver
                .read_rssi_and_fifo_raw(&mut buf)
                .await
                .unwrap()
                .unwrap()
        };

        // Then
        assert_eq!(0x11 - 99, rssi);
        assert_eq!(0x22, driver.last_status.unwrap().0);
        assert_eq!([0x33, 0x44].as_ref(), buf);
    }

    #[tokio::test]
    async fn drain_fifo_0() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 3], [0x22, 0x00, 0]),
                &[0x80 | 0x2F, 0xD7, 0x00]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let discarded = driver.drain_fifo().await.unwrap();

        // Then
        assert_eq!(0, discarded);
        assert_eq!(0x22, driver.last_status.unwrap().0);
    }

    #[tokio::test]
    async fn drain_fifo_1() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 3], [0x00, 0x00, 1]),
                &[0x80 | 0x2F, 0xD7, 0x00]
            )]
        ));

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 2], [0x22, 0x00]),
                &[0xC0 | 0x3F, 0x00]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let discarded = driver.drain_fifo().await.unwrap();

        // Then
        assert_eq!(1, discarded);
        assert_eq!(0x22, driver.last_status.unwrap().0);
    }

    #[tokio::test]
    async fn drain_fifo_16() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 3], [0x00, 0x00, 16]),
                &[0x80 | 0x2F, 0xD7, 0x00]
            )]
        ));

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!(
                    [u8; 17],
                    [
                        0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00
                    ]
                ),
                &[
                    0xC0 | 0x3F,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00
                ]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let discarded = driver.drain_fifo().await.unwrap();

        // Then
        assert_eq!(16, discarded);
        assert_eq!(0x22, driver.last_status.unwrap().0);
    }

    #[tokio::test]
    async fn drain_fifo_17() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 3], [0x00, 0x00, 17]),
                &[0x80 | 0x2F, 0xD7, 0x00]
            )]
        ));

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!(
                    [u8; 17],
                    [
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00
                    ]
                ),
                &[
                    0xC0 | 0x3F,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00
                ]
            )]
        ));

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 2], [0x22, 0x00]),
                &[0xC0 | 0x3F, 0x00]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        let discarded = driver.drain_fifo().await.unwrap();

        // Then
        assert_eq!(17, discarded);
        assert_eq!(0x22, driver.last_status.unwrap().0);
    }

    #[tokio::test]
    async fn write_fifo() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 2],
            [
                Operation::Transfer(singleton!([u8; 1], [0x22]), &[0x40 | 0x3F]),
                Operation::Write(singleton!([u8; 2], [0x33, 0x44]))
            ]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        driver.write_fifo(&[0x33, 0x44]).await.unwrap();

        // Then
        assert_eq!(0x22, driver.last_status.unwrap().0);
    }

    #[tokio::test]
    async fn strobe() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(singleton!([u8; 1], [0x22]), &[0x3D])]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        driver.strobe(Strobe::SNOP).await.unwrap();

        // Then
        assert_eq!(0x22, driver.last_status.unwrap().0);
    }

    #[tokio::test]
    async fn strobe_until_idle() {
        // Given
        let mut spi = MockSpiDevice::new();
        let delay = MockDelay::new();

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 1], [0x10]), // RX
                &[0x3D]
            )]
        ));

        spi.expect_transaction_operations(singleton!(
            [Operation<u8>; 1],
            [Operation::Transfer(
                singleton!([u8; 1], [0x00]), // IDLE
                &[0x3D]
            )]
        ));

        // When
        let mut driver: Driver<_, _> = Driver::new(spi, delay);
        driver.strobe_until_idle(Strobe::SNOP).await.unwrap();

        // Then
        assert_eq!(0x00, driver.last_status.unwrap().0);
    }
}

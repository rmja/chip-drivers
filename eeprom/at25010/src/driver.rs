use crate::{opcode::Opcode, DriverError, PartNumber};
use bitfield::bitfield;
use embedded_hal_async::{delay, spi, spi_transaction};

const PAGE_SIZE: usize = 8;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct StatusRegister(u8);
    /// Reserved for future use
    reserved, _: 7, 4;
    /// Block write protection
    pub bp, _: 3, 2;
    /// Write enable latch
    pub wel, _: 1;
    /// Ready/busy status
    pub bsy, _: 0;
}

const INITIAL_TIMEOUT_MS: u32 = 3; // Wait at least 3 ms
const RETRY_INTERVAL_US: u32 = 100;

pub struct Driver<SpiDevice, SpiBus, Delay>
where
    SpiDevice: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: spi::SpiBus,
    Delay: delay::DelayUs,
{
    spi: SpiDevice,
    delay: Delay,
    part_number: PartNumber,
}

pub struct StatefulDriver<SpiDevice, SpiBus, Delay>
where
    SpiDevice: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: spi::SpiBus,
    Delay: delay::DelayUs,
{
    pub driver: Driver<SpiDevice, SpiBus, Delay>,
    pub(crate) position: u16,
}

impl<SpiDevice, SpiBus, Delay> Driver<SpiDevice, SpiBus, Delay>
where
    SpiDevice: spi::SpiDevice<Bus = SpiBus>,
    SpiBus: spi::SpiBus,
    Delay: delay::DelayUs,
{
    pub const fn new(spi: SpiDevice, delay: Delay, part_number: PartNumber) -> Self {
        Self {
            part_number,
            spi,
            delay,
        }
    }

    pub const fn to_stateful(self) -> StatefulDriver<SpiDevice, SpiBus, Delay> {
        StatefulDriver {
            driver: self,
            position: 0,
        }
    }

    /// Get the EEPROM capacity in bytes
    pub const fn capacity(&self) -> u16 {
        match self.part_number {
            PartNumber::At25010 => 128,
            PartNumber::At25020 => 256,
            PartNumber::At25040 => 512,
            PartNumber::At25010b => 128,
            PartNumber::At25020b => 256,
            PartNumber::At25040b => 512,
        }
    }

    /// Read a sequence of bytes from the EEPROM.
    pub async fn read(&mut self, origin: u16, buffer: &mut [u8]) -> Result<(), DriverError> {
        if origin as usize + buffer.len() > self.capacity() as usize {
            return Err(DriverError::Capacity);
        }

        spi_transaction!(&mut self.spi, |bus| async {
            let opcode = [Opcode::READ(origin).as_u8(), (origin & 0xFF) as u8];
            bus.write(&opcode).await?;
            bus.read(buffer).await?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    /// Write a sequence of bytes to the EEPROM.
    pub async fn write(&mut self, origin: u16, buffer: &[u8]) -> Result<(), DriverError> {
        if origin as usize + buffer.len() > self.capacity() as usize {
            return Err(DriverError::Capacity);
        }

        let t_cs_us = (min_tcs_ns(self.part_number) + 999) / 1000;

        // Wait for a possible previous write to complete.
        self.flush().await?;

        // Disable write protection.
        self.enable_write().await?;

        // Wait until we can send a new spi command.
        self.delay
            .delay_us(t_cs_us)
            .await
            .map_err(|_| DriverError::Delay)?;

        // See if write was enabled (it may have been disabled by the WP pin).
        let sr = self.read_status_register().await?;
        if !sr.wel() {
            return Err(DriverError::WriteProtection);
        }

        let mut address = origin;
        let mut flushed_and_write_enabled = true;
        let offset_in_first_page = origin as usize % PAGE_SIZE;
        let (incomplete_first_page, remaining_pages) =
            buffer.split_at((PAGE_SIZE - offset_in_first_page) % PAGE_SIZE);

        assert!(incomplete_first_page.len() < 8);
        if !incomplete_first_page.is_empty() {
            // Wait until we can send a new spi command.
            self.delay
                .delay_us(t_cs_us)
                .await
                .map_err(|_| DriverError::Delay)?;

            self.write_page(address, incomplete_first_page).await?;
            address += incomplete_first_page.len() as u16;

            // Write is auto-disabled after sending a WRITE command.
            flushed_and_write_enabled = false;
        }

        for page in remaining_pages.chunks(PAGE_SIZE) {
            if !flushed_and_write_enabled {
                self.flush().await?;
                self.enable_write().await?;
            }

            // Wait until we can send a new spi command.
            self.delay
                .delay_us(t_cs_us)
                .await
                .map_err(|_| DriverError::Delay)?;

            self.write_page(address, page).await?;
            address += page.len() as u16;

            // Write is auto-disabled after sending a WRITE command.
            flushed_and_write_enabled = false;
        }

        assert_eq!(origin + buffer.len() as u16, address);

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), DriverError> {
        let sr = self.read_status_register().await?;
        if !sr.bsy() {
            return Ok(());
        }

        // Wait for idle.
        self.delay
            .delay_ms(INITIAL_TIMEOUT_MS)
            .await
            .map_err(|_| DriverError::Delay)?;
        let sr = self.read_status_register().await?;
        if sr.bsy() {
            loop {
                self.delay
                    .delay_us(RETRY_INTERVAL_US)
                    .await
                    .map_err(|_| DriverError::Delay)?;

                let sr = self.read_status_register().await?;
                if !sr.bsy() {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn enable_write(&mut self) -> Result<(), DriverError> {
        const TX: [u8; 1] = [Opcode::WREN.as_u8()];
        self.spi.write(&TX).await?;
        Ok(())
    }

    async fn read_status_register(&mut self) -> Result<StatusRegister, DriverError> {
        const TX: [u8; 2] = [Opcode::RDSR.as_u8(), 0x00];
        let mut rx: [u8; 2] = [0x00, 0x00];
        self.spi.transfer(&mut rx, &TX).await?;
        Ok(StatusRegister(rx[1]))
    }

    async fn write_page(&mut self, address: u16, buffer: &[u8]) -> Result<(), DriverError> {
        let len = buffer.len();
        assert!(len > 0);
        assert!(len <= PAGE_SIZE - (address as usize % PAGE_SIZE));

        spi_transaction!(&mut self.spi, |bus| async {
            bus.write(&[Opcode::WRITE(address).as_u8(), (address & 0xFF) as u8])
                .await?;
            bus.write(buffer).await?;
            Ok(())
        })
        .await?;

        Ok(())
    }
}

/// Get the minimum t_cs time in ns, i.e. the minimum time the CS pin must be de-asserted betweeen commands.
const fn min_tcs_ns(kind: PartNumber) -> u32 {
    match kind {
        PartNumber::At25010 => 250,
        PartNumber::At25020 => 250,
        PartNumber::At25040 => 250,
        PartNumber::At25010b => 100,
        PartNumber::At25020b => 100,
        PartNumber::At25040b => 100,
    }
}

#[cfg(test)]
mod tests {
    use mockall::Sequence;

    use embedded_hal_async_mocks::{delay::MockDelay, spi::MockSpiDevice};

    use super::*;

    #[tokio::test]
    async fn write_starting_at_page_boundary() {
        // Given
        let mut seq = Sequence::new();
        let mut spi = MockSpiDevice::new();
        let mut expected_transactions = 0;

        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));
        expected_transactions += 1;

        expect_write_wren(&mut spi, &mut seq);
        expected_transactions += 1;

        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x02));
        expected_transactions += 1;

        expect_write_page(
            &mut spi,
            &mut seq,
            0x08,
            &[0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80],
        );
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));
        expected_transactions += 2;

        expect_write_wren(&mut spi, &mut seq);
        expect_write_page(&mut spi, &mut seq, 0x10, &[0x90]);
        // expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));
        expected_transactions += 2;

        let mut delay = MockDelay::new();
        delay.expect_delay_us().withf(|_| true).return_const(Ok(()));
        delay.expect_delay_ms().withf(|_| true).return_const(Ok(()));

        // When
        let mut driver = Driver::new(spi, delay, PartNumber::At25010b);

        driver
            .write(
                0x08,
                &[0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90],
            )
            .await
            .unwrap();

        // Then
        assert_eq!(expected_transactions, driver.spi.transactions());
    }

    #[tokio::test]
    async fn write_starting_inside_page() {
        // Given
        let mut seq = Sequence::new();
        let mut spi = MockSpiDevice::new();
        let mut expected_transactions = 0;

        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));
        expected_transactions += 1;

        expect_write_wren(&mut spi, &mut seq);
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x02));
        expected_transactions += 2;

        expect_write_page(&mut spi, &mut seq, 0x0F, &[0x10]);
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));
        expected_transactions += 2;

        expect_write_wren(&mut spi, &mut seq);
        expect_write_page(
            &mut spi,
            &mut seq,
            0x10,
            &[0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90],
        );
        // expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));
        expected_transactions += 2;

        let mut delay = MockDelay::new();
        delay.expect_delay_us().withf(|_| true).return_const(Ok(()));
        delay.expect_delay_ms().withf(|_| true).return_const(Ok(()));

        // When
        let mut driver = Driver::new(spi, delay, PartNumber::At25010b);

        driver
            .write(
                0x0F,
                &[0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90],
            )
            .await
            .unwrap();

        // Then
        assert_eq!(expected_transactions, driver.spi.transactions());
    }

    fn expect_write_wren(spi: &mut MockSpiDevice, seq: &mut Sequence) {
        spi.bus
            .expect_write()
            .withf(|tx| tx == &[Opcode::WREN.as_u8()])
            .times(1)
            .in_sequence(seq)
            .return_const(Ok(()));
    }

    fn expect_read_status_register(
        spi: &mut MockSpiDevice,
        seq: &mut Sequence,
        returning: StatusRegister,
    ) {
        spi.bus
            .expect_transfer()
            .withf(|_, tx| tx == &[Opcode::RDSR.as_u8(), 0x00])
            .times(1)
            .in_sequence(seq)
            .returning(move |rx, _tx| {
                rx[1] = returning.0;
                Ok(())
            });
    }

    fn expect_write_page(
        spi: &mut MockSpiDevice,
        seq: &mut Sequence,
        address: u16,
        expected: &'static [u8],
    ) {
        spi.bus
            .expect_write()
            .withf(move |tx| tx == &[Opcode::WRITE(address).as_u8(), (address & 0xFF) as u8])
            .times(1)
            .in_sequence(seq)
            .return_const(Ok(()));
        spi.bus
            .expect_write()
            .withf(move |tx| tx == expected)
            .times(1)
            .in_sequence(seq)
            .return_const(Ok(()));
    }
}

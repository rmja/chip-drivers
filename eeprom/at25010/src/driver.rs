use crate::{opcode::Opcode, Error, PartNumber};
use bitfield::bitfield;
use embedded_hal_async::{delay, spi};
use embedded_storage::nor_flash::ErrorType;
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};

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

pub struct Driver<SpiDevice, Delay>
where
    SpiDevice: spi::SpiDevice,
    Delay: delay::DelayNs,
{
    spi: SpiDevice,
    delay: Delay,
    part_number: PartNumber,
}

pub struct StatefulDriver<SpiDevice, Delay>
where
    SpiDevice: spi::SpiDevice,
    Delay: delay::DelayNs,
{
    pub driver: Driver<SpiDevice, Delay>,
    pub position: u16,
}

impl<SpiDevice, Delay> Driver<SpiDevice, Delay>
where
    SpiDevice: spi::SpiDevice,
    Delay: delay::DelayNs,
{
    pub const fn new(spi: SpiDevice, delay: Delay, part_number: PartNumber) -> Self {
        Self {
            part_number,
            spi,
            delay,
        }
    }

    pub const fn to_stateful(self) -> StatefulDriver<SpiDevice, Delay> {
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
    pub async fn read(&mut self, origin: u16, buffer: &mut [u8]) -> Result<(), Error> {
        if origin as usize + buffer.len() > self.capacity() as usize {
            return Err(Error::OutOfBounds);
        }

        self.spi
            .transaction(&mut [
                spi::Operation::Write(&[Opcode::READ(origin).as_u8(), (origin & 0xFF) as u8]),
                spi::Operation::Read(buffer),
            ])
            .await?;

        Ok(())
    }

    /// Write a sequence of bytes to the EEPROM.
    pub async fn write(&mut self, origin: u16, buffer: &[u8]) -> Result<(), Error> {
        if origin as usize + buffer.len() > self.capacity() as usize {
            return Err(Error::OutOfBounds);
        }

        let t_cs_us = (min_tcs_ns(self.part_number) + 999) / 1000;

        // Wait for a possible previous write to complete.
        self.flush().await?;

        // Disable write protection.
        self.enable_write().await?;

        // Wait until we can send a new spi command.
        self.delay.delay_us(t_cs_us).await;

        // See if write was enabled (it may have been disabled by the WP pin).
        let sr = self.read_status_register().await?;
        if !sr.wel() {
            return Err(Error::WriteProtection);
        }

        let mut address = origin;
        let mut flushed_and_write_enabled = true;
        let offset_in_first_page = origin as usize % PAGE_SIZE;
        let (incomplete_first_page, remaining_pages) =
            buffer.split_at((PAGE_SIZE - offset_in_first_page) % PAGE_SIZE);

        assert!(incomplete_first_page.len() < 8);
        if !incomplete_first_page.is_empty() {
            // Wait until we can send a new spi command.
            self.delay.delay_us(t_cs_us).await;

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
            self.delay.delay_us(t_cs_us).await;

            self.write_page(address, page).await?;
            address += page.len() as u16;

            // Write is auto-disabled after sending a WRITE command.
            flushed_and_write_enabled = false;
        }

        assert_eq!(origin + buffer.len() as u16, address);

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        let sr = self.read_status_register().await?;
        if !sr.bsy() {
            return Ok(());
        }

        // Wait for idle.
        self.delay.delay_ms(INITIAL_TIMEOUT_MS).await;
        let sr = self.read_status_register().await?;
        if sr.bsy() {
            loop {
                self.delay.delay_us(RETRY_INTERVAL_US).await;

                let sr = self.read_status_register().await?;
                if !sr.bsy() {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn enable_write(&mut self) -> Result<(), Error> {
        const TX: [u8; 1] = [Opcode::WREN.as_u8()];
        self.spi.write(&TX).await?;
        Ok(())
    }

    async fn read_status_register(&mut self) -> Result<StatusRegister, Error> {
        const TX: [u8; 2] = [Opcode::RDSR.as_u8(), 0x00];
        let mut rx: [u8; 2] = [0x00, 0x00];
        self.spi.transfer(&mut rx, &TX).await?;
        Ok(StatusRegister(rx[1]))
    }

    async fn write_page(&mut self, address: u16, buffer: &[u8]) -> Result<(), Error> {
        let len = buffer.len();
        assert!(len > 0);
        assert!(len <= PAGE_SIZE - (address as usize % PAGE_SIZE));

        self.spi
            .transaction(&mut [
                spi::Operation::Write(&[Opcode::WRITE(address).as_u8(), (address & 0xFF) as u8]),
                spi::Operation::Write(buffer),
            ])
            .await?;

        Ok(())
    }
}

impl<SpiDevice, Delay> ErrorType for Driver<SpiDevice, Delay>
where
    SpiDevice: spi::SpiDevice,
    Delay: delay::DelayNs,
{
    type Error = Error;
}

impl<SpiDevice, Delay> ReadNorFlash for Driver<SpiDevice, Delay>
where
    SpiDevice: spi::SpiDevice,
    Delay: delay::DelayNs,
{
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset as u16, bytes).await
    }

    fn capacity(&self) -> usize {
        self.capacity() as usize
    }
}

impl<SpiDevice, Delay> NorFlash for Driver<SpiDevice, Delay>
where
    SpiDevice: spi::SpiDevice,
    Delay: delay::DelayNs,
{
    const WRITE_SIZE: usize = PAGE_SIZE;
    const ERASE_SIZE: usize = PAGE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset as u16, bytes).await
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if from % PAGE_SIZE as u32 != 0 || to % PAGE_SIZE as u32 != 0 {
            return Err(Error::NotAligned);
        }

        let mut origin = from as u16;
        while (origin as u32) < to {
            self.write(origin, &[0xff; PAGE_SIZE]).await?;
            origin += PAGE_SIZE as u16;
        }

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

        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));

        expect_write_wren(&mut spi, &mut seq);

        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x02));

        expect_write_page(
            &mut spi,
            &mut seq,
            0x08,
            &[0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80],
        );
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));

        expect_write_wren(&mut spi, &mut seq);
        expect_write_page(&mut spi, &mut seq, 0x10, &[0x90]);

        let mut delay = MockDelay::new();
        delay.expect_delay_us().withf(|_| true).return_const(());
        delay.expect_delay_ms().withf(|_| true).return_const(());

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
    }

    #[tokio::test]
    async fn write_starting_inside_page() {
        // Given
        let mut seq = Sequence::new();
        let mut spi = MockSpiDevice::new();

        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));

        expect_write_wren(&mut spi, &mut seq);
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x02));

        expect_write_page(&mut spi, &mut seq, 0x0F, &[0x10]);
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));

        expect_write_wren(&mut spi, &mut seq);
        expect_write_page(
            &mut spi,
            &mut seq,
            0x10,
            &[0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90],
        );

        let mut delay = MockDelay::new();
        delay.expect_delay_us().withf(|_| true).return_const(());
        delay.expect_delay_ms().withf(|_| true).return_const(());

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
    }

    fn expect_write_wren(spi: &mut MockSpiDevice<u8>, seq: &mut Sequence) {
        spi.expect_transaction()
            .withf(|ops| {
                if let spi::Operation::Write(tx) = &ops[0] {
                    tx[0] == Opcode::WREN.as_u8()
                } else {
                    false
                }
            })
            .times(1)
            .in_sequence(seq)
            .return_const(Ok(()));
    }

    fn expect_read_status_register(
        spi: &mut MockSpiDevice<u8>,
        seq: &mut Sequence,
        returning: StatusRegister,
    ) {
        spi.expect_transaction()
            .withf(|ops| {
                if let spi::Operation::Transfer(_rx, tx) = &ops[0] {
                    tx == &[Opcode::RDSR.as_u8(), 0x00]
                } else {
                    false
                }
            })
            .times(1)
            .in_sequence(seq)
            .returning(move |ops| {
                if let spi::Operation::Transfer(rx, _tx) = &mut ops[0] {
                    rx[1] = returning.0;
                }
                Ok(())
            });
    }

    fn expect_write_page(
        spi: &mut MockSpiDevice<u8>,
        seq: &mut Sequence,
        address: u16,
        expected: &'static [u8],
    ) {
        spi.expect_transaction()
            .withf(move |tx| {
                tx[0]
                    == spi::Operation::Write(&[
                        Opcode::WRITE(address).as_u8(),
                        (address & 0xFF) as u8,
                    ])
                    && tx[1] == spi::Operation::Write(expected)
            })
            .times(1)
            .in_sequence(seq)
            .return_const(Ok(()));
    }
}

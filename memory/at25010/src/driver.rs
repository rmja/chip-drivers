use crate::{opcode::Opcode, traits, PartNumber};
use bitfield::bitfield;

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

#[derive(Debug)]
pub struct WriteProtectError;

pub struct Driver<Spi: traits::Spi, Timer: traits::Timer> {
    spi: Spi,
    timer: Timer,
    part_number: PartNumber,
}

impl<Spi, Timer> Driver<Spi, Timer>
where
    Spi: traits::Spi,
    Timer: traits::Timer,
{
    const INITIAL_TIMEOUT_US: u32 = 3_000; // Wait at least 3 ms
    const RETRY_INTERVAL_US: u32 = 100;

    pub fn new(spi: Spi, timer: Timer, part_number: PartNumber) -> Self {
        Self {
            part_number,
            spi,
            timer,
        }
    }

    /// Read a sequence of bytes from the EEPROM.
    pub async fn read(&mut self, origin: u16, buffer: &mut [u8]) {
        assert!(origin + buffer.len() as u16 <= capacity(self.part_number));

        self.spi.select();
        self.spi
            .write(&[Opcode::READ(origin).as_u8(), (origin & 0xFF) as u8])
            .await;
        self.spi.read(buffer).await;
        self.spi.deselect();
    }

    /// Write a sequence of bytes to the EEPROM.
    pub async fn write(&mut self, origin: u16, buffer: &[u8]) -> Result<(), WriteProtectError> {
        assert!(origin + buffer.len() as u16 <= capacity(self.part_number));

        let t_cs = min_tcs_ns(self.part_number);

        // Disable write protection.
        self.spi.select();
        self.spi.write(&[Opcode::WREN.as_u8()]).await;
        self.spi.deselect();

        // Wait until we can send a new spi command.
        self.timer.sleep_nanos(t_cs).await;

        // See if write was enabled (it may have been disabled by the WP pin).
        let sr = self.read_status_register().await;

        if !sr.wel() {
            return Err(WriteProtectError);
        }

        let mut address = origin;
        let mut write_enabled = true;
        let offset_in_first_page = origin as usize % PAGE_SIZE;
        let (incomplete_first_page, remaining_pages) =
            buffer.split_at((PAGE_SIZE - offset_in_first_page) % PAGE_SIZE);

        assert!(incomplete_first_page.len() < 8);
        if incomplete_first_page.len() > 0 {
            // Wait until we can send a new spi command.
            self.timer.sleep_nanos(t_cs).await;

            self.write_page(address, incomplete_first_page).await;
            address += incomplete_first_page.len() as u16;

            // Write is auto-disabled after sending a WRITE command.
            write_enabled = false;
        }

        for page in remaining_pages.chunks(PAGE_SIZE) {
            if !write_enabled {
                // Enable write.
                self.spi.select();
                self.spi.write(&[Opcode::WREN.as_u8()]).await;
                self.spi.deselect();
            }

            // Wait until we can send a new spi command.
            self.timer.sleep_nanos(t_cs).await;

            self.write_page(address as u16, page).await;
            address += page.len() as u16;

            // Write is auto-disabled after sending a WRITE command.
            write_enabled = false;
        }

        assert_eq!(origin + buffer.len() as u16, address);

        Ok(())
    }

    async fn write_page(&mut self, address: u16, buffer: &[u8]) {
        let len = buffer.len();
        assert!(len > 0);
        assert!(len <= PAGE_SIZE - (address as usize % PAGE_SIZE));

        self.spi.select();
        self.spi
            .write(&[Opcode::WRITE(address).as_u8(), (address & 0xFF) as u8])
            .await;
        self.spi.write(buffer).await;
        self.spi.deselect();

        // Wait for idle.
        self.timer.sleep_micros(Self::INITIAL_TIMEOUT_US).await;
        let sr = self.read_status_register().await;
        if sr.bsy() {
            loop {
                self.timer.sleep_micros(Self::RETRY_INTERVAL_US).await;

                let sr = self.read_status_register().await;
                if !sr.bsy() {
                    break;
                }
            }
        }
    }

    async fn read_status_register(&mut self) -> StatusRegister {
        const CMD: [u8; 2] = [Opcode::RDSR.as_u8(), 0x00];
        let mut rx: [u8; 2] = [0x00, 0x00];

        self.spi.select();
        self.spi.transfer(&CMD, &mut rx).await;
        self.spi.deselect();

        StatusRegister(rx[1])
    }
}

/// Get the EEPROM capacity in bytes
const fn capacity(kind: PartNumber) -> u16 {
    match kind {
        PartNumber::At25010 => 128,
        PartNumber::At25020 => 256,
        PartNumber::At25040 => 512,
        PartNumber::At25010b => 128,
        PartNumber::At25020b => 256,
        PartNumber::At25040b => 512,
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

    use crate::traits::{MockSpi, MockTimer};

    use super::*;

    #[tokio::test]
    async fn write_starting_at_page_boundary() {
        // Given
        let mut seq = Sequence::new();
        let mut spi = MockSpi::new();

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
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));

        let mut timer = MockTimer::new();
        timer.expect_sleep_nanos().withf(|_| true).return_const(());
        timer.expect_sleep_micros().withf(|_| true).return_const(());

        // When
        let mut driver = Driver::new(spi, timer, PartNumber::At25010b);

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
        let mut spi = MockSpi::new();

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
        expect_read_status_register(&mut spi, &mut seq, StatusRegister(0x00));

        let mut timer = MockTimer::new();
        timer.expect_sleep_nanos().withf(|_| true).return_const(());
        timer.expect_sleep_micros().withf(|_| true).return_const(());

        // When
        let mut driver = Driver::new(spi, timer, PartNumber::At25010b);

        driver
            .write(
                0x0F,
                &[0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90],
            )
            .await
            .unwrap();

        // Then
    }

    fn expect_write_wren(spi: &mut MockSpi, seq: &mut Sequence) {
        spi.expect_select()
            .times(1)
            .in_sequence(seq)
            .return_const(());
        spi.expect_write()
            .withf(|tx| tx == &[Opcode::WREN.as_u8()])
            .times(1)
            .in_sequence(seq)
            .return_const(());
        spi.expect_deselect()
            .times(1)
            .in_sequence(seq)
            .return_const(());
    }

    fn expect_read_status_register(
        spi: &mut MockSpi,
        seq: &mut Sequence,
        returning: StatusRegister,
    ) {
        spi.expect_select()
            .times(1)
            .in_sequence(seq)
            .return_const(());
        spi.expect_transfer()
            .withf(|tx, _| tx == &[Opcode::RDSR.as_u8(), 0x00])
            .times(1)
            .in_sequence(seq)
            .returning(move |_tx, rx| {
                rx[1] = returning.0;
            });
        spi.expect_deselect()
            .times(1)
            .in_sequence(seq)
            .return_const(());
    }

    fn expect_write_page(
        spi: &mut MockSpi,
        seq: &mut Sequence,
        address: u16,
        expected: &'static [u8],
    ) {
        spi.expect_select()
            .times(1)
            .in_sequence(seq)
            .return_const(());
        spi.expect_write()
            .withf(move |tx| tx == &[Opcode::WRITE(address).as_u8(), (address & 0xFF) as u8])
            .times(1)
            .in_sequence(seq)
            .return_const(());
        spi.expect_write()
            .withf(move |tx| tx == expected)
            .times(1)
            .in_sequence(seq)
            .return_const(());
        spi.expect_deselect()
            .times(1)
            .in_sequence(seq)
            .return_const(());
    }
}

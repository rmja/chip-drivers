use crate::{
    errors::*,
    opcode::{ExtReg, Opcode, Reg, Strobe, OPCODE_MAX},
    statusbyte::{State, StatusByte},
    traits, PartNumber, Rssi, RX_FIFO_SIZE, TX_FIFO_SIZE, ConfigPatch,
};
use alloc::vec;
use futures::future::{self, Either};

pub struct Driver<Spi: traits::Spi, Pins: traits::Pins, Timer: traits::Timer> {
    spi: Spi,
    pins: Pins,
    timer: Timer,
    last_status: Option<StatusByte>,
    pub rssi_offset: Rssi,
}

impl<Spi: traits::Spi, Pins: traits::Pins, Timer: traits::Timer> Driver<Spi, Pins, Timer> {
    pub fn new(spi: Spi, pins: Pins, timer: Timer) -> Self {
        Self {
            spi,
            pins,
            timer,
            last_status: None,
            rssi_offset: -99, // Default offset defined in users guide
        }
    }

    pub async fn init(&mut self) {
        self.pins.set_reset(); // Release chip reset pin.
        future::ready(()).await
    }

    pub async fn hw_reset(&mut self) -> Result<(), TimeoutError> {
        // Reset chip.
        self.pins.clear_reset(); // Trigger chip reset pin.
        self.timer.sleep_millis(2).await;
        self.pins.set_reset(); // Release chip reset pin.

        // Wait for chip to become available.
        self.spi.select();
        self.timer.sleep_millis(1).await; // Wait 1ms until the chip has had a chance to set the SO pin high.
        let result = self.wait_for_xtal().await;
        self.spi.deselect();

        self.last_status = None;

        result
    }

    /// Get the spi status returned by the last register read or strobe.
    /// Writing registers does not update status.
    pub fn last_status(&self) -> Option<StatusByte> {
        self.last_status
    }

    /// Read the chip part number.
    /// This action _does_ update `last_status`.
    pub async fn read_part_number(&mut self) -> Result<PartNumber, InvalidPartNumber> {
        match self.read_reg(ExtReg::PARTNUMBER).await {
            0x20 => Ok(PartNumber::Cc1200),
            0x21 => Ok(PartNumber::Cc1201),
            _ => Err(InvalidPartNumber),
        }
    }

    /// Read a single register value from chip.
    /// This action _does_ update `last_status`.
    pub async fn read_reg<R: Reg>(&mut self, reg: R) -> u8 {
        let mut tx_buffer = [0; OPCODE_MAX + 1];
        let opcode_len = reg.get_read_opcode(false).assign(&mut tx_buffer);
        let tx = &tx_buffer[..opcode_len + 1];

        let mut rx_buffer = [0; OPCODE_MAX + 1];
        let rx = &mut rx_buffer[..tx.len()];

        self.spi.select();
        self.spi.transfer(tx, rx).await;
        self.last_status = Some(StatusByte(rx[0]));
        self.spi.deselect();

        rx[rx.len() - 1]
    }

    /// Read a sequence of register values from chip.
    /// This action _does_ update `last_status`.
    pub async fn read_regs<R: Reg>(&mut self, first: R, buffer: &mut [u8]) {
        let mut opcode_tx_buffer = [0; OPCODE_MAX];
        let opcode_len = first
            .get_read_opcode(buffer.len() > 1)
            .assign(&mut opcode_tx_buffer);
        let opcode_tx = &opcode_tx_buffer[..opcode_len];

        let mut opcode_rx_buffer = [0; OPCODE_MAX];
        let opcode_rx = &mut opcode_rx_buffer[..opcode_tx.len()];

        self.spi.select();
        self.spi.transfer(opcode_tx, opcode_rx).await;
        self.last_status = Some(StatusByte(opcode_rx[0]));
        self.spi.read(buffer).await;
        self.spi.deselect();
    }

    /// Write a single register value to chip.
    /// This action _does not_ update `last_status`.
    pub async fn write_reg<R: Reg>(&mut self, reg: R, value: u8) {
        let mut tx_buffer = [0; OPCODE_MAX + 1];
        let opcode_len = reg.get_write_opcode(false).assign(&mut tx_buffer);
        let tx = &mut tx_buffer[0..opcode_len + 1];
        tx[opcode_len] = value;

        self.spi.select();
        self.spi.write(tx).await;
        self.spi.deselect();

        self.last_status = None;
    }

    /// Write a sequence of register values to chip.
    /// This action _does not_ update `last_status`.
    pub async fn write_regs<R: Reg>(&mut self, first: R, values: &[u8]) {
        let mut opcode_tx_buffer = [0; OPCODE_MAX];
        let opcode_len = first
            .get_write_opcode(values.len() > 1)
            .assign(&mut opcode_tx_buffer);
        let opcode_tx = &opcode_tx_buffer[..opcode_len];

        self.spi.select();
        self.spi.write(opcode_tx).await;
        self.spi.write(values).await;
        self.spi.deselect();

        self.last_status = None;
    }

    /// Write a configuration patch to chip.
    /// This action _does not_ update `last_status`.
    pub async fn write_patch<'a, 'patch, R: Reg>(&'a mut self, patch: ConfigPatch<'patch, R>) where 'a: 'patch {
        self.write_regs(patch.first, patch.values).await;
    }

    /// Modify register values.
    /// This action _does_ update `last_status`.
    pub async fn modify_regs<R: Reg, F: FnOnce(&mut [u8])>(
        &mut self,
        first: R,
        count: usize,
        configure: F,
    ) {
        let mut buf = vec![0; count];
        self.read_regs(first, &mut buf).await;
        configure(&mut buf);
        self.write_regs(first, &buf).await;

        self.last_status = None;
    }

    /// Read the current RSSI level.
    /// This action _does_ update `last_status`.
    pub async fn read_rssi(&mut self) -> Result<Rssi, InvalidRssi> {
        let mut tx = [0; 3];
        assert_eq!(2, ExtReg::RSSI1.get_write_opcode(false).assign(&mut tx));

        let mut rx = [0; 3];

        self.spi.select();
        self.spi.transfer(&tx, &mut rx).await;
        self.last_status = Some(StatusByte(rx[0]));
        self.spi.deselect();

        self.map_rssi(rx[2])
    }

    /// Read from the RX fifo.
    /// This action _does_ update `last_status`.
    pub async fn read_fifo(&mut self, buffer: &mut [u8]) {
        assert!(buffer.len() <= RX_FIFO_SIZE);

        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::ReadFifoBurst.assign(&mut opcode_tx));

        let mut opcode_rx = [0];

        self.spi.select();
        self.spi.transfer(&opcode_tx, &mut opcode_rx).await;
        self.last_status = Some(StatusByte(opcode_rx[0]));
        self.spi.read(buffer).await;
        self.spi.deselect();
    }

    /// Read the RSSI and RX fifo in one transaction.
    /// This action _does_ update `last_status`.
    pub async fn read_rssi_and_fifo(&mut self, buffer: &mut [u8]) -> Result<Rssi, InvalidRssi> {
        assert!(buffer.len() <= RX_FIFO_SIZE);

        let mut tx = [0; 3 + 1];
        assert_eq!(
            2,
            Opcode::ReadExtSingle(ExtReg::RSSI1).assign(&mut tx[0..2])
        );
        // RSSI is returned intx[2]
        assert_eq!(1, Opcode::ReadFifoBurst.assign(&mut tx[3..4]));

        let mut rx = [0; 3 + 1];

        self.spi.select();
        self.spi.transfer(&tx, &mut rx).await;
        self.last_status = Some(StatusByte(rx[0]));
        self.spi.read(buffer).await;
        self.spi.deselect();

        self.map_rssi(rx[2])
    }

    /// Write to the TX fifo.
    /// This action _does_ update `last_status`.
    pub async fn write_fifo(&mut self, buffer: &[u8]) {
        assert!(buffer.len() <= TX_FIFO_SIZE);

        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::WriteFifoBurst.assign(&mut opcode_tx));

        let mut opcode_rx = [0];

        self.spi.select();
        self.spi.transfer(&opcode_tx, &mut opcode_rx).await;
        self.last_status = Some(StatusByte(opcode_rx[0]));
        self.spi.write(buffer).await;
        self.spi.deselect();
    }

    // Map the RSSI1 register field to an rssi value.
    fn map_rssi(&self, rssi1_value: u8) -> Result<Rssi, InvalidRssi> {
        let rssi = rssi1_value as i8;
        match rssi {
            -128 => Err(InvalidRssi),
            rssi => Ok(rssi + self.rssi_offset),
        }
    }

    /// Wait for the xtal to stabilize.
    async fn wait_for_xtal(&mut self) -> Result<(), TimeoutError> {
        let rising = self.pins.miso_wait_low();
        let timeout = self.timer.sleep_millis(2_000);

        // Wait for any of the two futures to complete.
        match future::select(rising, timeout).await {
            Either::Left(_) => Ok(()),
            Either::Right(_) => Err(TimeoutError),
        }
    }

    /// Strobe a command to the chip.
    /// This action _does_ update `last_status`.
    pub async fn strobe(&mut self, strobe: Strobe) {
        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::Strobe(strobe).assign(&mut opcode_tx));
        let mut opcode_rx = [0];

        self.spi.select();
        self.spi.transfer(&opcode_tx, &mut opcode_rx).await;
        self.last_status = Some(StatusByte(opcode_rx[0]));
        if strobe == Strobe::SRES {
            // When SRES strobe is issued the CSn pin must be kept low until the SO pin goes low again.
            self.pins.miso_wait_low().await;
        }
        self.spi.deselect();
    }

    /// Strobe a command to the chip, and continue to do so until `pred` is satisfied.
    /// This action _does_ update `last_status`.
    pub async fn strobe_until<Pred>(&mut self, strobe: Strobe, pred: Pred)
    where
        Pred: Fn(StatusByte) -> bool,
    {
        assert_ne!(Strobe::SRES, strobe);

        let mut opcode_tx = [0];
        assert_eq!(1, Opcode::Strobe(strobe).assign(&mut opcode_tx));
        let mut opcode_rx = [0];

        self.spi.select();
        loop {
            self.spi.transfer(&opcode_tx, &mut opcode_rx).await;
            self.last_status = Some(StatusByte(opcode_rx[0]));

            if pred(self.last_status.unwrap()) {
                break;
            }
        }
        self.spi.deselect();
    }

    /// Strobe a command to the chip, and continue to do so until the chip enters the IDLE state.
    /// This action _does_ update `last_status`.
    pub async fn strobe_until_idle(&mut self, strobe: Strobe) {
        self.strobe_until(strobe, |status| status.state() == State::IDLE)
            .await;
    }
}

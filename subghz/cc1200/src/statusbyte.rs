use bitfield::bitfield;
use core::mem::transmute;

bitfield! {
    /// The status byte sent over SPI when the header byte, data byte, or command strobe is sent.
    #[derive(Clone, Copy)]
    pub struct StatusByte(u8);
    /// Stays high until power and crystal have stabilized. Should always be low when using the SPI interface.
    pub chip_rdyn, _: 7;
    /// Indicates the current main state machine mode.
    state_bits, _: 6, 4;
    reserved, _: 3, 0;
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// Idle state
    IDLE = 0b000,
    /// Receive mode
    RX = 0b001,
    /// Transmit mode
    TX = 0b010,
    /// Fast TX ready
    FSTXON = 0b011,
    /// Frequency synthesizer calibration is running
    CALIBRATE = 0b100,
    /// PLL is settling
    SETTLING = 0b101,
    /// RX FIFO has over/underflowed
    RX_FIFO_ERROR = 0b110,
    /// TX FIFO has over/underflowed.
    TX_FIFO_ERROR = 0b111,
}

impl StatusByte {
    /// Get the current chip state
    pub fn state(self) -> State {
        unsafe { transmute(self.state_bits()) }
    }

    /// Get whether the chip is ready.
    /// true if the chip is ready, false otherwise
    pub fn chip_rdy(self) -> bool {
        !self.chip_rdyn()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn can_get_state() {
        // Given
        let byte = StatusByte(0b1_110_0000);

        // Then
        assert_eq!(State::RX_FIFO_ERROR, byte.state());
        assert_eq!(true, byte.chip_rdyn());
    }
}

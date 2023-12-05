use core::mem::transmute;

use super::ext::Marcstate;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum MarcStateValue {
    SLEEP = 0b00000,
    IDLE = 0b00001,
    XOFF = 0b00010,
    BIAS_SETTLE_MC = 0b00011,
    REG_SETTLE_MC = 0b00100,
    MANCAL = 0b00101,
    BIAS_SETTLE = 0b00110,
    REG_SETTLE = 0b00111,
    STARTCAL = 0b01000,
    BWBOOST = 0b01001,
    FS_LOCK = 0b01010,
    IFADCON = 0b01011,
    ENDCAL = 0b01100,
    RX = 0b01101,
    RX_END = 0b01110,
    RXDCM = 0b01111,
    TXRX_SWITCH = 0b10000,
    RX_FIFO_ERR = 0b10001,
    FSTXON = 0b10010,
    TX = 0b10011,
    TX_END = 0b10100,
    RXTX_SWITCH = 0b10101,
    TX_FIFO_ERR = 0b10110,
    IFADCON_TXRX = 0b10111,
    Reserved_11000 = 0b11000,
    Reserved_11001 = 0b11001,
    Reserved_11010 = 0b11010,
    Reserved_11011 = 0b11011,
    Reserved_11100 = 0b11100,
    Reserved_11101 = 0b11101,
    Reserved_11110 = 0b11110,
    Reserved_11111 = 0b11111,
}

impl Marcstate {
    pub fn marc_state(&self) -> MarcStateValue {
        unsafe { transmute(self.marc_state_bits()) }
    }
}

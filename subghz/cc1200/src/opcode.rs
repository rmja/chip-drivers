pub const OPCODE_MAX: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opcode {
    /// Write a single register.
    WriteSingle(PriReg),
    /// Write multiple registers.
    WriteBurst(PriReg),
    /// Read a single register.
    ReadSingle(PriReg),
    /// Read multiple registers.
    ReadBurst(PriReg),

    /// Write a single register.
    WriteExtSingle(ExtReg),
    WriteExtBurst(ExtReg),
    ReadExtSingle(ExtReg),
    ReadExtBurst(ExtReg),

    /// Command strobe.
    Strobe(Strobe),
    /// Read from RX FIFO.
    ReadFifoBurst,
    /// Write to TX FIFO.
    WriteFifoBurst,
}

impl Opcode {
    pub fn assign(&self, buffer: &mut [u8]) -> usize {
        buffer[0] = self.as_u8();

        match *self {
            Opcode::WriteExtSingle(reg) => {
                buffer[1] = reg as u8;
                2
            }
            Opcode::WriteExtBurst(reg) => {
                buffer[1] = reg as u8;
                2
            }
            Opcode::ReadExtSingle(reg) => {
                buffer[1] = reg as u8;
                2
            }
            Opcode::ReadExtBurst(reg) => {
                buffer[1] = reg as u8;
                2
            }
            _ => 1,
        }
    }

    pub const fn as_u8(&self) -> u8 {
        const SINGLE_WRITE: u8 = 0x00;
        const BURST_WRITE: u8 = 0x40;
        const SINGLE_READ: u8 = 0x80;
        const BURST_READ: u8 = 0xC0;

        match *self {
            Opcode::WriteSingle(reg) => SINGLE_WRITE | reg as u8,
            Opcode::WriteBurst(reg) => BURST_WRITE | reg as u8,
            Opcode::ReadSingle(reg) => SINGLE_READ | reg as u8,
            Opcode::ReadBurst(reg) => BURST_READ | reg as u8,
            Opcode::WriteExtSingle(_) => SINGLE_WRITE | PriReg::EXTENDED_ADDRESS as u8,
            Opcode::WriteExtBurst(_) => BURST_WRITE | PriReg::EXTENDED_ADDRESS as u8,
            Opcode::ReadExtSingle(_) => SINGLE_READ | PriReg::EXTENDED_ADDRESS as u8,
            Opcode::ReadExtBurst(_) => BURST_READ | PriReg::EXTENDED_ADDRESS as u8,
            Opcode::Strobe(strobe) => strobe as u8,
            Opcode::WriteFifoBurst => BURST_WRITE | 0x3F,
            Opcode::ReadFifoBurst => BURST_READ | 0x3F,
        }
    }
}

/// Register marker trait
#[const_trait]
pub trait Reg: Copy + PartialEq {
    const FIRST: Self;
    const LAST: Self;

    fn as_u8(self) -> u8;
    fn get_read_opcode(self, burst: bool) -> Opcode;
    fn get_write_opcode(self, burst: bool) -> Opcode;
}

impl const Reg for PriReg {
    const FIRST: Self = Self::IOCFG0;
    const LAST: Self = Self::PKT_LEN;

    fn as_u8(self) -> u8 {
        self as u8
    }

    fn get_read_opcode(self, burst: bool) -> Opcode {
        if burst {
            Opcode::ReadBurst(self)
        } else {
            Opcode::ReadSingle(self)
        }
    }

    fn get_write_opcode(self, burst: bool) -> Opcode {
        if burst {
            Opcode::WriteBurst(self)
        } else {
            Opcode::WriteSingle(self)
        }
    }
}

impl const Reg for ExtReg {
    const FIRST: Self = Self::IF_MIX_CFG;
    const LAST: Self = Self::PA_CFG3;

    fn as_u8(self) -> u8 {
        self as u8
    }

    fn get_read_opcode(self, burst: bool) -> Opcode {
        if burst {
            Opcode::ReadExtBurst(self)
        } else {
            Opcode::ReadExtSingle(self)
        }
    }

    fn get_write_opcode(self, burst: bool) -> Opcode {
        if burst {
            Opcode::WriteExtBurst(self)
        } else {
            Opcode::WriteExtSingle(self)
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PriReg {
    IOCFG3 = 0x00,
    IOCFG2 = 0x01,
    IOCFG1 = 0x02,
    IOCFG0 = 0x03,
    SYNC3 = 0x04,
    SYNC2 = 0x05,
    SYNC1 = 0x06,
    SYNC0 = 0x07,
    SYNC_CFG1 = 0x08,
    SYNC_CFG0 = 0x09,
    DEVIATION_M = 0x0A,
    MODCFG_DEV_E = 0x0B,
    DCFILT_CFG = 0x0C,
    PREAMBLE_CFG1 = 0x0D,
    PREAMBLE_CFG0 = 0x0E,
    IQIC = 0x0F,
    CHAN_BW = 0x10,
    MDMCFG1 = 0x11,
    MDMCFG0 = 0x12,
    SYMBOL_RATE2 = 0x13,
    SYMBOL_RATE1 = 0x14,
    SYMBOL_RATE0 = 0x15,
    AGC_REF = 0x16,
    AGC_CS_THR = 0x17,
    AGC_GAIN_ADJUST = 0x18,
    AGC_CFG3 = 0x19,
    AGC_CFG2 = 0x1A,
    AGC_CFG1 = 0x1B,
    AGC_CFG0 = 0x1C,
    FIFO_CFG = 0x1D,
    DEV_ADDR = 0x1E,
    SETTLING_CFG = 0x1F,
    FS_CFG = 0x20,
    WOR_CFG1 = 0x21,
    WOR_CFG0 = 0x22,
    WOR_EVENT0_MSB = 0x23,
    WOR_EVENT0_LSB = 0x24,
    RXDCM_TIME = 0x25,
    PKT_CFG2 = 0x26,
    PKT_CFG1 = 0x27,
    PKT_CFG0 = 0x28,
    RFEND_CFG1 = 0x29,
    RFEND_CFG0 = 0x2A,
    PA_CFG1 = 0x2B,
    PA_CFG0 = 0x2C,
    ASK_CFG = 0x2D,
    PKT_LEN = 0x2E,
    EXTENDED_ADDRESS = 0x2F,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ExtReg {
    IF_MIX_CFG = 0x00,
    FREQOFF_CFG = 0x01,
    TOC_CFG = 0x02,
    MARC_SPARE = 0x03,
    ECF_CFG = 0x04,
    MDMCFG2 = 0x05,
    EXT_CTRL = 0x06,
    RCCAL_FINE = 0x07,
    RCCAL_COARSE = 0x08,
    RCCAL_OFFSET = 0x09,
    FREQOFF1 = 0x0A,
    FREQOFF0 = 0x0B,
    FREQ2 = 0x0C,
    FREQ1 = 0x0D,
    FREQ0 = 0x0E,
    IF_ADC2 = 0x0F,
    IF_ADC1 = 0x10,
    IF_ADC0 = 0x11,
    FS_DIG1 = 0x12,
    FS_DIG0 = 0x13,
    FS_CAL3 = 0x14,
    FS_CAL2 = 0x15,
    FS_CAL1 = 0x16,
    FS_CAL0 = 0x17,
    FS_CHP = 0x18,
    FS_DIVTWO = 0x19,
    FS_DSM1 = 0x1A,
    FS_DSM0 = 0x1B,
    FS_DVC1 = 0x1C,
    FS_DVC0 = 0x1D,
    FS_LBI = 0x1E,
    FS_PFD = 0x1F,
    FS_PRE = 0x20,
    FS_REG_DIV_CML = 0x21,
    FS_SPARE = 0x22,
    FS_VCO4 = 0x23,
    FS_VC03 = 0x24,
    FS_VC02 = 0x25,
    FS_VC01 = 0x26,
    FS_VC00 = 0x27,
    GBIAS6 = 0x28,
    GBIAS5 = 0x29,
    GBIAS4 = 0x2A,
    GBIAS3 = 0x2B,
    GBIAS2 = 0x2C,
    GBIAS1 = 0x2D,
    GBIAS0 = 0x2E,
    IFAMP = 0x2F,
    LNA = 0x30,
    RXMIX = 0x31,
    XOSC5 = 0x32,
    XOSC4 = 0x33,
    XOSC3 = 0x34,
    XOSC2 = 0x35,
    XOSC1 = 0x36,
    XOSC0 = 0x37,
    ANALOG_SPARE = 0x38,
    PA_CFG3 = 0x39,
    // 0x3A-0x3E: Not used
    // 0x3F-0x40: Reserved
    // 0x41-0x63: Not used
    WOR_TIME1 = 0x64,
    WOR_TIME0 = 0x65,
    WOR_CAPTURE1 = 0x66,
    WOR_CAPtURE0 = 0x67,
    BIST = 0x68,
    DCFILTOFFSET_I1 = 0x69,
    DCFILTOFFSET_I0 = 0x6A,
    DCFILTOFFSET_Q1 = 0x6B,
    DCFILTOFFSET_Q0 = 0x6C,
    IQIE_I1 = 0x6D,
    IQIE_I0 = 0x6E,
    IQIE_Q1 = 0x6F,
    IQIE_Q0 = 0x70,
    RSSI1 = 0x71,
    RSSI0 = 0x72,
    MARCSTATE = 0x73,
    LQI_VAL = 0x74,
    PQT_SYNC_ERR = 0x75,
    DEM_STATUS = 0x76,
    FREQOFF_EST1 = 0x77,
    FREQOFF_EST0 = 0x78,
    AGC_GAIN3 = 0x79,
    AGC_GAIN2 = 0x7A,
    AGC_GAIN1 = 0x7B,
    AGC_GAIN0 = 0x7C,
    CFM_RX_DATA_OUT = 0x7D,
    CFM_TX_DATA_IN = 0x7E,
    ASK_SOFT_RX_DATA = 0x7F,
    RNDGEN = 0x80,
    MAGN2 = 0x81,
    MAGN1 = 0x82,
    MAGN0 = 0x83,
    ANG1 = 0x84,
    ANG0 = 0x85,
    CHFILT_I2 = 0x86,
    CHFILT_I1 = 0x87,
    CHFILT_I0 = 0x88,
    CHFILT_Q2 = 0x89,
    CHFILT_Q1 = 0x8A,
    CHFILT_Q0 = 0x8B,
    GPIO_STATUS = 0x8C,
    FSCAL_CTRL = 0x8D,
    PHASE_ADJUST = 0x8E,
    PARTNUMBER = 0x8F,
    PARTVERSION = 0x90,
    SERIAL_STATUS = 0x91,
    MODEM_STATUS1 = 0x92,
    MODEM_STATUS0 = 0x93,
    MARC_STATUS1 = 0x94,
    MARC_STATUS0 = 0x95,
    PA_IFAMP_TEST = 0x96,
    FSRF_TEST = 0x97,
    PRE_TEST = 0x98,
    PRE_OVR = 0x99,
    ADC_TEST = 0x9A,
    DVC_TEST = 0x9B,
    ATEST = 0x9C,
    ATEST_LVDS = 0x9D,
    ATEST_MODE = 0x9E,
    XOSC_TEST1 = 0x9F,
    XOSC_TEST0 = 0xA0,
    AES = 0xA1,
    MDM_TEST = 0xA2,
    // 0xA3-0xD1
    RXFIRST = 0xD2,
    TXFIRST = 0xD3,
    RXLAST = 0xD4,
    TXLAST = 0xD5,
    NUM_TXBYTES = 0xD6,
    NUM_RXBYTES = 0xD7,
    FIFO_NUM_TXBYTES = 0xD8,
    FIFO_NUM_RXBYTES = 0xD9,
    RXFIFO_PRE_BUF = 0xDA,
    // 0xDB-0xDF
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Strobe {
    /// Reset chip
    SRES = 0x30,
    /// Enable and calibrate frequency synthesizer
    SFSTXON = 0x31,
    /// Turn off crystal oscillator (Enter XOFF state when CSn is de-asserted)
    SXOFF = 0x32,
    /// Calibrate frequency synthesizer and turn it off
    SCAL = 0x33,
    /// Enable RX
    SRX = 0x34,
    /// Enable TX
    STX = 0x35,
    /// Exit RX/TX and turn off frequency synthesizer
    SIDLE = 0x36,
    /// Automatic frequency compensation
    SAFC = 0x37,
    /// Start automatic RX polling sequence
    SWOR = 0x38,
    /// Enter SLEEP mode when CSn is de-asserted
    SPWD = 0x39,
    /// Flush the RX FIFO
    SFRX = 0x3A,
    /// Flush the TX FIFO
    SFTX = 0x3B,
    /// Reset real time clock
    SWORRST = 0x3C,
    /// No operation - may be used to get access to the chip status byte
    SNOP = 0x3D,
}

pub const OPCODE_MAX: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opcode {
    /// Write a single register.
    WriteSingle(u16),
    /// Write multiple registers.
    WriteBurst(u16),
    /// Read a single register.
    ReadSingle(u16),
    /// Read multiple registers.
    ReadBurst(u16),

    /// Command strobe.
    Strobe(Strobe),
    /// Read from RX FIFO.
    ReadFifoBurst,
    /// Write to TX FIFO.
    WriteFifoBurst,
}

impl Opcode {
    pub const fn read(address: u16, burst: bool) -> Self {
        if burst {
            Opcode::ReadBurst(address)
        } else {
            Opcode::ReadSingle(address)
        }
    }

    pub const fn write(address: u16, burst: bool) -> Self {
        if burst {
            Opcode::WriteBurst(address)
        } else {
            Opcode::WriteSingle(address)
        }
    }

    pub fn assign(&self, buffer: &mut [u8]) -> usize {
        buffer[0] = self.as_u8();

        match *self {
            Opcode::WriteSingle(address) if address > 0x2F => {
                buffer[1] = (address & 0xFF) as u8;
                2
            }
            Opcode::WriteBurst(address) if address > 0x2F => {
                buffer[1] = (address & 0xFF) as u8;
                2
            }
            Opcode::ReadSingle(address) if address > 0x2F => {
                buffer[1] = (address & 0xFF) as u8;
                2
            }
            Opcode::ReadBurst(address) if address > 0x2F => {
                buffer[1] = (address & 0xFF) as u8;
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
        const EXTENDED_ADDRESS: u8 = 0x2F;

        match *self {
            Opcode::WriteSingle(address) if address <= 0x2F => SINGLE_WRITE | address as u8,
            Opcode::WriteBurst(address) if address <= 0x2F => BURST_WRITE | address as u8,
            Opcode::ReadSingle(address) if address <= 0x2F => SINGLE_READ | address as u8,
            Opcode::ReadBurst(address) if address <= 0x2F => BURST_READ | address as u8,
            Opcode::WriteSingle(_) => SINGLE_WRITE | EXTENDED_ADDRESS,
            Opcode::WriteBurst(_) => BURST_WRITE | EXTENDED_ADDRESS,
            Opcode::ReadSingle(_) => SINGLE_READ | EXTENDED_ADDRESS,
            Opcode::ReadBurst(_) => BURST_READ | EXTENDED_ADDRESS,
            Opcode::Strobe(strobe) => strobe as u8,
            Opcode::WriteFifoBurst => BURST_WRITE | 0x3F,
            Opcode::ReadFifoBurst => BURST_READ | 0x3F,
        }
    }
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

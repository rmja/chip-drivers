#[allow(dead_code)]
pub(crate) enum Opcode {
    /// Set write enable latch
    WREN,
    /// Reset write enable latch
    WRDI,
    /// Read status register
    RDSR,
    /// Write status register
    WRSR,
    /// Read from memory array
    READ(u16),
    /// Write from memory array
    WRITE(u16),
}

impl Opcode {
    pub const fn as_u8(self) -> u8 {
        const A8: u8 = 0b1000;
        match self {
            Opcode::WREN => 0b110,
            Opcode::WRDI => 0b100,
            Opcode::RDSR => 0b101,
            Opcode::WRSR => 0b001,
            Opcode::READ(addr) if addr > 0xFF => A8 | 0b011,
            Opcode::READ(_) => 0b011,
            Opcode::WRITE(addr) if addr > 0xFF => A8 | 0b010,
            Opcode::WRITE(_) => 0b010,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read() {
        assert_eq!(0b0011, Opcode::READ(0x00).as_u8());
        assert_eq!(0b0011, Opcode::READ(0xFF).as_u8());
        assert_eq!(0b1011, Opcode::READ(0x100).as_u8());
    }

    #[test]
    fn write() {
        assert_eq!(0b0010, Opcode::WRITE(0x00).as_u8());
        assert_eq!(0b0010, Opcode::WRITE(0xFF).as_u8());
        assert_eq!(0b1010, Opcode::WRITE(0x100).as_u8());
    }
}
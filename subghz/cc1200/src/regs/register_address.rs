use core::ops::Add;

use super::{
    ext::{IfMixCfg, PaCfg3},
    pri::{Iocfg3, PktLen},
    Register, RegisterAddress,
};

impl RegisterAddress {
    pub const PRI_MIN: RegisterAddress = Iocfg3::ADDRESS;
    pub const PRI_MAX: RegisterAddress = PktLen::ADDRESS;
    pub const EXT_MIN: RegisterAddress = IfMixCfg::ADDRESS;
    pub const EXT_MAX: RegisterAddress = PaCfg3::ADDRESS;

    pub const fn split(self, len: usize) -> ((RegisterAddress, usize), (RegisterAddress, usize)) {
        if self.0 <= Self::PRI_MAX.0 && self.0 + len as u16 <= Self::PRI_MAX.0 {
            ((self, len), (Self::EXT_MIN, 0))
        } else if self.0 >= Self::EXT_MIN.0 {
            ((Self::PRI_MIN, 0), (self, len))
        } else {
            let pri_len = (Self::PRI_MAX.0 - self.0 + 1) as usize;
            let ext_len = len - pri_len;
            ((self, pri_len), (Self::EXT_MIN, ext_len))
        }
    }
}

impl Add<usize> for RegisterAddress {
    type Output = Result<Self, ()>;

    fn add(self, rhs: usize) -> Self::Output {
        let rhs = rhs as u16;
        let sum = self.0 + rhs;

        if self.0 >= Self::PRI_MIN.0 && sum <= Self::PRI_MAX.0 {
            // Already primary register, remains primary register
            Ok(Self(sum))
        } else if self.0 >= Self::EXT_MIN.0 && sum <= Self::EXT_MAX.0 {
            // Already extended register, remains extended register
            Ok(Self(sum))
        } else if self.0 <= Self::PRI_MAX.0 {
            // Was primary, becomes extended after addition
            let pri_len = Self::PRI_MAX.0 + 1;
            let ext_offset = sum - pri_len;
            Ok(Self(Self::EXT_MIN.0 + ext_offset))
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::regs::{ext::*, pri::*};

    use super::*;

    #[test]
    fn can_add() {
        assert_eq!(Ok(Iocfg2::ADDRESS), Iocfg3::ADDRESS + 1);
        assert_eq!(Ok(IfMixCfg::ADDRESS), PktLen::ADDRESS + 1);
        assert_eq!(Ok(FreqoffCfg::ADDRESS), IfMixCfg::ADDRESS + 1);
        assert_eq!(Ok(PaCfg3::ADDRESS), AnalogSpare::ADDRESS + 1);
        assert_eq!(Err(()), PaCfg3::ADDRESS + 1);
    }
}

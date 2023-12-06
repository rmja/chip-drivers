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

    pub const fn idx(&self) -> usize {
        if self.0 <= Self::PRI_MAX.0 {
            (self.0 - Self::PRI_MIN.0) as usize
        } else {
            let offset = Self::PRI_MAX.0 - Self::PRI_MIN.0 + 1;
            (offset + self.0 - Self::EXT_MIN.0) as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idx() {
        let addresses = RegisterAddress::PRI_MIN.0..=RegisterAddress::PRI_MAX.0;
        for (index, pri) in addresses.into_iter().enumerate() {
            let pri = RegisterAddress(pri);
            assert_eq!(index, pri.idx());
        }

        let addresses = RegisterAddress::EXT_MIN.0..=RegisterAddress::EXT_MAX.0;
        for (index, ext) in addresses.into_iter().enumerate() {
            let ext = RegisterAddress(ext);
            assert_eq!(0x2F + index, ext.idx());
        }
    }
}

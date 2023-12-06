use crate::regs::{Register, RegisterAddress};

const PRI_MIN: RegisterAddress = RegisterAddress::PRI_MIN;
const PRI_MAX: RegisterAddress = RegisterAddress::PRI_MAX;
const EXT_MIN: RegisterAddress = RegisterAddress::EXT_MIN;
const EXT_MAX: RegisterAddress = RegisterAddress::EXT_MAX;

pub struct Config(pub [u8; 105]);

#[derive(Clone, Copy)]
pub struct ConfigPatch<'a> {
    pub first_address: RegisterAddress,
    pub values: &'a [u8],
}

impl<'a> ConfigPatch<'a> {
    pub const fn new(config: &'a Config) -> Self {
        ConfigPatch {
            first_address: PRI_MIN,
            values: &config.0,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Get a register value, or None if the register is not part of the configuration.
    pub fn get<R: Register>(&self) -> Option<R> {
        let index = if self.first_address < EXT_MIN && R::ADDRESS >= EXT_MIN {
            (0x2F - self.first_address.0 + R::ADDRESS.0 - EXT_MIN.0) as usize
        } else {
            (R::ADDRESS.0 - self.first_address.0) as usize
        };

        let value = self.values.get(index);
        if let Some(&value) = value {
            Some(R::from(value))
        } else {
            None
        }
    }

    pub const fn split_pri_ext(self) -> (ConfigPatch<'a>, ConfigPatch<'a>) {
        let first = self.first_address;
        let len = self.values.len() as u16;
        let (pri, ext) = if first.0 <= PRI_MAX.0 && first.0 + len <= PRI_MAX.0 {
            ((first, len), (EXT_MIN, 0))
        } else if first.0 >= EXT_MIN.0 {
            ((PRI_MIN, 0), (first, len))
        } else {
            let pri_len = PRI_MAX.0 - first.0 + 1;
            let ext_len = len - pri_len;
            ((first, pri_len), (EXT_MIN, ext_len))
        };

        assert!(pri.0 .0 >= PRI_MIN.0 && pri.0 .0 <= PRI_MAX.0);
        assert!(ext.0 .0 >= EXT_MIN.0 && ext.0 .0 <= EXT_MAX.0);
        assert!(self.values.len() == (pri.1 + ext.1) as usize);

        let (pri_values, ext_values) = self.values.split_at(pri.1 as usize);
        (
            ConfigPatch {
                first_address: pri.0,
                values: pri_values,
            },
            ConfigPatch {
                first_address: ext.0,
                values: ext_values,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        configs::wmbus_modecmto,
        regs::{ext::*, pri::*},
    };

    use super::*;

    #[test]
    fn can_get() {
        let config = wmbus_modecmto::<0>();
        let patch = ConfigPatch::new(&config);
        let iocfg2 = patch.get::<Iocfg2>().unwrap();
        let freqoff_cfg = patch.get::<FreqoffCfg>().unwrap();

        let (pri, ext) = patch.split_pri_ext();
        assert_eq!(iocfg2, pri.get::<Iocfg2>().unwrap());
        assert_eq!(freqoff_cfg, ext.get::<FreqoffCfg>().unwrap());
    }

    #[test]
    fn can_split_pri_ext() {
        let config = wmbus_modecmto::<0>();
        let patch = ConfigPatch::new(&config);
        let (pri, ext) = patch.split_pri_ext();
        assert_eq!(Iocfg3::ADDRESS, pri.first_address);
        assert_eq!(47, pri.values.len());
        assert_eq!(IfMixCfg::ADDRESS, ext.first_address);
        assert_eq!(58, ext.values.len());
    }
}

use crate::regs::{ext::IfMixCfg, Register, RegisterAddress};
pub struct Config(pub [u8; 105]);

impl Default for Config {
    fn default() -> Self {
        Self([0; 105])
    }
}

#[derive(Clone, Copy)]
pub struct ConfigPatch<'a> {
    pub first_address: RegisterAddress,
    pub values: &'a [u8],
}

impl<'a> ConfigPatch<'a> {
    pub const fn new(config: &'a Config) -> Self {
        ConfigPatch {
            first_address: Iocfg3::ADDRESS,
            values: &config.0,
        }
    }

    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Get a register value, or None if the register is not part of the configuration.
    pub fn get<R: Register>(&self) -> Option<R> {
        let index =
            if self.first_address.0 < IfMixCfg::ADDRESS.0 && R::ADDRESS.0 >= IfMixCfg::ADDRESS.0 {
                (0x2F - self.first_address.0 + R::ADDRESS.0 - IfMixCfg::ADDRESS.0) as usize
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

    pub const fn split(self) -> (ConfigPatch<'a>, ConfigPatch<'a>) {
        let (pri, ext) = self.first_address.split(self.values.len());
        let (pri_values, ext_values) = self.values.split_at(pri.1);
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
        regs::{
            ext::{FreqoffCfg, IfMixCfg},
            pri::{Iocfg2, Iocfg3},
            Register,
        },
    };

    #[test]
    fn can_get() {
        let config = wmbus_modecmto::<0>();
        let iocfg2 = config.get::<Iocfg2>().unwrap();
        let freqoff_cfg = config.get::<FreqoffCfg>().unwrap();

        let (pri, ext) = config.split();
        assert_eq!(iocfg2, pri.get::<Iocfg2>().unwrap());
        assert_eq!(freqoff_cfg, ext.get::<FreqoffCfg>().unwrap());
    }

    #[test]
    fn can_split() {
        let config = wmbus_modecmto::<0>();
        let (pri, ext) = config.split();
        assert_eq!(Iocfg3::ADDRESS, pri.first_address);
        assert_eq!(47, pri.values.len());
        assert_eq!(IfMixCfg::ADDRESS, ext.first_address);
        assert_eq!(58, ext.values.len());
    }
}

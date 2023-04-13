use crate::regs::{ext::IfMixCfg, pri::PktLen, Register, RegisterAddress};

#[derive(Clone, Copy)]
pub struct ConfigPatch<'a> {
    pub first_address: RegisterAddress,
    pub values: &'a [u8],
}

impl<'a> ConfigPatch<'a> {
    /// Get a register value, or None if the register is not part of the configuration.
    pub const fn get<R: ~const Register>(&self) -> Option<R> {
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

    pub const fn split(self) -> (Option<ConfigPatch<'a>>, Option<ConfigPatch<'a>>) {
        if self.first_address.0 <= PktLen::ADDRESS.0
            && self.first_address.0 + self.values.len() as u16 <= PktLen::ADDRESS.0
        {
            (Some(self), None)
        } else if self.first_address.0 >= IfMixCfg::ADDRESS.0 {
            (None, Some(self))
        } else {
            let pri_len = PktLen::ADDRESS.0 - self.first_address.0 + 1;
            let (pri, ext) = self.values.split_at(pri_len as usize);
            (
                Some(ConfigPatch {
                    first_address: self.first_address,
                    values: pri,
                }),
                Some(ConfigPatch {
                    first_address: IfMixCfg::ADDRESS,
                    values: ext,
                }),
            )
        }
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
        let pri = pri.unwrap();
        let ext = ext.unwrap();

        assert_eq!(iocfg2, pri.get::<Iocfg2>().unwrap());
        assert_eq!(freqoff_cfg, ext.get::<FreqoffCfg>().unwrap());
    }

    #[test]
    fn can_split() {
        let config = wmbus_modecmto::<0>();
        let (pri, ext) = config.split();
        let pri = pri.unwrap();
        assert_eq!(Iocfg3::ADDRESS, pri.first_address);
        assert_eq!(47, pri.values.len());

        let ext = ext.unwrap();
        assert_eq!(IfMixCfg::ADDRESS, ext.first_address);
        assert_eq!(58, ext.values.len());
    }
}

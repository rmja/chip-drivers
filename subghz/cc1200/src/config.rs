use crate::regs::Register;

#[derive(Clone, Copy)]
pub struct ConfigPatch<'a> {
    pub first_address: u16,
    pub values: &'a [u8],
}

impl<'a> ConfigPatch<'a> {
    /// Get a register value, or None if the register is not part of the configuration.
    pub const fn get<R: ~const Register>(&self) -> Option<R> {
        let index = (R::ADDRESS - self.first_address) as usize;
        let value = self.values.get(index).copied();
        if let Some(value) = value {
            Some(R::from(value))
        } else {
            None
        }
    }
}

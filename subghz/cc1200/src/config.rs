use core::ops::Index;

use crate::opcode::Reg;

#[derive(Clone, Copy)]
pub struct ConfigPatch<'a, R: Reg> {
    pub first: R,
    pub values: &'a [u8],
}

impl<R: ~const Reg> ConfigPatch<'_, R> {
    /// Get whether the configuration contains values for all registers.
    pub fn is_full(&self) -> bool {
        self.first == R::FIRST
            && R::FIRST.as_u8() as usize + self.values.len() - 1 == R::LAST.as_u8() as usize
    }

    /// Get a register value, or None if the register is not part of the configuration.
    pub const fn get(&self, reg: R) -> Option<u8> {
        let index = reg.as_u8() as usize - self.first.as_u8() as usize;
        self.values.get(index).copied()
    }
}

impl<R: ~const Reg> const Index<R> for ConfigPatch<'_, R> {
    type Output = u8;

    fn index(&self, index: R) -> &Self::Output {
        let index = index.as_u8() as usize - self.first.as_u8() as usize;
        &self.values[index]
    }
}

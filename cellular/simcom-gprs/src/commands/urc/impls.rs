use alloc::{sync::Arc, vec::Vec};
use core::{cell::Cell, fmt::Debug};

use atat::AtatResp;

use super::Data;

impl Data {
    pub fn new(data: &[u8]) -> Self {
        Self(Arc::new(Cell::new(Some(data.to_vec()))))
    }

    pub fn take(&self) -> Option<Vec<u8>> {
        self.0.take()
    }
}

impl Debug for Data {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Data {
    fn format(&self, _fmt: defmt::Formatter) {}
}

impl AtatResp for Data {}

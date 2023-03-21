use alloc::{sync::Arc, vec::Vec};
use core::fmt::Debug;
use embassy_sync::mutex::Mutex;

use atat::AtatResp;

use super::Data;

impl Data {
    pub fn new(data: &[u8]) -> Self {
        Self(Arc::new(Mutex::new(Some(data.to_vec()))))
    }

    pub fn take(&self) -> Option<Vec<u8>> {
        self.0.try_lock().unwrap().take()
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

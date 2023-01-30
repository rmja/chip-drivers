#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(impl_trait_projections)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(async_closure)]
#![feature(never_type)]
#![feature(const_mut_refs)]
#![feature(let_chains)]
#![feature(exclusive_range_pattern)]

// This mod MUST go first, so that the others see its macros.
#[macro_use]
mod fmt;

mod adapters;
pub mod atat_async;
pub mod commands;
mod device;
mod digester;
mod error;
pub mod services;

extern crate alloc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, /* Hash32, */ Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProfileId(pub u8);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ContextId(pub u8);

use atat::atat_derive::AtatLen;
pub use device::Device;
pub use digester::SimcomDigester;
pub use error::DriverError;
use serde::{Deserialize, Serialize};

#[cfg(feature = "sim900")]
pub const MAX_SOCKETS: usize = 8;
#[cfg(not(feature = "sim900"))]
pub const MAX_SOCKETS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PartNumber {
    #[cfg(feature = "sim800")]
    Sim800,
    #[cfg(feature = "sim900")]
    Sim900,
}

impl PartNumber {
    pub const fn max_sockets(&self) -> usize {
        match self {
            #[cfg(feature = "sim800")]
            PartNumber::Sim800 => 6,
            #[cfg(feature = "sim900")]
            PartNumber::Sim900 => 8,
        }
    }
}

#[cfg(all(test, feature = "defmt"))]
mod tests {
    //! This module is required in order to satisfy the requirements of defmt, while running tests.
    //! Note that this will cause all log `defmt::` log statements to be thrown away.

    #[defmt::global_logger]
    struct GlobalLogger;

    unsafe impl defmt::Logger for GlobalLogger {
        fn acquire() {}
        unsafe fn flush() {}
        unsafe fn release() {}
        unsafe fn write(_bytes: &[u8]) {}
    }

    defmt::timestamp!("");

    #[defmt::panic_handler]
    fn panic() -> ! {
        panic!()
    }
}

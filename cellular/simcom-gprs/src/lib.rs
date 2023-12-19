#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(async_closure)]
#![feature(never_type)]
#![feature(const_mut_refs)]
#![feature(let_chains)]
#![feature(exclusive_range_pattern)]
#![feature(assert_matches)]

#[macro_use]
mod fmt;

pub mod commands;
mod config;
mod device;
mod digester;
mod error;
mod ingress;
pub mod services;

extern crate alloc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProfileId(pub u8);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ContextId(pub u8);

pub type SimcomClient<'a, W, const INGRESS_BUF_SIZE: usize> =
    atat::asynch::Client<'a, W, INGRESS_BUF_SIZE>;

pub type SimcomResponseChannel<const INGRESS_BUF_SIZE: usize> =
    atat::ResponseChannel<INGRESS_BUF_SIZE>;

pub type SimcomUrcChannel = atat::UrcChannel<Urc, URC_CAPACITY, URC_SUBSCRIBERS>;
pub type SimcomUrcSubscription<'a> = atat::UrcSubscription<'a, Urc, URC_CAPACITY, URC_SUBSCRIBERS>;

use atat::atat_derive::AtatLen;
use commands::urc::Urc;
pub use config::{FlowControl, SimcomConfig};
pub use device::SimcomDevice;
use device::{URC_CAPACITY, URC_SUBSCRIBERS};
pub use digester::SimcomDigester;
pub use error::DriverError;
pub use ingress::SimcomIngress;
use serde::{Deserialize, Serialize};

pub use atat;

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
    struct TestLogger;

    unsafe impl defmt::Logger for TestLogger {
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

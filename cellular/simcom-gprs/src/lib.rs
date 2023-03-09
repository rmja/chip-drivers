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
#![feature(assert_matches)]

// This mod MUST go first, so that the others see its macros.
#[macro_use]
mod fmt;

pub mod commands;
mod device;
mod digester;
mod error;
pub mod services;

extern crate alloc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProfileId(pub u8);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, AtatLen)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ContextId(pub u8);

pub type SimcomAtatBuffers<const INGRESS_BUF_SIZE: usize, const RES_CAPACITY: usize> =
    atat::Buffers<Urc, INGRESS_BUF_SIZE, RES_CAPACITY, URC_CAPACITY, URC_SUBSCRIBERS>;

pub type SimcomAtatIngress<'a, const INGRESS_BUF_SIZE: usize, const RES_CAPACITY: usize> =
    atat::Ingress<
        'a,
        SimcomDigester,
        Urc,
        INGRESS_BUF_SIZE,
        RES_CAPACITY,
        URC_CAPACITY,
        URC_SUBSCRIBERS,
    >;

pub type SimcomAtatUrcChannel<const INGRESS_BUF_SIZE: usize> =
    atat::UrcChannel<Urc, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS>;

use atat::atat_derive::AtatLen;
use commands::urc::Urc;
pub use device::Device;
use device::{URC_CAPACITY, URC_SUBSCRIBERS};
pub use digester::SimcomDigester;
pub use error::DriverError;
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

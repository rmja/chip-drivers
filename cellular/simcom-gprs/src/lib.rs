#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, feature(assert_matches))]
#![cfg_attr(test, feature(type_alias_impl_trait))]

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

pub type SimcomClient<'a, W, const N: usize> = atat::asynch::Client<'a, W, N>;
pub type SimcomResponseSlot<const N: usize> = atat::ResponseSlot<N>;
pub type SimcomUrcChannel = atat::UrcChannel<Urc, URC_CAPACITY, URC_SUBSCRIBERS>;
pub type SimcomUrcSubscription<'a> = atat::UrcSubscription<'a, Urc, URC_CAPACITY, URC_SUBSCRIBERS>;

pub const CLIENT_BUF_SIZE: usize = <commands::tcpip::WriteData as atat::AtatCmd>::MAX_LEN;

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

// It seems as if the type export of SimcomUrcChannel does not work - it causes a rustc ICE in nightly 2024-12-10
// We therefore export the fundamentals for creating the channel manually
// See https://github.com/rust-lang/rust/issues/133808
pub mod rustc_ice_workaround {
    pub type Urc = crate::commands::urc::Urc;
    pub const URC_CAPACITY: usize = crate::device::URC_CAPACITY;
    pub const URC_SUBSCRIBERS: usize = crate::device::URC_SUBSCRIBERS;
}

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

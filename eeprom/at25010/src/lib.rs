#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(inherent_associated_types)]

mod driver;
mod error;
mod opcode;

#[derive(Clone, Copy)]
pub enum PartNumber {
    At25010,
    At25020,
    At25040,
    At25010b,
    At25020b,
    At25040b,
}

pub use driver::{Driver, StatefulDriver};
pub use error::Error;

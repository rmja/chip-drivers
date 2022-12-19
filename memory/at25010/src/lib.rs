#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod traits;
mod opcode;
mod driver;

#[derive(Clone, Copy)]
pub enum PartNumber {
    At25010,
    At25020,
    At25040,
    At25010b,
    At25020b,
    At25040b,
}

pub use driver::Driver;
#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait, impl_trait_projections)]

extern crate alloc;

mod driver;
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

pub use driver::Driver;

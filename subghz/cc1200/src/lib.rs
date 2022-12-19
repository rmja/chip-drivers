#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate bitfield;

#[macro_use]
extern crate num_derive;

mod config;
mod driver;
mod errors;
pub mod gpio;
mod opcode;
pub mod regs;
mod statusbyte;
pub mod traits;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PartNumber {
    Cc1200,
    Cc1201,
}

pub type Rssi = i8;

pub use self::{
    config::ConfigPatch,
    driver::Driver,
    errors::*,
    opcode::Strobe,
    statusbyte::{State, StatusByte},
};

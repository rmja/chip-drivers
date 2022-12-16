#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate bitfield;

mod config;
mod driver;
mod errors;
mod gpio;
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
    gpio::Gpio,
};

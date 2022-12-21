#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

extern crate alloc;
extern crate bitfield;

#[macro_use]
extern crate num_derive;

mod config;
mod driver;
pub mod gpio;
mod opcode;
pub mod regs;
mod statusbyte;
pub mod traits;
mod error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PartNumber {
    Cc1200,
    Cc1201,
}

pub type Rssi = i8;

pub const RX_FIFO_SIZE: usize = 128;
pub const TX_FIFO_SIZE: usize = 128;

pub use self::{
    config::ConfigPatch,
    driver::Driver,
    error::DriverError,
    opcode::{ExtReg, PriReg, Reg, Strobe},
    statusbyte::{State, StatusByte},
};

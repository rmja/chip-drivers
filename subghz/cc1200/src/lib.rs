#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(const_option)]
#![feature(const_slice_index)]
#![feature(const_trait_impl)]
#![feature(inherent_associated_types)]
#![feature(let_chains)]
#![feature(async_fn_in_trait)]
#![feature(const_slice_split_at_not_mut)]

extern crate alloc;
extern crate bitfield;

#[macro_use]
extern crate num_derive;

mod config;
mod driver;
mod error;
pub mod gpio;
mod opcode;
pub mod regs;
mod statusbyte;

#[cfg(feature = "ctrl")]
pub mod controllers;
pub mod configs;

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
    opcode::Strobe,
    statusbyte::{State, StatusByte},
};

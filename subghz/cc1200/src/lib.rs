#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]
#![allow(incomplete_features)]
#![feature(const_option)]
#![feature(const_slice_index)]
#![feature(const_trait_impl)]
#![feature(inherent_associated_types)]
#![feature(let_chains)]
#![cfg_attr(feature = "serial-controller", feature(coroutines))]
#![cfg_attr(test, feature(type_alias_impl_trait))]

extern crate bitfield;

#[macro_use]
extern crate num_derive;

mod config;
mod driver;
mod error;
pub mod gpio;
pub mod regs;
mod statusbyte;

mod cmd;
pub mod configs;
pub mod controllers;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PartNumber {
    Cc1200,
    Cc1201,
}

pub type Rssi = i16;

pub const RX_FIFO_SIZE: usize = 128;
pub const TX_FIFO_SIZE: usize = 128;

pub use self::{
    config::{Config, ConfigPatch},
    driver::{CalibrationValue, Driver},
    error::DriverError,
    statusbyte::{State, StatusByte},
};

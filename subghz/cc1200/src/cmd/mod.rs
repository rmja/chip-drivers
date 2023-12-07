use crate::StatusByte;

mod burst;
mod single;
mod strobe;

const SINGLE_WRITE: u8 = 0x00;
const BURST_WRITE: u8 = 0x40;
const SINGLE_READ: u8 = 0x80;
const BURST_READ: u8 = 0xC0;
const FIFO: u8 = 0x3F;
const EXTENDED_ADDRESS: u8 = 0x2F;

pub trait Command {
    fn len(&self) -> usize;
}

pub trait Response {
    fn status_byte(&self) -> StatusByte;
}

pub use {
    burst::BurstHeader,
    single::SingleCommand,
    strobe::{Strobe, StrobeCommand},
};

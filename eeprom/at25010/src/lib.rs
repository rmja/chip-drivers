#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait, impl_trait_projections)]
#![feature(inherent_associated_types)]

extern crate alloc;

mod driver;
#[cfg(feature = "embedded-io")]
pub mod embeddedio;
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

// See https://stackoverflow.com/questions/63622942/how-can-i-implement-fromsome-traits-associated-type
pub enum Error<Spi: embedded_hal_async::spi::Error, T: embedded_hal_async::delay::DelayUs> {
    WriteProtection,
    Capacity,
    Spi(Spi),
    Delay(<T as embedded_hal_async::delay::DelayUs>::Error),
}

impl<Spi, T> core::fmt::Debug for crate::Error<Spi, T>
where
    Spi: embedded_hal_async::spi::Error,
    T: embedded_hal_async::delay::DelayUs,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::WriteProtection => write!(f, "WriteProtection"),
            Self::Capacity => write!(f, "Capacity"),
            Self::Spi(arg0) => f.debug_tuple("Spi").field(arg0).finish(),
            Self::Delay(arg0) => f.debug_tuple("Delay").field(arg0).finish(),
        }
    }
}

pub use driver::Driver;

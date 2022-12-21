// See https://stackoverflow.com/questions/63622942/how-can-i-implement-fromsome-traits-associated-type
pub enum DriverError<Spi: embedded_hal_async::spi::Error, T: embedded_hal_async::delay::DelayUs> {
    WriteProtection,
    Capacity,
    Spi(Spi),
    Delay(<T as embedded_hal_async::delay::DelayUs>::Error),
}

impl<Spi, T> core::fmt::Debug for crate::DriverError<Spi, T>
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

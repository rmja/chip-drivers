pub enum DriverError<SpiDeviceError: embedded_hal_async::spi::Error, T: embedded_hal_async::delay::DelayUs> {
    Timeout,
    RxFifoOverflow,
    TxFifoUnderflow,
    InvalidPartNumber,
    InvalidRssi,
    Spi(SpiDeviceError),
    Delay(<T as embedded_hal_async::delay::DelayUs>::Error),
}

// Explicit implementation of Debug because DelayUs may not implement Debug even thoug DelayUs::Error does.
impl<Spi, T> core::fmt::Debug for DriverError<Spi, T>
where
    Spi: embedded_hal_async::spi::Error,
    T: embedded_hal_async::delay::DelayUs,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Timeout => write!(f, "Timeout"),
            Self::RxFifoOverflow => write!(f, "RxFifoOverflow"),
            Self::TxFifoUnderflow => write!(f, "TxFifoUnderflow"),
            Self::InvalidPartNumber => write!(f, "InvalidPartNumber"),
            Self::InvalidRssi => write!(f, "InvalidRssi"),
            Self::Spi(arg0) => f.debug_tuple("Spi").field(arg0).finish(),
            Self::Delay(arg0) => f.debug_tuple("Delay").field(arg0).finish(),
        }
    }
}

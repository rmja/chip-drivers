#[derive(Debug)]
pub enum DriverError<Spi: embedded_hal_async::spi::Error, T: embedded_hal_async::delay::DelayUs> {
    Timeout,
    RxFifoOverflow,
    TxFifoUnderflow,
    InvalidPartNumber,
    InvalidRssi,
    Spi(Spi),
    Delay(<T as embedded_hal_async::delay::DelayUs>::Error),
}

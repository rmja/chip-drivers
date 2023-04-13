#[derive(Debug)]
pub enum DriverError {
    Timeout,
    InvalidPartNumber,
    InvalidRssi,
    Spi,
}

impl<SpiError> From<SpiError> for DriverError
where
    SpiError: embedded_hal_async::spi::Error,
{
    fn from(_value: SpiError) -> Self {
        Self::Spi
    }
}

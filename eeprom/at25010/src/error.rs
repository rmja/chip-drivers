#[derive(Debug)]
pub enum DriverError {
    WriteProtection,
    Capacity,
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

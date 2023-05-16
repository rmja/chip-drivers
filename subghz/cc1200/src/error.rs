#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DriverError {
    Timeout,
    InvalidPartNumber,
    Spi,
}

impl<SpiError> From<SpiError> for DriverError
where
    SpiError: embedded_hal_async::spi::Error,
{
    fn from(_value: SpiError) -> Self {
        DriverError::Spi
    }
}

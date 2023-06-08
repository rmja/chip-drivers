use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[derive(Debug)]
pub enum Error {
    NotAligned,
    OutOfBounds,
    WriteProtection,
    Spi,
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match *self {
            Error::NotAligned => NorFlashErrorKind::NotAligned,
            Error::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            _ => NorFlashErrorKind::Other,
        }
    }
}

impl<SpiError> From<SpiError> for Error
where
    SpiError: embedded_hal_async::spi::Error,
{
    fn from(_value: SpiError) -> Self {
        Self::Spi
    }
}

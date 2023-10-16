use embedded_hal_async::spi;
use mockall::mock;

#[derive(Debug, Clone, Copy)]
pub struct SpiError;

impl spi::Error for SpiError {
    fn kind(&self) -> spi::ErrorKind {
        spi::ErrorKind::Other
    }
}

mock! {
    #[derive(Debug)]
    pub SpiDevice<Word: Copy + 'static = u8> { }

    impl<Word: Copy + 'static> spi::SpiDevice<Word> for SpiDevice<Word> {
        async fn transaction<'a>(&mut self,operations: &mut [spi::Operation<'a, Word>]) -> Result<(), SpiError>;
    }

    impl<Word: Copy + 'static> spi::ErrorType for SpiDevice<Word> {
        type Error = SpiError;
    }
}

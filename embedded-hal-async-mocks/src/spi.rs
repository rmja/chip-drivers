use embedded_hal_async::spi;
use mockall::mock;

#[derive(Debug, Clone, Copy)]
pub struct SpiError;
impl spi::Error for SpiError {
    fn kind(&self) -> spi::ErrorKind {
        todo!()
    }
}

#[derive(Debug, Default)]
pub struct MockSpiDevice {
    pub bus: MockSpiBus,
    transactions: usize,
}

impl MockSpiDevice {
    pub fn new() -> Self {
        Self {
            bus: MockSpiBus::new(),
            transactions: 0,
        }
    }

    pub fn transactions(&self) -> usize {
        self.transactions
    }
}

unsafe impl spi::SpiDevice for MockSpiDevice {
    type Bus = MockSpiBus;

    async fn transaction<R, F, Fut>(&mut self, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(*mut Self::Bus) -> Fut,
        Fut: futures::Future<Output = Result<R, <Self::Bus as spi::ErrorType>::Error>>,
    {
        self.transactions += 1;
        f(&mut self.bus).await
    }
}

impl spi::ErrorType for MockSpiDevice {
    type Error = SpiError;
}

mock! {
    #[derive(Debug)]
    pub SpiBus { }

    impl spi::SpiBus for SpiBus {
        async fn transfer<'a>(
            &'a mut self,
            read: &'a mut [u8],
            write: &'a [u8],
        ) -> Result<(), SpiError>;

        async fn transfer_in_place<'a>(&'a mut self, words: &'a mut [u8]) -> Result<(), SpiError>;
    }

    impl spi::SpiBusRead for SpiBus {
        async fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError>;
    }

    impl spi::SpiBusWrite for SpiBus {
        async fn write(&mut self, words: &[u8]) -> Result<(), SpiError>;
    }

    impl spi::SpiBusFlush for SpiBus {
        async fn flush(&mut self) -> Result<(), SpiError>;
    }

    impl spi::ErrorType for SpiBus {
        type Error = SpiError;
    }
}

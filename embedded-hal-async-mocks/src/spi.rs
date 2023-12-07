use embedded_hal_async::spi::{self, Operation};
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

impl MockSpiDevice<u8> {
    pub fn expect_transaction_operations(&mut self, expected: &'static [Operation<'static, u8>]) {
        self.expect_transaction()
            .withf(move |transaction| {
                if transaction.len() != expected.len() {
                    return false;
                }
                for (actual, expected) in transaction.iter().zip(expected) {
                    if !Self::is_match(actual, expected) {
                        return false;
                    }
                }

                true
            })
            .returning(move |transaction| {
                for (dest, src) in transaction.iter_mut().zip(expected) {
                    Self::assign(dest, src);
                }
                Ok(())
            })
            .times(1);
    }

    fn is_match(x: &Operation<'_, u8>, y: &Operation<'_, u8>) -> bool {
        if let Operation::Read(x) = x
            && let Operation::Read(y) = y
        {
            x.len() == y.len()
        } else if let Operation::Write(x) = x
            && let Operation::Write(y) = y
        {
            x == y
        } else if let Operation::Transfer(_, x) = x
            && let Operation::Transfer(_, y) = y
        {
            x == y
        } else {
            false
        }
    }

    fn assign(dest: &mut Operation<'_, u8>, src: &Operation<'_, u8>) {
        if let Operation::Read(dest) = dest
            && let Operation::Read(src) = src
        {
            dest.copy_from_slice(src)
        } else if let Operation::Transfer(dest, _) = dest
            && let Operation::Transfer(src, _) = src
        {
            dest.copy_from_slice(src)
        }
    }
}

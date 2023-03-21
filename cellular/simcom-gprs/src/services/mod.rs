pub mod data;
pub mod network;

#[cfg(test)]
mod serial_mock {
    use core::convert::Infallible;

    use alloc::vec::Vec;
    use embassy_sync::{
        blocking_mutex::raw::CriticalSectionRawMutex,
        pubsub::{PubSubChannel, Publisher, Subscriber},
    };

    pub struct SerialMock(PubSubChannel<CriticalSectionRawMutex, Vec<u8>, 1, 1, 1>);

    pub struct TxMock<'a> {
        buf: Vec<u8>,
        publisher: Publisher<'a, CriticalSectionRawMutex, Vec<u8>, 1, 1, 1>,
    }

    pub type RxMock<'a> = Subscriber<'a, CriticalSectionRawMutex, Vec<u8>, 1, 1, 1>;

    impl SerialMock {
        pub const fn new() -> Self {
            Self(PubSubChannel::new())
        }

        pub fn split<'a>(&'a self) -> (TxMock<'a>, RxMock<'a>) {
            (
                TxMock::new(self.0.publisher().unwrap()),
                self.0.subscriber().unwrap(),
            )
        }
    }

    impl<'a> TxMock<'a> {
        fn new(publisher: Publisher<'a, CriticalSectionRawMutex, Vec<u8>, 1, 1, 1>) -> Self {
            TxMock {
                buf: Vec::new(),
                publisher,
            }
        }
    }

    impl embedded_io::Io for TxMock<'_> {
        type Error = Infallible;
    }

    impl embedded_io::blocking::Write for TxMock<'_> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.buf.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.publisher.try_publish(self.buf.clone()).unwrap();
            self.buf.clear();
            Ok(())
        }
    }

    impl embedded_io::asynch::Write for TxMock<'_> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.buf.extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.publisher.try_publish(self.buf.clone()).unwrap();
            self.buf.clear();
            Ok(())
        }
    }
}

#[cfg(test)]
pub mod tokio {
    use core::{convert::Infallible, time::Duration};

    use embedded_hal_async::delay::DelayUs;

    #[derive(Clone)]
    pub struct TokioDelay;

    impl DelayUs for TokioDelay {
        type Error = Infallible;

        async fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
            tokio::time::sleep(Duration::from_micros(us as u64)).await;
            Ok(())
        }

        async fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
            tokio::time::sleep(Duration::from_millis(ms as u64)).await;
            Ok(())
        }
    }
}

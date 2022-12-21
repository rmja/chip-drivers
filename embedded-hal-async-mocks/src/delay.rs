use embedded_hal_async::delay;
use mockall::mock;

#[derive(Debug, Clone, Copy)]
pub struct DelayError;

mock! {
    #[derive(Debug)]
    pub Delay {}

    impl delay::DelayUs for Delay {
        type Error = DelayError;

        async fn delay_us(&mut self, us: u32) -> Result<(), DelayError>;
        async fn delay_ms(&mut self, ms: u32) -> Result<(), DelayError>;
    }
}

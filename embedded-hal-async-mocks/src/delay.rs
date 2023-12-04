use embedded_hal_async::delay;
use mockall::mock;

mock! {
    #[derive(Debug)]
    pub Delay {}

    impl delay::DelayNs for Delay {
        async fn delay_ns(&mut self, us: u32);
        async fn delay_us(&mut self, us: u32);
        async fn delay_ms(&mut self, ms: u32);
    }
}

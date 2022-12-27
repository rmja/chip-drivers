pub trait IrqPin<Timestamp> {
    async fn wait_for_high(&mut self) -> Timestamp;
    async fn wait_for_low(&mut self);
}

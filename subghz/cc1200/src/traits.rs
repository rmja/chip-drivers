use alloc::boxed::Box;
use async_trait::async_trait;

#[async_trait]
pub trait Timer : Send {
    async fn sleep_millis(&self, millis: u32);
}

#[async_trait]
pub trait Spi : Send {
    fn select(&mut self);
    fn deselect(&mut self);
    async fn read(&mut self, buffer: &mut [u8]);
    async fn write(&mut self, buffer: &[u8]);
    async fn transfer(&mut self, tx_buffer: &[u8], rx_buffer: &mut [u8]);
    async fn miso_wait_low(&mut self);
}

#[async_trait]
pub trait Pins : Send {
    fn set_reset(&mut self);
    fn clear_reset(&mut self);
    async fn miso_wait_low(&mut self);
}

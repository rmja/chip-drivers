use async_trait::async_trait;
use alloc::boxed::Box;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Spi {
    fn select(&mut self);
    fn deselect(&mut self);
    async fn read(&mut self, rx_buffer: &mut [u8]);
    async fn write(&mut self, tx_buffer: &[u8]);
    async fn transfer(&mut self, tx_buffer: &[u8], rx_buffer: &mut [u8]);
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Timer {
    async fn sleep_nanos(&self, nanos: u32);
    async fn sleep_micros(&self, micros: u32);
}
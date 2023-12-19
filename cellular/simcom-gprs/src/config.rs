use atat::Config;
use embassy_time::{Duration, Instant};
use embedded_hal::digital::OutputPin;

pub trait SimcomConfig {
    type ResetPin: OutputPin;

    const FLOW_CONTROL: FlowControl = FlowControl::None;

    fn reset_pin(&mut self) -> &mut Self::ResetPin;

    fn atat_config(&self) -> Config {
        Config::new().get_response_timeout(Self::get_response_timeout)
    }

    fn get_response_timeout(start: Instant, timeout: Duration) -> Instant {
        start + timeout
    }
}

pub enum FlowControl {
    /// No flow control is being used
    None,
    /// Hardware flow control
    RtsCts,
}

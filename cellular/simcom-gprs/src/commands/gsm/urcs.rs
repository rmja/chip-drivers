pub use super::PinStatusCode;
use atat::atat_derive::AtatResp;

/// 3.2.28 AT+CPIN Enter PIN
#[derive(AtatResp, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PinStatus {
    #[at_arg(position = 0)]
    pub code: PinStatusCode,
}

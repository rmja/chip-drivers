use atat::atat_derive::AtatResp;
use heapless::String;
use heapless_bytes::Bytes;

use super::{NetworkRegistrationStat, NetworkRegistrationUrcConfig};

/// 3.2.8 Manufacturer Identification
#[derive(AtatResp)]
pub struct ManufacturerIdResponse {
    pub manufacturer: Bytes<16>,
}

/// 3.2.9 Manufacturer Model
#[derive(AtatResp)]
pub struct ModelIdResponse {
    pub model: Bytes<16>,
}

/// 3.2.10 Request Revision Identification of Software Release
#[derive(AtatResp)]
pub struct SoftwareVersionResponse {
    pub version: Bytes<32>,
}

// 3.2.32 AT+CREG Network Registration
#[derive(AtatResp)]
pub struct NetworkRegistrationStatus {
    #[at_arg(position = 0)]
    pub n: NetworkRegistrationUrcConfig,
    #[at_arg(position = 1)]
    pub stat: NetworkRegistrationStat,
    #[at_arg(position = 2)]
    pub lac: Option<String<4>>,
    #[at_arg(position = 3)]
    pub ci: Option<String<8>>,
    #[at_arg(position = 4)]
    pub act_status: Option<u8>,
}

// 3.2.35 AT+CSQ Signal Quality Report
#[derive(AtatResp)]
pub struct SignalQuality {
    #[at_arg(position = 0)]
    rssi: u8,
    #[at_arg(position = 1)]
    pub ber: u8,
}

impl SignalQuality {
    pub fn rssi(&self) -> Option<i8> {
        match self.rssi {
            0 => Some(-115),
            1 => Some(-111),
            2..=32 => Some(-110 + 2 * (self.rssi - 2) as i8),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SignalQuality;

    #[test]
    fn test_rssi() {
        assert_eq!(Some(-115), SignalQuality { rssi: 0, ber: 0 }.rssi());
        assert_eq!(Some(-111), SignalQuality { rssi: 1, ber: 0 }.rssi());
        assert_eq!(Some(-110), SignalQuality { rssi: 2, ber: 0 }.rssi());
        assert_eq!(Some(-108), SignalQuality { rssi: 3, ber: 0 }.rssi());
        assert_eq!(Some(-54), SignalQuality { rssi: 30, ber: 0 }.rssi());
        assert_eq!(Some(-52), SignalQuality { rssi: 31, ber: 0 }.rssi());
        assert_eq!(None, SignalQuality { rssi: 99, ber: 0 }.rssi());
    }
}

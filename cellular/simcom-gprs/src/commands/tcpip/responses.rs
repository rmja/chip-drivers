use atat::atat_derive::AtatResp;
use heapless::String;
use heapless_bytes::Bytes;

use super::types::*;

/// 8.2.6 AT+CIPCLOSE Close TCP or UDP Connection.
#[derive(AtatResp)]
pub struct CloseOk {
    pub id: usize,
}

/// 8.2.11 AT+CIFSR Get Local IP Address
#[derive(AtatResp)]
pub struct LocalIP {
    pub ip: Bytes<15>,
}

/// 8.2.12 AT+CIPSTATUS Query Current Connection Status
#[derive(AtatResp)]
pub struct ConnectionStatus {
    pub id: u8,
    _bearer: Bytes<1>,
    pub mode: String<3>,
    pub ip: String<15>,
    pub port: String<5>,
    pub state: ClientState,
}

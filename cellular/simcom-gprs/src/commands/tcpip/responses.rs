use atat::atat_derive::AtatResp;
use heapless::String;
use heapless_bytes::Bytes;

use crate::MAX_SOCKETS;

use super::types::*;

/// 8.2.3 AT+CIPSEND Send Data Through TCP or UDP Connection
#[derive(AtatResp)]
pub struct SendBufferSize {
    pub size: [usize; MAX_SOCKETS],
}

#[derive(AtatResp)]
pub struct DataAccept {
    pub id: usize,
    pub accepted: usize,
}

/// 8.2.5 AT+CIPACK Query Previous Connection Data Transmitting State
#[derive(AtatResp)]
pub struct DataTransmittingState {
    /// The data amount which has been sent
    pub txlen: usize,
    /// The data amount confirmed successfully by the server
    pub acklen: usize,
    /// The data amount without confirmation by the server
    pub nacklen: usize,
}

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

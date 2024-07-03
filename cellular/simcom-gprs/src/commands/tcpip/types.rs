use atat::atat_derive::AtatEnum;
use serde::Deserialize;

#[derive(Debug, AtatEnum, PartialEq)]
pub enum MultiIpValue {
    SingleIpConnection = 0,
    MultiIpConnection = 1,
}

#[derive(Debug, AtatEnum, PartialEq)]
pub enum DataTransmittingMode {
    NormalMode = 0,
    QuickSendMode = 1,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ClientState {
    #[serde(rename = "INITIAL")]
    Initial,
    #[serde(rename = "CONNECTING")]
    Connecting,
    #[serde(rename = "CONNECTED")]
    Connected,
    #[serde(rename = "REMOTE CLOSING")]
    RemoteClosing,
    #[serde(rename = "CLOSING")]
    Closing,
    #[serde(rename = "CLOSED")]
    Closed,
}

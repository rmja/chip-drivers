use atat::atat_derive::AtatEnum;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, AtatEnum)]
pub enum MultiIpValue {
    SingleIpConnection = 0,
    MultiIpConnection = 1,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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

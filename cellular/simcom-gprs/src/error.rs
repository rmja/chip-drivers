use crate::services::{data::SocketError, network::NetworkError};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DriverError {
    BaudDetection,
    UnsupportedManufacturer,
    UnsupportedModel,
    Atat(atat::Error),
    AlreadyTaken,
    Network(NetworkError),
    Socket(SocketError),
}

impl From<atat::Error> for DriverError {
    fn from(value: atat::Error) -> Self {
        DriverError::Atat(value)
    }
}

impl From<NetworkError> for DriverError {
    fn from(value: NetworkError) -> Self {
        match value {
            NetworkError::Atat(atat) => DriverError::Atat(atat),
            other => DriverError::Network(other),
        }
    }
}

impl From<SocketError> for DriverError {
    fn from(value: SocketError) -> Self {
        match value {
            SocketError::Atat(atat) => DriverError::Atat(atat),
            other => DriverError::Socket(other),
        }
    }
}

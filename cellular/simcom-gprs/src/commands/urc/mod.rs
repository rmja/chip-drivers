mod complete;
mod streaming;

use atat::{
    atat_derive::{AtatResp, AtatUrc},
    digest::parser::urc_helper,
    nom::branch,
    AtatUrc,
};
use heapless::String;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Urc {
    ConnectOk(usize),
    ConnectFail(usize),
    AlreadyConnect(usize),
    SendOk(usize),
    Closed(usize),
    IpLookup(HostIp),
    DataAvailable(usize),
    ReadData(ReadResult),
}

#[derive(Debug, Clone, AtatUrc)]
enum UrcInner {
    #[at_urc("+CDNSGIP")]
    IpLookup(HostIp),
}

/// 8.2.14 AT+CDNSGIP Query the IP Address of Given Domain Name
#[derive(Debug, Clone, AtatResp, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HostIp {
    success: u8,
    pub host: String<128>,
    pub ip: String<15>,
    pub alt_ip: Option<String<15>>,
}

/// 8.2.26 AT+CIPRXGET Get Data from Network Manually
#[derive(Debug, Clone, AtatResp, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadResult {
    pub id: usize,
    pub data_len: usize,
    pub pending_len: usize,
}

impl From<UrcInner> for Urc {
    fn from(value: UrcInner) -> Self {
        match value {
            UrcInner::IpLookup(x) => Urc::IpLookup(x),
        }
    }
}

impl AtatUrc for Urc {
    type Response = Urc;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        if let Some(urc) = complete::parse_connection_status(resp) {
            Some(urc)
        } else if let Some(urc) = complete::parse_data_available(resp) {
            Some(urc)
        } else if let Some(urc) = complete::parse_read_data(resp) {
            Some(urc)
        } else {
            UrcInner::parse(resp).map(|x| x.into())
        }
    }
}

impl atat::Parser for Urc {
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), atat::digest::ParseError> {
        let (_, r) = branch::alt((
            streaming::parse_connection_status,
            streaming::parse_data_available,
            streaming::parse_read_data,
            streaming::parse_receive,
            urc_helper("+CDNSGIP"),
        ))(buf)?;
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use atat::{DigestResult, Digester};

    use crate::SimcomDigester;

    use super::*;

    #[test]
    fn can_parse_connect_ok() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"2, CONNECT OK"), 17),
            digester.digest(b"\r\n2, CONNECT OK\r\n")
        );
        let urc = Urc::parse(b"2, CONNECT OK").unwrap();
        assert_eq!(Urc::ConnectOk(2), urc);
    }

    #[test]
    fn can_parse_ip_lookup() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (
                DigestResult::Urc(b"+CDNSGIP: 1,\"utiliread.dk\",\"123.123.123.123\""),
                48
            ),
            digester.digest(b"\r\n+CDNSGIP: 1,\"utiliread.dk\",\"123.123.123.123\"\r\n")
        );
        let urc = Urc::parse(b"+CDNSGIP: 1,\"utiliread.dk\",\"123.123.123.123\"").unwrap();
        assert_eq!(
            Urc::IpLookup(HostIp {
                success: 1,
                host: String::from("utiliread.dk"),
                ip: String::from("123.123.123.123"),
                alt_ip: None
            }),
            urc
        );
    }

    #[test]
    fn can_parse_data_available() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET: 1,2"), 18),
            digester.digest(b"\r\n+CIPRXGET: 1,2\r\n")
        );
        let urc = Urc::parse(b"+CIPRXGET: 1,2").unwrap();
        assert_eq!(Urc::DataAvailable(2), urc);
    }

    #[test]
    fn can_parse_read_data() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n"), 30),
            digester.digest(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
        );
        let urc = Urc::parse(b"+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n").unwrap();
        assert_eq!(
            Urc::ReadData(ReadResult {
                id: 5,
                data_len: 8,
                pending_len: 0
            }),
            urc
        );
    }
}

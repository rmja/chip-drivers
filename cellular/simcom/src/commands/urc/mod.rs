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
    Receive(Receive),
    IpLookup(HostIp),
    DataAvailable(usize),
}

#[derive(Debug, Clone, AtatUrc)]
enum UrcInner {
    #[at_urc("+CDNSGIP")]
    IpLookup(HostIp),
}

/// 19.3 Summary of Unsolicited Result Codes
#[derive(Debug, Clone, AtatResp, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Receive {
    pub id: usize,
    pub len: usize,
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
        } else if let Some(urc) = complete::parse_receive(resp) {
            Some(urc)
        } else if let Some(urc) = complete::parse_data_available(resp) {
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
            streaming::parse_receive,
            streaming::parse_data_available,
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
    fn can_parse_receive() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+RECEIVE,2,1234:"), 20),
            digester.digest(b"\r\n+RECEIVE,2,1234:\r\nHTTP\r\n")
        );
        let urc = Urc::parse(b"+RECEIVE,2,1234:").unwrap();
        assert_eq!(Urc::Receive(Receive { id: 2, len: 1234 }), urc);
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
}

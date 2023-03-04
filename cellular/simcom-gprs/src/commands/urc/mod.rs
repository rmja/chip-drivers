mod complete;
mod impls;
mod streaming;

use core::cell::Cell;

use alloc::{sync::Arc, vec::Vec};
use atat::{
    atat_derive::{AtatResp, AtatUrc},
    digest::parser::urc_helper,
    nom::branch,
    AtatUrc,
};
use heapless::String;

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HostIp {
    _success: u8,
    pub host: String<128>,
    pub ip: String<15>,
    pub alt_ip: Option<String<15>>,
}

/// 8.2.26 AT+CIPRXGET Get Data from Network Manually
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadResult {
    pub id: usize,
    pub data_len: usize,
    pub pending_len: usize,
    pub data: Data,
}

#[derive(Clone)]
pub struct Data(Arc<Cell<Option<Vec<u8>>>>);

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
    use core::assert_matches::assert_matches;

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
        assert_matches!(urc, Urc::ConnectOk(2));
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

        if let Urc::IpLookup(urc) = urc {
            assert_eq!(1, urc._success);
            assert_eq!("utiliread.dk", urc.host);
            assert_eq!("123.123.123.123", urc.ip);
            assert_eq!(None, urc.alt_ip);
        } else {
            panic!("Invalid URC");
        }
    }

    #[test]
    fn can_parse_data_available() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET: 1,2"), 18),
            digester.digest(b"\r\n+CIPRXGET: 1,2\r\n")
        );
        let urc = Urc::parse(b"+CIPRXGET: 1,2").unwrap();
        assert_matches!(urc, Urc::DataAvailable(2));
    }

    #[test]
    fn can_parse_read_data() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n"), 30),
            digester.digest(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
        );
        let urc = Urc::parse(b"+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n").unwrap();
        if let Urc::ReadData(data) = urc {
            assert_eq!(5, data.id);
            assert_eq!(8, data.data_len);
            assert_eq!(0, data.pending_len);
            assert_eq!(b"HTTP\r\n\r\n", data.data.take().unwrap().as_slice());
        } else {
            panic!("Invalid URC");
        }
    }
}

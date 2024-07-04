mod complete;
mod impls;
mod streaming;

use alloc::{sync::Arc, vec::Vec};
use atat::{
    atat_derive::{AtatResp, AtatUrc},
    digest::parser::urc_helper,
    nom::branch,
    AtatUrc,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::String;

use crate::ContextId;

use super::{gprs, gsm};

pub use gsm::urcs::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Urc {
    CallReady,
    SmsReady,
    PinStatus(PinStatus),
    ConnectOk(usize),
    ConnectFail(usize),
    AlreadyConnect(usize),
    Closed(usize),
    PdpDeact,

    PdbState(PdpContextState),

    /// +CDNSGIP: ...
    DnsResult(Result<DnsLookup, usize>),

    /// +CIPRXGET: 1,...
    DataAvailable(usize),

    /// +CIPRXGET: 2,...
    ReadData(ReadResult),
}

#[derive(Debug, Clone, AtatUrc)]
enum UrcInner {
    #[at_urc("Call Ready")]
    CallReady,
    #[at_urc("SMS Ready")]
    SmsReady,
    #[at_urc("+CPIN")]
    PinStatus(PinStatus),
    #[at_urc("+CDNSGIP")]
    DnsOk(DnsLookup),
}

/// 7.2.5 AT+CGACT PDP Context Activate or Deactivate
#[derive(Debug, Clone, AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PdpContextState {
    pub cid: ContextId,
    pub state: gprs::PdpState,
}

/// 8.2.14 AT+CDNSGIP Query the IP Address of Given Domain Name
#[derive(Debug, Clone, AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DnsLookup {
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
pub struct Data(Arc<Mutex<CriticalSectionRawMutex, Option<Vec<u8>>>>);

impl From<UrcInner> for Urc {
    fn from(value: UrcInner) -> Self {
        match value {
            UrcInner::CallReady => Urc::CallReady,
            UrcInner::SmsReady => Urc::SmsReady,
            UrcInner::PinStatus(x) => Urc::PinStatus(x),
            UrcInner::DnsOk(x) => Urc::DnsResult(Ok(x)),
        }
    }
}

impl AtatUrc for Urc {
    type Response = Urc;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        if let Some(urc) = complete::parse_pdp_state(resp) {
            Some(urc)
        } else if let Some(urc) = complete::parse_connection_status(resp) {
            Some(urc)
        } else if let Some(urc) = complete::parse_data_available(resp) {
            Some(urc)
        } else if let Some(urc) = complete::parse_read_data(resp) {
            Some(urc)
        } else if let Some(urc) = complete::parse_dns_error(resp) {
            Some(urc)
        } else if resp == b"+PDP: DEACT" {
            Some(Urc::PdpDeact)
        } else {
            UrcInner::parse(resp).map(|x| x.into())
        }
    }
}

impl atat::Parser for Urc {
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), atat::digest::ParseError> {
        let (_, r) = branch::alt((
            streaming::parse_pdp_state,
            streaming::parse_connection_status,
            streaming::parse_data_available,
            streaming::parse_read_data,
            streaming::parse_receive,
            urc_helper("Call Ready"),
            urc_helper("SMS Ready"),
            urc_helper("+PDP: DEACT"),
            urc_helper("+CPIN"),
            urc_helper("+CGACT"),
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
    fn can_parse_call_ready() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"Call Ready"), 14),
            digester.digest(b"\r\nCall Ready\r\n")
        );
        let urc = Urc::parse(b"Call Ready").unwrap();
        assert_matches!(urc, Urc::CallReady);
    }

    #[test]
    fn can_parse_sms_ready() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"SMS Ready"), 13),
            digester.digest(b"\r\nSMS Ready\r\n")
        );
        let urc = Urc::parse(b"SMS Ready").unwrap();
        assert_matches!(urc, Urc::SmsReady);
    }

    #[test]
    fn can_parse_pdp_deact() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+PDP: DEACT"), 15),
            digester.digest(b"\r\n+PDP: DEACT\r\n")
        );
        let urc = Urc::parse(b"+PDP: DEACT").unwrap();
        assert_matches!(urc, Urc::PdpDeact);
    }

    #[test]
    fn can_parse_pin_status() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CPIN: READY"), 16),
            digester.digest(b"\r\n+CPIN: READY\r\n")
        );
        let urc = Urc::parse(b"+CPIN: READY").unwrap();
        assert_matches!(
            urc,
            Urc::PinStatus(PinStatus {
                code: PinStatusCode::Ready
            })
        );
    }

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
    fn can_parse_pdp_context_state() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CGACT: 1,0"), 13),
            digester.digest(b"\r\n+CGACT: 1,0\r\n")
        );
        let urc = Urc::parse(b"+CGACT: 1,0").unwrap();

        if let Urc::PdbState(urc) = urc {
            assert_eq!(ContextId(1), urc.cid);
            assert_eq!(gprs::PdpState::Deactivated, urc.state);
        } else {
            panic!("Invalid URC");
        }
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

        if let Urc::DnsResult(Ok(urc)) = urc {
            assert_eq!(1, urc._success);
            assert_eq!("utiliread.dk", urc.host);
            assert_eq!("123.123.123.123", urc.ip);
            assert_eq!(None, urc.alt_ip);
        } else {
            panic!("Invalid URC");
        }
    }

    #[test]
    fn can_parse_ip_lookup_error() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CDNSGIP: 0,8"), 17),
            digester.digest(b"\r\n+CDNSGIP: 0,8\r\n")
        );
        let urc = Urc::parse(b"+CDNSGIP: 0,8").unwrap();

        if let Urc::DnsResult(Err(code)) = urc {
            assert_eq!(8, code);
        } else {
            panic!("Invalid URC");
        }
    }

    #[test]
    fn can_parse_data_available_sim800() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET: 1,2"), 18),
            digester.digest(b"\r\n+CIPRXGET: 1,2\r\n")
        );
        let urc = Urc::parse(b"+CIPRXGET: 1,2").unwrap();
        assert_matches!(urc, Urc::DataAvailable(2));
    }

    #[test]
    fn can_parse_data_available_sim900() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET:1,2"), 17),
            digester.digest(b"\r\n+CIPRXGET:1,2\r\n")
        );
        let urc = Urc::parse(b"+CIPRXGET:1,2").unwrap();
        assert_matches!(urc, Urc::DataAvailable(2));
    }

    #[test]
    fn can_parse_read_data_sim800() {
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

    #[test]
    fn can_parse_read_data_sim900() {
        let mut digester = SimcomDigester::new();

        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET:2,5,8,0\r\nHTTP\r\n\r\n"), 29),
            digester.digest(b"\r\n+CIPRXGET:2,5,8,0\r\nHTTP\r\n\r\n")
        );
        let urc = Urc::parse(b"+CIPRXGET:2,5,8,0\r\nHTTP\r\n\r\n").unwrap();
        if let Urc::ReadData(data) = urc {
            assert_eq!(5, data.id);
            assert_eq!(8, data.data_len);
            assert_eq!(0, data.pending_len);
            assert_eq!(b"HTTP\r\n\r\n", data.data.take().unwrap().as_slice());
        } else {
            panic!("Invalid URC");
        }
    }

    #[test]
    fn can_parse_adjacent_urcs_and_ok_and_prompt() {
        let mut digester = SimcomDigester::new();

        // This can be seen when we are requesting a ReadData for one socket and another socket connects between the request and the response
        let buf = b"\r\n1, CONNECT OK\r\n\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n\r\nOK\r\n\r\n> ";

        assert_eq!((DigestResult::None, 0), digester.digest(&buf[..16]));
        assert_eq!(
            (DigestResult::Urc(b"1, CONNECT OK"), 17),
            digester.digest(buf)
        );

        let buf = &buf[17..];
        assert_eq!((DigestResult::None, 0), digester.digest(&buf[..29]));
        assert_eq!(
            (DigestResult::Urc(b"+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n"), 30),
            digester.digest(buf)
        );

        let buf = &buf[30..];
        assert_eq!((DigestResult::None, 0), digester.digest(&buf[..5]));
        assert_eq!((DigestResult::Response(Ok(b"")), 6), digester.digest(buf));

        let buf = &buf[6..];
        assert_eq!((DigestResult::Prompt('>' as u8), 4), digester.digest(buf));

        let buf = &buf[4..];
        assert!(buf.is_empty());
    }
}

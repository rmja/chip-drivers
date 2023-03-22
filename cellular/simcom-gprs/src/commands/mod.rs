use atat::atat_derive::{AtatCmd, AtatResp};

pub mod gprs;
pub mod gsm;
pub mod simcom;
pub mod tcpip;
pub mod urc;
pub mod v25ter;

#[derive(AtatCmd)]
#[at_cmd("", NoResponse, termination = "\r")]
pub struct AT;

#[derive(AtatResp)]
pub struct NoResponse;

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::{AtatCmd, DigestResult, Digester};

    use crate::SimcomDigester;

    use super::*;

    #[test]
    fn can_at() {
        let cmd = AT;
        assert_eq_hex!(b"AT\r", cmd.as_bytes());

        let mut digester = SimcomDigester::new();
        assert_eq!(
            (DigestResult::Response(Ok(&[])), 9),
            digester.digest(b"AT\r\r\nOK\r\n")
        );
    }
}

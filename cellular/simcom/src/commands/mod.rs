use atat::atat_derive::{AtatCmd, AtatResp};

pub mod gprs;
pub mod gsm;
pub mod simcom;
pub mod tcpip;
pub mod urc;
pub mod v25ter;

#[derive(Clone, AtatCmd)]
#[at_cmd("", NoResponse, termination = "\r")]
pub struct AT;

#[derive(Debug, Clone, AtatResp)]
pub struct NoResponse;

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use assert_hex::assert_eq_hex;
    use atat::{AtatCmd, DigestResult, Digester};
    use embedded_io::{asynch::Write, Io};

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

    pub struct TestWriter<'a> {
        written: &'a mut Vec<u8>,
    }

    impl<'a> TestWriter<'a> {
        pub const fn new(written: &'a mut Vec<u8>) -> Self {
            Self { written }
        }
    }

    impl Io for TestWriter<'_> {
        type Error = Infallible;
    }

    impl Write for TestWriter<'_> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }
    }
}

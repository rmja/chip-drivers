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
pub(crate) use cmd_ex::AtatCmdEx;

#[cfg(test)]
mod cmd_ex {
    use atat::AtatCmd;

    pub trait AtatCmdEx {
        fn to_vec(&self) -> Vec<u8>;
    }

    impl<T: AtatCmd> AtatCmdEx for T {
        fn to_vec(&self) -> Vec<u8> {
            let mut buf = Vec::new();
            buf.resize(T::MAX_LEN, 0);
            let written = self.write(buf.as_mut_slice());
            buf.truncate(written);
            buf
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::{DigestResult, Digester};

    use crate::SimcomDigester;

    use super::*;

    #[test]
    fn can_at() {
        let cmd = AT;
        assert_eq_hex!(b"AT\r", cmd.to_vec().as_slice());

        let mut digester = SimcomDigester::new();
        assert_eq!(
            (DigestResult::Response(Ok(&[])), 9),
            digester.digest(b"AT\r\r\nOK\r\n")
        );
    }
}

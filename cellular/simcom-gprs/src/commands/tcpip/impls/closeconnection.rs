use atat::{
    atat_derive::AtatCmd,
    nom::{bytes, character, sequence},
    AtatCmd,
};

use crate::commands::{
    tcpip::{CloseConnection, CloseOk},
    NoResponse,
};

impl AtatCmd for CloseConnection {
    type Response = CloseOk;

    const MAX_LEN: usize = "AT+CIPCLOSE=X\r".len();

    // There is no timeout documentation for sim900
    // It has been observed that e.g. "0, CLOSE" arrives up to 10 seconds after "AT+CIPCLOSE=0"
    #[cfg(feature = "sim900")]
    const MAX_TIMEOUT_MS: u32 = 10_000;

    fn write(&self, buf: &mut [u8]) -> usize {
        let inner = CloseConnectionInner { id: self.id };
        inner.write(buf)
    }

    fn parse(
        &self,
        resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        if let Ok((reminder, (id, _))) = sequence::tuple::<_, _, (), _>((
            character::complete::u8,
            bytes::complete::tag(", CLOSE OK"),
        ))(resp?)
        {
            if reminder.is_empty() {
                return Ok(CloseOk { id: id as usize });
            }
        }

        Err(atat::Error::Parse)
    }
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPCLOSE", NoResponse, termination = "\r")]
struct CloseConnectionInner {
    pub id: usize,
}

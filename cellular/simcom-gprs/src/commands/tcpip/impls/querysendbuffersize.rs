use atat::{
    nom::{bytes, character, combinator, sequence},
    AtatCmd,
};

use crate::{
    commands::tcpip::{QuerySendBufferSize, SendBufferSize},
    MAX_SOCKETS,
};

const CMD: &[u8] = b"AT+CIPSEND?\r";

impl AtatCmd for QuerySendBufferSize {
    type Response = SendBufferSize;

    const MAX_LEN: usize = CMD.len();

    fn write(&self, buf: &mut [u8]) -> usize {
        buf[..CMD.len()].copy_from_slice(CMD);
        CMD.len()
    }

    fn parse(
        &self,
        resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        let mut result = SendBufferSize {
            size: [0; MAX_SOCKETS],
        };

        let mut resp = resp?;

        for i in 0..MAX_SOCKETS {
            match sequence::tuple::<_, _, (), _>((
                combinator::opt(bytes::complete::tag(b"\r\n")),
                bytes::complete::tag("+CIPSEND: "),
                character::complete::u8,
                bytes::complete::tag(","),
                character::complete::u16,
            ))(resp)
            {
                Ok((reminder, (_, _, id, _, size))) if i == id as usize => {
                    result.size[i] = size as usize;
                    if reminder.is_empty() {
                        return Ok(result);
                    }

                    resp = reminder;
                }
                _ => return Err(atat::Error::Parse),
            }
        }

        Err(atat::Error::Parse)
    }
}

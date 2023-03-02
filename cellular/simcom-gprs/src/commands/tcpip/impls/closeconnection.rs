use atat::{
    atat_derive::AtatCmd,
    nom::{bytes, character, sequence},
    AtatCmd,
};

use crate::commands::{
    tcpip::{CloseConnection, CloseOk},
    NoResponse,
};

impl AtatCmd<33> for CloseConnection {
    type Response = CloseOk;

    fn as_bytes(&self) -> heapless::Vec<u8, 33> {
        let inner = CloseConnectionInner { id: self.id };
        inner.as_bytes()
    }

    fn parse(
        &self,
        resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        if let Ok((reminder, (id, _))) = sequence::tuple::<_, _, (), _>((
            character::complete::u8,
            bytes::complete::tag(", CLOSE OK"),
        ))(resp?) && reminder.is_empty()
        {
            Ok(CloseOk { id: id as usize })
        } else {
            Err(atat::Error::Parse)
        }
    }
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPCLOSE", NoResponse, termination = "\r")]
struct CloseConnectionInner {
    pub id: usize,
}

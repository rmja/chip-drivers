use atat::{
    nom::{bytes, character, combinator, sequence},
    AtatCmd,
};

use crate::commands::tcpip::{DataAccept, WriteData, WRITE_DATA_MAX_LEN};

impl AtatCmd for WriteData<'_> {
    const MAX_LEN: usize = WRITE_DATA_MAX_LEN;
    const MAX_TIMEOUT_MS: u32 = 5_000;

    type Response = DataAccept;

    fn write(&self, buf: &mut [u8]) -> usize {
        let len = self.buf.len();
        buf[..len].copy_from_slice(self.buf);
        len
    }

    fn parse(
        &self,
        resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        if let Ok((reminder, (_, id, _, accepted))) = sequence::tuple::<_, _, (), _>((
            combinator::recognize(sequence::tuple((
                bytes::complete::tag("DATA ACCEPT:"),
                combinator::opt(bytes::complete::tag(b" ")),
            ))),
            character::complete::u8,
            bytes::complete::tag(","),
            character::complete::u16,
        ))(resp?)
        {
            if reminder.is_empty() {
                return Ok(DataAccept {
                    id: id as usize,
                    accepted: accepted as usize,
                });
            }
        }

        Err(atat::Error::Parse)
    }
}

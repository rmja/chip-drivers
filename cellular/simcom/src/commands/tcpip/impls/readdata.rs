use core::cell::RefCell;

use crate::commands::{
    tcpip::{ReadData, ReadResult},
    NoResponse,
};
use atat::{
    atat_derive::AtatCmd,
    nom::{bytes, character, sequence},
    AtatCmd,
};
use heapless::Vec;

impl<'a> ReadData<'a> {
    pub const fn new(id: usize, buf: &'a mut [u8]) -> Self {
        Self {
            id,
            buf: RefCell::new(buf),
        }
    }
}

impl<'a> AtatCmd<56> for ReadData<'a> {
    type Response = ReadResult;

    fn as_bytes(&self) -> Vec<u8, 56> {
        const MAX_READ: usize = 1460;
        let header = ReadDataHeader {
            id: self.id,
            max_len: usize::min(self.buf.borrow().len(), MAX_READ),
        };
        header.as_bytes()
    }

    fn parse(
        &self,
        resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        let resp = resp?;

        if let Ok((reminder, (_, id, _, data_len, _, pending_len, _))) =
            sequence::tuple::<_, _, (), _>((
                bytes::complete::tag("+CIPRXGET: 2,"),
                character::complete::u8,
                bytes::complete::tag(","),
                character::complete::u16,
                bytes::complete::tag(","),
                character::complete::u16,
                bytes::complete::tag("\r\n"),
            ))(resp)
        {
            let mut buf = self.buf.borrow_mut();
            buf[..data_len as usize].copy_from_slice(reminder);
            Ok(ReadResult {
                _mode: 2,
                id,
                data_len: data_len as usize,
                pending_len: pending_len as usize,
            })
        } else {
            Err(atat::Error::Parse)
        }
    }
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPRXGET=2,", NoResponse, value_sep = false, termination = "\r")]
struct ReadDataHeader {
    id: usize,
    /// The requested number of data bytes (1-1460 bytes) to be read
    max_len: usize,
}

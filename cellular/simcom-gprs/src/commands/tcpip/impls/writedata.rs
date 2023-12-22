use atat::AtatCmd;

use crate::commands::{
    tcpip::{WriteData, WRITE_DATA_MAX_LEN},
    NoResponse,
};

impl AtatCmd for WriteData<'_> {
    const MAX_LEN: usize = WRITE_DATA_MAX_LEN;
    const MAX_TIMEOUT_MS: u32 = 645_000;
    const EXPECTS_RESPONSE_CODE: bool = false;

    type Response = NoResponse;

    fn write(&self, buf: &mut [u8]) -> usize {
        let len = self.buf.len();
        buf[..len].copy_from_slice(self.buf);
        len
    }

    fn parse(
        &self,
        _resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        Ok(NoResponse)
    }
}

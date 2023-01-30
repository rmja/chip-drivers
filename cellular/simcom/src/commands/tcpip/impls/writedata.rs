use atat::AtatCmd;

use crate::commands::{tcpip::WriteData, NoResponse};
use heapless::Vec;

impl AtatCmd<0> for WriteData<'_> {
    const MAX_TIMEOUT_MS: u32 = 645_000;
    const EXPECTS_RESPONSE_CODE: bool = false;

    type Response = NoResponse;

    fn parse(
        &self,
        _resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        Ok(NoResponse)
    }

    fn as_bytes(&self) -> Vec<u8, 0> {
        Vec::new()
    }

    fn get_slice(&self, _bytes: &Vec<u8, 0>) -> &[u8] {
        self.buf
    }
}

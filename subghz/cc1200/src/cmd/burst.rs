use crate::{regs::RegisterAddress, StatusByte};

use super::{Command, Response, BURST_READ, BURST_WRITE, EXTENDED_ADDRESS, FIFO};

pub struct BurstHeader {
    pub request: BurstHeaderRequest,
    pub response: BurstHeaderResponse,
}

pub struct BurstHeaderRequest {
    buf: [u8; 2],
}

pub struct BurstHeaderResponse {
    len: usize,
    buf: [u8; 2],
}

impl BurstHeader {
    pub const fn read(first: RegisterAddress) -> Self {
        let request = BurstHeaderRequest::read(first);
        let response = BurstHeaderResponse::new(request.len());
        Self { request, response }
    }

    pub const fn write(first: RegisterAddress) -> Self {
        let request = BurstHeaderRequest::write(first);
        let response = BurstHeaderResponse::new(request.len());
        Self { request, response }
    }

    pub const fn read_fifo() -> Self {
        let request = BurstHeaderRequest::read_fifo();
        let response = BurstHeaderResponse::new(request.len());
        Self { request, response }
    }

    pub const fn write_fifo() -> Self {
        let request = BurstHeaderRequest::write_fifo();
        let response = BurstHeaderResponse::new(request.len());
        Self { request, response }
    }
}

impl Command for BurstHeader {
    fn len(&self) -> usize {
        self.request.len()
    }
}

impl BurstHeaderRequest {
    const fn read(first: RegisterAddress) -> Self {
        let buf = if first.is_primary() {
            [BURST_READ | first.0 as u8, 0]
        } else {
            [BURST_READ | EXTENDED_ADDRESS, first.0 as u8]
        };
        Self { buf }
    }

    const fn write(first: RegisterAddress) -> Self {
        let buf = if first.is_primary() {
            [BURST_WRITE | first.0 as u8, 0]
        } else {
            [BURST_WRITE | EXTENDED_ADDRESS, first.0 as u8]
        };
        Self { buf }
    }

    const fn read_fifo() -> Self {
        let buf = [BURST_READ | FIFO, 0];
        Self { buf }
    }

    const fn write_fifo() -> Self {
        let buf = [BURST_WRITE | FIFO, 0];
        Self { buf }
    }

    const fn len(&self) -> usize {
        if (self.buf[0] & FIFO) == EXTENDED_ADDRESS {
            2
        } else {
            1
        }
    }
}

impl AsRef<[u8]> for BurstHeaderRequest {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len()]
    }
}

impl BurstHeaderResponse {
    const fn new(len: usize) -> Self {
        Self { len, buf: [0; 2] }
    }
}

impl Response for BurstHeaderResponse {
    fn status_byte(&self) -> StatusByte {
        StatusByte(self.buf[0])
    }
}

impl AsRef<[u8]> for BurstHeaderResponse {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl AsMut<[u8]> for BurstHeaderResponse {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.len]
    }
}

#[cfg(test)]
mod tests {
    use crate::regs::{ext::FreqoffCfg, pri::Iocfg2, Register};

    use super::*;

    #[test]
    fn read_primary() {
        let mut header = BurstHeader::read(Iocfg2::ADDRESS);
        assert_eq!(1, header.len());
        assert_eq!(&[BURST_READ | 0x01], header.request.as_ref());

        assert_eq!(1, header.response.as_ref().len());
        assert_eq!(1, header.response.as_mut().len());
    }

    #[test]
    fn read_extended() {
        let mut header = BurstHeader::read(FreqoffCfg::ADDRESS);
        assert_eq!(2, header.len());
        assert_eq!(
            &[BURST_READ | EXTENDED_ADDRESS, 0x01],
            header.request.as_ref()
        );

        assert_eq!(2, header.response.as_ref().len());
        assert_eq!(2, header.response.as_mut().len());
    }

    #[test]
    fn write_primary() {
        let mut header = BurstHeader::write(Iocfg2::ADDRESS);
        assert_eq!(1, header.len());
        assert_eq!(&[BURST_WRITE | 0x01], header.request.as_ref());

        assert_eq!(1, header.response.as_ref().len());
        assert_eq!(1, header.response.as_mut().len());
    }

    #[test]
    fn write_extended() {
        let mut header = BurstHeader::write(FreqoffCfg::ADDRESS);
        assert_eq!(2, header.len());
        assert_eq!(
            &[BURST_WRITE | EXTENDED_ADDRESS, 0x01],
            header.request.as_ref()
        );

        assert_eq!(2, header.response.as_ref().len());
        assert_eq!(2, header.response.as_mut().len());
    }

    #[test]
    fn read_fifo() {
        let mut header = BurstHeader::read_fifo();
        assert_eq!(1, header.len());
        assert_eq!(&[BURST_READ | 0x3F], header.request.as_ref());

        assert_eq!(1, header.response.as_ref().len());
        assert_eq!(1, header.response.as_mut().len());
    }

    #[test]
    fn write_fifo() {
        let mut header = BurstHeader::write_fifo();
        assert_eq!(1, header.len());
        assert_eq!(&[BURST_WRITE | 0x3F], header.request.as_ref());

        assert_eq!(1, header.response.as_ref().len());
        assert_eq!(1, header.response.as_mut().len());
    }
}

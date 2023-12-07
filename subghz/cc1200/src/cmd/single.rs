use crate::{regs::RegisterAddress, StatusByte};

use super::{Command, Response, EXTENDED_ADDRESS, FIFO, SINGLE_READ, SINGLE_WRITE};

pub struct SingleCommand {
    pub request: SingleRequest,
    pub response: SingleResponse,
}

pub struct SingleRequest {
    buf: [u8; 3],
}

pub struct SingleResponse {
    len: usize,
    buf: [u8; 3],
}

impl SingleCommand {
    pub const fn read(address: RegisterAddress) -> Self {
        let request = SingleRequest::read(address);
        let response = SingleResponse::new(request.len());
        Self { request, response }
    }

    pub const fn write(address: RegisterAddress, value: u8) -> Self {
        let request = SingleRequest::write(address, value);
        let response = SingleResponse::new(request.len());
        Self { request, response }
    }
}

impl Command for SingleCommand {
    fn len(&self) -> usize {
        self.request.len()
    }
}

impl SingleRequest {
    const fn read(address: RegisterAddress) -> Self {
        let buf = if address.is_primary() {
            [SINGLE_READ | address.0 as u8, 0, 0]
        } else {
            [SINGLE_READ | EXTENDED_ADDRESS, address.0 as u8, 0]
        };
        Self { buf }
    }

    const fn write(address: RegisterAddress, value: u8) -> Self {
        let buf = if address.is_primary() {
            [SINGLE_WRITE | address.0 as u8, value, 0]
        } else {
            [SINGLE_WRITE | EXTENDED_ADDRESS, address.0 as u8, value]
        };
        Self { buf }
    }

    const fn len(&self) -> usize {
        if (self.buf[0] & FIFO) == EXTENDED_ADDRESS {
            3
        } else {
            2
        }
    }
}

impl AsRef<[u8]> for SingleRequest {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len()]
    }
}

impl SingleResponse {
    const fn new(len: usize) -> Self {
        Self { len, buf: [0; 3] }
    }

    pub fn value(&self) -> u8 {
        self.buf[self.len - 1]
    }
}

impl Response for SingleResponse {
    fn status_byte(&self) -> StatusByte {
        StatusByte(self.buf[0])
    }
}

impl AsRef<[u8]> for SingleResponse {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl AsMut<[u8]> for SingleResponse {
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
        let mut cmd = SingleCommand::read(Iocfg2::ADDRESS);
        assert_eq!(2, cmd.len());
        assert_eq!(&[SINGLE_READ | 0x01, 0x00], cmd.request.as_ref());

        assert_eq!(2, cmd.response.as_ref().len());
        assert_eq!(2, cmd.response.as_mut().len());
    }

    #[test]
    fn read_extended() {
        let mut cmd = SingleCommand::read(FreqoffCfg::ADDRESS);
        assert_eq!(3, cmd.len());
        assert_eq!(
            &[SINGLE_READ | EXTENDED_ADDRESS, 0x01, 0x00],
            cmd.request.as_ref()
        );

        assert_eq!(3, cmd.response.as_ref().len());
        assert_eq!(3, cmd.response.as_mut().len());
    }

    #[test]
    fn write_primary() {
        let mut cmd = SingleCommand::write(Iocfg2::ADDRESS, 0xA0);
        assert_eq!(2, cmd.len());
        assert_eq!(&[SINGLE_WRITE | 0x01, 0xA0], cmd.request.as_ref());

        assert_eq!(2, cmd.response.as_ref().len());
        assert_eq!(2, cmd.response.as_mut().len());
    }

    #[test]
    fn write_extended() {
        let mut cmd = SingleCommand::write(FreqoffCfg::ADDRESS, 0xA0);
        assert_eq!(3, cmd.len());
        assert_eq!(
            &[SINGLE_WRITE | EXTENDED_ADDRESS, 0x01, 0xA0],
            cmd.request.as_ref()
        );

        assert_eq!(3, cmd.response.as_ref().len());
        assert_eq!(3, cmd.response.as_mut().len());
    }
}

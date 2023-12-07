use core::slice;

use crate::StatusByte;

use super::{Command, Response};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Strobe {
    /// Reset chip
    SRES = 0x30,
    /// Enable and calibrate frequency synthesizer
    SFSTXON = 0x31,
    /// Turn off crystal oscillator (Enter XOFF state when CSn is de-asserted)
    SXOFF = 0x32,
    /// Calibrate frequency synthesizer and turn it off
    SCAL = 0x33,
    /// Enable RX
    SRX = 0x34,
    /// Enable TX
    STX = 0x35,
    /// Exit RX/TX and turn off frequency synthesizer
    SIDLE = 0x36,
    /// Automatic frequency compensation
    SAFC = 0x37,
    /// Start automatic RX polling sequence
    SWOR = 0x38,
    /// Enter SLEEP mode when CSn is de-asserted
    SPWD = 0x39,
    /// Flush the RX FIFO
    SFRX = 0x3A,
    /// Flush the TX FIFO
    SFTX = 0x3B,
    /// Reset real time clock
    SWORRST = 0x3C,
    /// No operation - may be used to get access to the chip status byte
    SNOP = 0x3D,
}

pub struct StrobeCommand {
    pub request: StrobeRequest,
    pub response: StrobeResponse,
}

pub struct StrobeRequest(u8);

pub struct StrobeResponse(u8);

impl StrobeCommand {
    pub const fn new(strobe: Strobe) -> Self {
        Self {
            request: StrobeRequest(strobe as u8),
            response: StrobeResponse(0),
        }
    }
}

impl Command for StrobeCommand {
    fn len(&self) -> usize {
        1
    }
}

impl AsRef<[u8]> for StrobeRequest {
    fn as_ref(&self) -> &[u8] {
        slice::from_ref(&self.0)
    }
}

impl Response for StrobeResponse {
    fn status_byte(&self) -> StatusByte {
        StatusByte(self.0)
    }
}

impl AsRef<[u8]> for StrobeResponse {
    fn as_ref(&self) -> &[u8] {
        slice::from_ref(&self.0)
    }
}

impl AsMut<[u8]> for StrobeResponse {
    fn as_mut(&mut self) -> &mut [u8] {
        slice::from_mut(&mut self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strobe() {
        let mut cmd = StrobeCommand::new(Strobe::SNOP);
        assert_eq!(1, cmd.len());
        assert_eq!(&[0x3d], cmd.request.as_ref());

        assert_eq!(1, cmd.response.as_ref().len());
        assert_eq!(1, cmd.response.as_mut().len());
    }
}

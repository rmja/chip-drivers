use crate::{regs::ext::Marcstate, DriverError, State};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControllerError {
    Recalibrated,
    FifoOverflow,
    Driver(DriverError),
    UnrecoverableChipState(State, Marcstate),
    Offline,
}

impl From<DriverError> for ControllerError {
    fn from(value: DriverError) -> Self {
        ControllerError::Driver(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_driver_error() {
        let error = ControllerError::Driver(DriverError::InvalidPartNumber);
        let msg = format!("{:?}", error);
        assert_eq!("Driver(InvalidPartNumber)", &msg);
    }

    #[test]
    fn display_state_error() {
        let error = ControllerError::UnrecoverableChipState(State::RX_FIFO_ERROR, Marcstate(0x41));
        let msg = format!("{:?}", error);
        assert_eq!("UnrecoverableChipState(RX_FIFO_ERROR, 0x41)", &msg);
    }
}

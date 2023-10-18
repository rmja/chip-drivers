use crate::{DriverError, State};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControllerError {
    Driver(DriverError),
    FifoOverflow,
    UnrecoverableChipState(State),
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
        let error = ControllerError::UnrecoverableChipState(State::RX_FIFO_ERROR);
        let msg = format!("{:?}", error);
        assert_eq!("UnrecoverableChipState(RX_FIFO_ERROR)", &msg);
    }
}

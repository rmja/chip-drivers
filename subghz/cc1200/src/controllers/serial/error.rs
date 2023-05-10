use crate::{DriverError, State};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControllerError {
    Driver(DriverError),
    RxFifoOverflow,
    UnrecoverableChipState(State),
}

impl From<DriverError> for ControllerError {
    fn from(value: DriverError) -> Self {
        ControllerError::Driver(value)
    }
}

use crate::{DriverError, State};

#[derive(Debug)]
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

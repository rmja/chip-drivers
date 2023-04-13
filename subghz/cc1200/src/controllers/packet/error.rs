use crate::DriverError;

#[derive(Debug)]
pub enum ControllerError {
    Driver(DriverError),
    WriteCapacity,
    TxFifoUnderflow,
    RxFifoOverflow,
}

impl From<DriverError> for ControllerError {
    fn from(value: DriverError) -> Self {
        ControllerError::Driver(value)
    }
}

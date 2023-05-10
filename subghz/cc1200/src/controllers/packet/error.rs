use crate::DriverError;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

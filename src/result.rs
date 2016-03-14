use std::io;
use std::convert;
use std::sync::PoisonError;
use byteorder;

#[derive(Debug)]
pub enum ThrustError {
    Other,
    NotReady,
    Str(String),
    IO(io::Error),
    ByteOrder(byteorder::Error),
    PoisonError
}

pub type ThrustResult<T> = Result<T, ThrustError>;

impl convert::From<io::Error> for ThrustError {
    fn from(val: io::Error) -> ThrustError {
        ThrustError::IO(val)
    }
}

impl convert::From<byteorder::Error> for ThrustError {
    fn from(val: byteorder::Error) -> ThrustError {
        ThrustError::ByteOrder(val)
    }
}

impl<T> convert::From<PoisonError<T>> for ThrustError {
    fn from(val: PoisonError<T>) -> ThrustError {
        ThrustError::PoisonError
    }
}

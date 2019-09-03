use std::{error, fmt};

#[derive(Debug)]
pub enum Error<SPIE> {
    SpiError(SPIE),
}

impl<SPIE: fmt::Display> fmt::Display for Error<SPIE> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SpiError(e) => write!(f, "{}", e)
        }
    }
}

impl<SPIE: error::Error> error::Error for Error<SPIE> {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::SpiError(e) => Some(e)
        }
    }
}


impl<SPIE: fmt::Debug> From<SPIE> for Error<SPIE> {
    fn from(e: SPIE) -> Self {
        Error::SpiError(e)
    }
}


#[derive(Debug)]
pub enum TransmissionError<TErr> {
    DeviceError(TErr),
    MaximumRetriesExceeded,
}


impl<TErr: fmt::Display> fmt::Display for TransmissionError<TErr> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransmissionError::MaximumRetriesExceeded => f.write_str("Maximum retries exceeded"),
            TransmissionError::DeviceError(e) => write!(f, "{}", e)
        }
    }
}


impl<TErr: error::Error> error::Error for TransmissionError<TErr> {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            TransmissionError::DeviceError(e) => Some(e),
            _ => None
        }
    }
}


impl<TErr> From<TErr> for TransmissionError<TErr> {
    fn from(e: TErr) -> Self {
        TransmissionError::DeviceError(e)
    }
}

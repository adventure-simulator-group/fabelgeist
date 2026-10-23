//! Concrete failures at NumPy scalar decoding and tensor-layout boundaries.

use std::error::Error;
use std::io;

use derive_more::{Display, From};

use super::NpyLayoutError;

pub type Result<T> = std::result::Result<T, NpyDecodeError>;

#[derive(Debug, Display, From)]
pub enum NpyDecodeError {
    #[display("{_0}")]
    Descriptor(npyz::ParseTypeStrError),
    #[display("{_0}")]
    Scalar(npyz::DTypeError),
    #[display("{_0}")]
    Payload(io::Error),
    #[display("{_0}")]
    Layout(NpyLayoutError),
}

impl Error for NpyDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Descriptor(error) => Some(error),
            Self::Scalar(error) => Some(error),
            Self::Payload(error) => Some(error),
            // Layout presentation stays transparent, including its native rank cause.
            Self::Layout(error) => error.source(),
        }
    }
}

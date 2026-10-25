use super::BufferByteLength;
use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

pub(super) type ReadResult<T> = Result<T, BufferReadError>;

/// Why a logical buffer prefix could not be read into host elements.
#[derive(Debug)]
pub enum BufferReadError {
    MappingExtentOverflow {
        requested: BufferByteLength,
    },
    ExtentExceedsAllocation {
        requested: BufferByteLength,
        available: BufferByteLength,
    },
    ZeroSizedElement,
    PartialElement {
        bytes: BufferByteLength,
        element_bytes: usize,
    },
    Truncated {
        requested: BufferByteLength,
        available: BufferByteLength,
    },
    HostAddress(TryFromIntError),
    HostAllocation(TryReserveError),
    Native(wgpu::Error),
    Mapping(wgpu::BufferAsyncError),
    Poll(wgpu::PollError),
    Channel(futures_channel::oneshot::Canceled),
}

impl fmt::Display for BufferReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MappingExtentOverflow { requested } => write!(
                formatter,
                "GPU readback cannot align a {requested}-byte mapping extent"
            ),
            Self::ExtentExceedsAllocation {
                requested,
                available,
            } => write!(
                formatter,
                "GPU readback requests {requested} bytes from a {available}-byte allocation"
            ),
            Self::ZeroSizedElement => {
                formatter.write_str("GPU readback requires nonzero-sized elements")
            }
            Self::PartialElement {
                bytes,
                element_bytes,
            } => write!(
                formatter,
                "GPU readback's {bytes} bytes do not contain whole {element_bytes}-byte elements"
            ),
            Self::Truncated {
                requested,
                available,
            } => write!(
                formatter,
                "GPU readback received {available} of {requested} bytes"
            ),
            Self::HostAddress(cause) => write!(formatter, "GPU readback host extent: {cause}"),
            Self::HostAllocation(cause) => {
                write!(formatter, "GPU readback host allocation: {cause}")
            }
            Self::Native(cause) => write!(formatter, "GPU readback validation: {cause}"),
            Self::Mapping(cause) => write!(formatter, "GPU mapping: {cause}"),
            Self::Poll(cause) => write!(formatter, "GPU readback progress: {cause}"),
            Self::Channel(cause) => write!(formatter, "GPU mapping callback: {cause}"),
        }
    }
}

impl Error for BufferReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HostAddress(cause) => Some(cause),
            Self::HostAllocation(cause) => Some(cause),
            Self::Native(cause) => Some(cause),
            Self::Mapping(cause) => Some(cause),
            Self::Poll(cause) => Some(cause),
            Self::Channel(cause) => Some(cause),
            _ => None,
        }
    }
}

//! Local buffer allocation and queue-write request admission.

use super::{BufferByteLength, BufferByteOffset};
use std::{error::Error, fmt};

/// A request rejected before native allocation or queue writing.
/// Native device, usage and alignment failures retain their separate causes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferAdmissionError {
    EmptyAllocation,
    WriteOffsetOverflow {
        at: BufferByteOffset,
        bytes: BufferByteLength,
    },
    WriteOutOfBounds {
        at: BufferByteOffset,
        bytes: BufferByteLength,
        capacity: BufferByteLength,
    },
}

impl fmt::Display for BufferAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAllocation => formatter.write_str("Buffer size must be greater than 0"),
            Self::WriteOffsetOverflow { .. } => {
                formatter.write_str("Buffer write overflows an offset")
            }
            Self::WriteOutOfBounds {
                at,
                bytes,
                capacity,
            } => write!(
                formatter,
                "Writing {bytes} bytes at {at} runs past the end of a {capacity}-byte buffer"
            ),
        }
    }
}

impl Error for BufferAdmissionError {}

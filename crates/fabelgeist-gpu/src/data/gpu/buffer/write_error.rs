//! Checked addressed-upload failures before native queue validation.

use super::{BufferByteLength, BufferByteOffset};

/// Failure to admit one host upload within a buffer's logical byte extent.
///
/// Native queue alignment, usage and allocation checks are separate. These
/// errors have no source error: their fields retain the rejected byte request.
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{BufferByteLength, BufferWriteError};
/// let error = BufferWriteError::OutOfBounds {
///     at: 0u64,
///     bytes: BufferByteLength::from(4u64),
///     length: BufferByteLength::from(16u64),
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferWriteError {
    /// The requested offset plus upload length exceeds the native u64 range.
    OffsetOverflow {
        at: BufferByteOffset,
        bytes: BufferByteLength,
    },
    /// The representable upload end exceeds the buffer's logical byte length.
    OutOfBounds {
        at: BufferByteOffset,
        bytes: BufferByteLength,
        length: BufferByteLength,
    },
}

pub type BufferWriteResult<T> = std::result::Result<T, BufferWriteError>;

impl std::fmt::Display for BufferWriteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OffsetOverflow { .. } => formatter.write_str("Buffer write overflows an offset"),
            Self::OutOfBounds { at, bytes, length } => write!(
                formatter,
                "Writing {bytes} bytes at {at} runs past the end of a {length}-byte buffer"
            ),
        }
    }
}

impl std::error::Error for BufferWriteError {}

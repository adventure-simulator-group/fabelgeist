//! Admission and native mapping failures for one buffer readback.

use super::BufferByteLength;
use std::{error::Error, fmt, num::TryFromIntError};

/// The result of admitting and mapping a buffer's logical bytes.
///
/// Other GPU providers retain their own errors. Conversion into `anyhow` at a
/// mixed-provider caller preserves this concrete root and its native cause.
pub type BufferReadResult<T> = Result<T, BufferReadError>;

/// A host readback rejection or a native mapping/callback failure.
///
/// Byte fields retain the logical extent or the actual mapped-view extent.
/// Mapping and cancellation retain SDK/channel causes rather than their text.
#[derive(Debug)]
pub enum BufferReadError {
    HostLengthConversion {
        bytes: BufferByteLength,
        cause: TryFromIntError,
    },
    ZeroSizedElement,
    PartialElement {
        bytes: BufferByteLength,
        element_bytes: BufferByteLength,
    },
    TruncatedMappedView {
        expected: BufferByteLength,
        available: BufferByteLength,
    },
    Mapping(wgpu::BufferAsyncError),
    CanceledChannel(futures_channel::oneshot::Canceled),
}

impl fmt::Display for BufferReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HostLengthConversion { cause, .. } => fmt::Display::fmt(cause, formatter),
            Self::ZeroSizedElement => {
                formatter.write_str("GPU readback requires nonzero-sized elements")
            }
            Self::PartialElement { .. } => {
                formatter.write_str("GPU readback byte length is not a whole number of elements")
            }
            Self::TruncatedMappedView { .. } => formatter.write_str("GPU readback is truncated"),
            Self::Mapping(cause) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    write!(formatter, "GPU Mapping error: {cause:?}")
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = cause;
                    formatter.write_str("GPU Mapping error")
                }
            }
            Self::CanceledChannel(_) => formatter.write_str("Mapping channel closed"),
        }
    }
}

impl Error for BufferReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HostLengthConversion { cause, .. } => Some(cause),
            Self::Mapping(cause) => Some(cause),
            Self::CanceledChannel(cause) => Some(cause),
            Self::ZeroSizedElement
            | Self::PartialElement { .. }
            | Self::TruncatedMappedView { .. } => None,
        }
    }
}

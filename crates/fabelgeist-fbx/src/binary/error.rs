//! Rejections at the binary FBX decoding boundary.

use crate::{FbxArrayCount, UnsupportedFbxArrayEncoding};
use std::{fmt, io};
use thiserror::Error;

/// Result shared by binary decoding and serialized scene construction.
pub type Result<T> = std::result::Result<T, FbxDecodeError>;

/// A binary decoding rejection with its admitted context or native cause.
#[derive(Debug, Error)]
pub enum FbxDecodeError {
    #[error("this is an ASCII FBX file; re-export it as binary FBX")]
    AsciiInput,
    #[error("not a binary FBX file")]
    NotBinaryInput,
    #[error("FBX read overflow")]
    ReadOverflow {
        /// Native byte position of the failed read.
        native_offset: usize,
        /// Rejected native byte request, before addition.
        native_length: usize,
    },
    #[error("truncated FBX file: wanted {native_length} bytes at offset {native_offset}")]
    TruncatedRead {
        native_offset: usize,
        native_length: usize,
    },
    #[error(fmt = fmt_inflate)]
    Inflate { source: io::Error },
    #[error("{0}")]
    UnsupportedArrayEncoding(#[from] UnsupportedFbxArrayEncoding),
    #[error(
        "FBX array property is short: {native_bytes} bytes for {} x {native_width}",
        .count.native_len()
    )]
    ArrayShort {
        /// Observed native decoded-buffer length in bytes.
        native_bytes: usize,
        count: FbxArrayCount,
        /// Input width of the selected native element decoder, in bytes.
        native_width: usize,
    },
    #[error("unknown FBX property type {:?}", char::from(*.native_tag))]
    PropertyTag {
        /// Rejected native tag byte, before supported-tag admission.
        native_tag: u8,
    },
}

// The previous anyhow context printed each native cause only in alternate
// Display. Keep that presentation while deriving the Error source chain.
fn fmt_inflate(source: &io::Error, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("inflating FBX array property")?;
    if formatter.alternate() {
        let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(source);
        while let Some(error) = cause {
            write!(formatter, ": {error}")?;
            cause = error.source();
        }
    }
    Ok(())
}

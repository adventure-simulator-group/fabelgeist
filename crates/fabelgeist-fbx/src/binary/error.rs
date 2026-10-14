//! Rejections at the native binary FBX serialization boundary.
use std::{error::Error, fmt, io};

/// A binary FBX decoding failure, with rejected native diagnostic provenance.
///
/// Native positions, lengths, counts and tags here describe failed serialized
/// input. They are not admitted storage or metadata values and expose no
/// arithmetic, validation or conversion API. General storage ownership remains
/// separate from this format-specific failure classification.
#[derive(Debug)]
pub enum FbxDecodeError {
    AsciiInput,
    NotBinaryInput,
    ReadOverflow {
        /// Native byte position of the failed read.
        native_offset: usize,
        /// Rejected native byte request, before addition.
        native_length: usize,
    },
    TruncatedRead {
        /// Native byte position of the failed read.
        native_offset: usize,
        /// Rejected native byte request.
        native_length: usize,
    },
    Inflate {
        /// Original native decompressor IO failure.
        source: io::Error,
    },
    ArrayShort {
        /// Observed native decoded-buffer length in bytes.
        native_bytes: usize,
        /// Native wire element count, before successful array admission.
        native_count: usize,
        /// Native scalar byte arity used by the little-endian conversion.
        native_width: usize,
    },
    PropertyTag {
        /// Unrecognized native wire byte, before tag admission.
        native_tag: u8,
    },
}

/// Result shared by the binary decoder and serialized scene constructor.
///
/// Errors retain their concrete FBX classification:
/// ```
/// use fabelgeist_fbx::{FbxDecodeError, Node, Result, parse};
/// let parsed: Result<Vec<Node>> = parse(b"; FBX");
/// assert!(matches!(parsed, Err(FbxDecodeError::AsciiInput)));
/// ```
/// The public error is not an arbitrary native IO result:
/// ```compile_fail
/// use fabelgeist_fbx::{Node, parse};
/// let parsed: std::result::Result<Vec<Node>, std::io::Error> = parse(b"; FBX");
/// ```
pub type Result<T> = std::result::Result<T, FbxDecodeError>;

impl fmt::Display for FbxDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AsciiInput => {
                formatter.write_str("this is an ASCII FBX file; re-export it as binary FBX")
            }
            Self::NotBinaryInput => formatter.write_str("not a binary FBX file"),
            Self::ReadOverflow { .. } => formatter.write_str("FBX read overflow"),
            Self::TruncatedRead {
                native_offset,
                native_length,
            } => write!(
                formatter,
                "truncated FBX file: wanted {native_length} bytes at offset {native_offset}"
            ),
            Self::Inflate { .. } => formatter.write_str("inflating FBX array property"),
            Self::ArrayShort {
                native_bytes,
                native_count,
                native_width,
            } => write!(
                formatter,
                "FBX array property is short: {native_bytes} bytes for {native_count} x {native_width}"
            ),
            Self::PropertyTag { native_tag } => write!(
                formatter,
                "unknown FBX property type {:?}",
                *native_tag as char
            ),
        }
    }
}
impl Error for FbxDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Inflate { source } => Some(source),
            _ => None,
        }
    }
}

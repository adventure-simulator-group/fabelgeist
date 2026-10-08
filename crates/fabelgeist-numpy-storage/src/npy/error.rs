//! NPY format/file failures and their original native diagnostic causes.
use std::{error::Error, fmt, io, num::ParseIntError, path::PathBuf, str::Utf8Error};

/// Required native header field missing at its existing serialized read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpyHeaderField {
    Dtype,
    Shape,
}

/// Existing decoding failures; rejected text is serialization provenance.
///
/// Native descriptor/token fields retain rejected input spelling only. They
/// neither admit metadata nor supply a semantic value/getter/arithmetic API.
#[derive(Debug)]
pub enum NpyDecodeError {
    Magic,
    TruncatedHeader,
    HeaderEncoding(Utf8Error),
    MissingField(NpyHeaderField),
    Dtype {
        /// Rejected descriptor after the existing native quote trimming.
        native_descr: String,
    },
    FortranOrder,
    ShapeDimension {
        /// Rejected native shape token after the existing whitespace trimming.
        native_token: String,
        source: ParseIntError,
    },
    TruncatedPayload,
}

/// Native file reading or decoding failure with the original path context.
#[derive(Debug)]
pub enum NpyReadError {
    Read {
        /// Original native OS path; diagnostic provenance, not a file owner.
        native_path: PathBuf,
        source: io::Error,
    },
    Decode {
        /// Original native OS path used for the serialized array input.
        native_path: PathBuf,
        source: NpyDecodeError,
    },
}

/// Concrete result for serialized array admission.
/// ```
/// use fabelgeist_numpy_storage::{NpyArray, npy::{DecodeResult, NpyDecodeError}};
/// let array: DecodeResult<NpyArray> = NpyArray::from_bytes(&[]);
/// assert!(matches!(array, Err(NpyDecodeError::Magic)));
/// ```
/// ```compile_fail
/// use fabelgeist_numpy_storage::{NpyArray, npy::ReadResult};
/// let array: ReadResult<NpyArray> = NpyArray::from_bytes(&[]);
/// ```
pub type DecodeResult<T> = std::result::Result<T, NpyDecodeError>;
/// Concrete result for native file read followed by serialized admission.
pub type ReadResult<T> = std::result::Result<T, NpyReadError>;

impl fmt::Display for NpyDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Magic => formatter.write_str("not a .npy array"),
            Self::TruncatedHeader => formatter.write_str("truncated .npy header"),
            Self::HeaderEncoding(_) => formatter.write_str("non-UTF-8 .npy header"),
            Self::MissingField(field) => match field {
                NpyHeaderField::Dtype => formatter.write_str("missing 'descr' in .npy header"),
                NpyHeaderField::Shape => formatter.write_str("missing 'shape' in .npy header"),
            },
            Self::Dtype { native_descr } => {
                write!(formatter, "unsupported NumPy dtype {native_descr:?}")
            }
            Self::FortranOrder => {
                formatter.write_str("Fortran-ordered .npy arrays are not supported")
            }
            Self::ShapeDimension { .. } => formatter.write_str("bad .npy shape"),
            Self::TruncatedPayload => formatter.write_str("truncated .npy payload"),
        }
    }
}
impl Error for NpyDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HeaderEncoding(source) => Some(source),
            Self::ShapeDimension { source, .. } => Some(source),
            _ => None,
        }
    }
}
impl fmt::Display for NpyReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { native_path, .. } => {
                write!(formatter, "reading {}", native_path.display())
            }
            Self::Decode { native_path, .. } => {
                write!(formatter, "parsing {}", native_path.display())
            }
        }
    }
}
impl Error for NpyReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Decode { source, .. } => Some(source),
        }
    }
}

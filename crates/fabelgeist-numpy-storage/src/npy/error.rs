use fabelgeist_fs::NativeFile;
use fabelgeist_storage::{StorageBoundsError, StorageByteLength};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpySection {
    Version,
    HeaderLength,
    Header,
    Payload,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpyHeaderField {
    Dtype,
    Shape,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyRank(usize);
impl From<usize> for NpyRank {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedNpyValue(String);
impl From<&str> for RejectedNpyValue {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyVersionEncoding([u8; 2]);
impl From<[u8; 2]> for NpyVersionEncoding {
    fn from(value: [u8; 2]) -> Self {
        Self(value)
    }
}
#[derive(Debug)]
pub enum NpyDecodeError {
    Magic,
    Version(NpyVersionEncoding),
    Bounds {
        section: NpySection,
        source: StorageBoundsError,
    },
    HeaderEncoding(std::str::Utf8Error),
    MissingField(NpyHeaderField),
    Dtype(RejectedNpyValue),
    FortranOrder,
    ShapeDimension {
        input: RejectedNpyValue,
        source: std::num::ParseIntError,
    },
    ElementCountOverflow,
    PayloadLengthOverflow,
    TruncatedPayload {
        expected: StorageByteLength,
        available: StorageByteLength,
    },
    Rank {
        expected: NpyRank,
        actual: NpyRank,
        source: std::array::TryFromSliceError,
    },
}
#[derive(Debug)]
pub enum NpyReadError {
    Read {
        file: NativeFile,
        source: std::io::Error,
    },
    Decode {
        file: NativeFile,
        source: NpyDecodeError,
    },
}
impl std::fmt::Display for NpyDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Magic => f.write_str("not a NumPy array"),
            Self::Version(version) => write!(
                f,
                "unsupported NumPy format version {}.{}",
                version.0[0], version.0[1]
            ),
            Self::Bounds { section, source } => write!(f, "NumPy {section:?}: {source}"),
            Self::HeaderEncoding(source) => write!(f, "non-UTF-8 NumPy header: {source}"),
            Self::MissingField(field) => write!(f, "missing NumPy header {field:?}"),
            Self::Dtype(value) => write!(f, "unsupported NumPy dtype {:?}", value.0),
            Self::FortranOrder => f.write_str("Fortran-ordered NumPy arrays are unsupported"),
            Self::ShapeDimension { input, source } => {
                write!(f, "invalid NumPy dimension {:?}: {source}", input.0)
            }
            Self::ElementCountOverflow => {
                f.write_str("NumPy element count overflows the storage layout")
            }
            Self::PayloadLengthOverflow => {
                f.write_str("NumPy payload length overflows the storage layout")
            }
            Self::TruncatedPayload {
                expected,
                available,
            } => write!(
                f,
                "NumPy payload needs {expected}, only {available} available"
            ),
            Self::Rank {
                expected, actual, ..
            } => {
                write!(f, "NumPy array rank {}, expected {}", actual.0, expected.0)
            }
        }
    }
}
impl std::error::Error for NpyDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bounds { source, .. } => Some(source),
            Self::HeaderEncoding(source) => Some(source),
            Self::ShapeDimension { source, .. } => Some(source),
            Self::Rank { source, .. } => Some(source),
            _ => None,
        }
    }
}
impl std::fmt::Display for NpyReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { file, source } => write!(
                f,
                "reading NumPy file {}: {source}",
                file.as_ref().display()
            ),
            Self::Decode { file, source } => write!(
                f,
                "decoding NumPy file {}: {source}",
                file.as_ref().display()
            ),
        }
    }
}
impl std::error::Error for NpyReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Decode { source, .. } => Some(source),
        }
    }
}

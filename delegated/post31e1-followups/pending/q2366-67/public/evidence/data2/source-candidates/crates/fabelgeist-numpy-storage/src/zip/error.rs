//! ZIP application failures with concrete parser and native I/O provenance.
use std::{error::Error, fmt, io, path::PathBuf};

use super::{ArchiveMemberName, ZipCompressionCode};

/// Concrete result shared by ZIP archive admission and member decoding.
///
/// ```
/// use fabelgeist_numpy_storage::{ZipArchive, zip::{Result, ZipReadError}};
/// let archive: Result<ZipArchive> = ZipArchive::from_bytes(Vec::new());
/// assert!(matches!(archive, Err(ZipReadError::MissingEndRecord)));
/// ```
/// Native I/O is a retained cause, not the owning parser error:
/// ```compile_fail
/// use fabelgeist_numpy_storage::ZipArchive;
/// let archive: std::result::Result<ZipArchive, std::io::Error> =
///     ZipArchive::from_bytes(Vec::new());
/// ```
pub type Result<T> = std::result::Result<T, ZipReadError>;

/// Native file operation whose original I/O failure is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipFileOperation {
    Open,
    Map,
}

/// A ZIP admission, lookup or decoding failure.
///
/// Member names and rejected compression codes retain their canonical owners.
/// Native paths are diagnostic provenance from a failed operating-system call;
/// they do not certify a successful file identity.
#[derive(Debug)]
pub enum ZipReadError {
    MissingEndRecord,
    /// Directory or end-record framing rejected by the structural parser.
    Archive {
        source: rawzip::Error,
    },
    Native {
        operation: ZipFileOperation,
        /// Original native OS path used by the failed open or map.
        native_path: PathBuf,
        source: io::Error,
    },
    MissingMember {
        /// Exact member query, retained without changing its spelling.
        name: ArchiveMemberName,
    },
    CorruptMember {
        /// Central-directory name retained at the failed serialized read.
        name: ArchiveMemberName,
        source: rawzip::Error,
    },
    MemberPastEnd {
        /// Member whose local framing or compressed payload is incomplete.
        name: ArchiveMemberName,
        source: rawzip::Error,
    },
    Inflate {
        /// Central-directory name retained at the failed native decompressor.
        name: ArchiveMemberName,
        source: io::Error,
    },
    UnsupportedCompression {
        /// Rejected method code, retained before successful payload decoding.
        code: ZipCompressionCode,
    },
}

impl fmt::Display for ZipReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEndRecord => {
                formatter.write_str("not a zip archive: no end-of-central-directory record")
            }
            Self::Archive { source } => write!(formatter, "invalid zip archive: {source}"),
            Self::Native {
                operation,
                native_path,
                ..
            } => match operation {
                ZipFileOperation::Open => write!(formatter, "opening {}", native_path.display()),
                ZipFileOperation::Map => {
                    write!(formatter, "memory-mapping {}", native_path.display())
                }
            },
            Self::MissingMember { name } => {
                write!(formatter, "archive has no member {name:?}")
            }
            Self::CorruptMember { name, .. } => {
                write!(formatter, "corrupt zip entry {name}")
            }
            Self::MemberPastEnd { name, .. } => write!(
                formatter,
                "zip entry {name} runs past the end of the archive"
            ),
            Self::Inflate { name, .. } => {
                write!(formatter, "inflating zip entry {name}")
            }
            Self::UnsupportedCompression { code } => {
                write!(formatter, "unsupported zip compression method {code}")
            }
        }?;
        if formatter.alternate()
            && matches!(self, Self::Native { .. } | Self::Inflate { .. })
            && let Some(source) = self.source()
        {
            write!(formatter, ": {source}")?;
        }
        Ok(())
    }
}
impl Error for ZipReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Native { source, .. } | Self::Inflate { source, .. } => Some(source),
            Self::Archive { source }
            | Self::CorruptMember { source, .. }
            | Self::MemberPastEnd { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<rawzip::Error> for ZipReadError {
    fn from(source: rawzip::Error) -> Self {
        match source.kind() {
            rawzip::ErrorKind::MissingEndOfCentralDirectory => Self::MissingEndRecord,
            _ => Self::Archive { source },
        }
    }
}

impl ZipReadError {
    pub(super) fn member(name: ArchiveMemberName, source: rawzip::Error) -> Self {
        match source.kind() {
            rawzip::ErrorKind::Eof => Self::MemberPastEnd { name, source },
            _ => Self::CorruptMember { name, source },
        }
    }
}

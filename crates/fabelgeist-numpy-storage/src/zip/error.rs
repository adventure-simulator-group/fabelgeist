//! Existing ZIP-reader failures and rejected native diagnostic provenance.
use std::{error::Error, fmt, io, path::PathBuf};

use super::{ArchiveMemberName, ZipCompressionCode};

/// Native file operation whose original I/O failure is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipFileOperation {
    Open,
    Map,
}

/// One of the ZIP reader's existing failures.
///
/// Member names and rejected compression codes retain their canonical owners.
/// Native paths are diagnostic provenance from a failed operating-system call;
/// they do not certify a successful file identity.
#[derive(Debug)]
pub enum ZipReadError {
    MissingEndRecord,
    Native {
        operation: ZipFileOperation,
        /// Original native OS path used by the failed open or map.
        native_path: PathBuf,
        source: io::Error,
    },
    MissingMember {
        /// Original native lookup text, retained for quoted diagnostics.
        name: ArchiveMemberName,
    },
    CorruptMember {
        /// Central-directory name retained at the failed serialized read.
        name: ArchiveMemberName,
    },
    MemberPastEnd {
        /// Central-directory name retained at the failed payload extent check.
        name: ArchiveMemberName,
    },
    Inflate {
        /// Central-directory name retained at the failed native decompressor.
        name: ArchiveMemberName,
        source: io::Error,
    },
    UnsupportedCompression {
        /// Rejected native wire method, before successful payload decoding.
        code: ZipCompressionCode,
    },
}

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

impl fmt::Display for ZipReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEndRecord => {
                formatter.write_str("not a zip archive: no end-of-central-directory record")
            }
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
            Self::CorruptMember { name } => {
                write!(formatter, "corrupt zip entry {name}")
            }
            Self::MemberPastEnd { name } => write!(
                formatter,
                "zip entry {name} runs past the end of the archive"
            ),
            Self::Inflate { name, .. } => {
                write!(formatter, "inflating zip entry {name}")
            }
            Self::UnsupportedCompression { code } => write!(
                formatter,
                "unsupported zip compression method {code}"
            ),
        }?;
        if formatter.alternate()
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
            _ => None,
        }
    }
}

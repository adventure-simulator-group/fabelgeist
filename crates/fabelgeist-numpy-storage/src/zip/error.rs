//! Existing ZIP-reader failures and rejected native diagnostic provenance.
use std::{error::Error, fmt, io, path::PathBuf};

/// Native file operation whose original I/O failure is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipFileOperation {
    Open,
    Map,
}

/// One of the ZIP reader's existing failures.
///
/// Paths, names and compression codes here are OS or serialized diagnostic
/// provenance from a failed operation. They do not admit file/member identities
/// or compression metadata and expose no primitive constructor/getter API.
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
        native_query: String,
    },
    CorruptMember {
        /// Central-directory name retained at the failed serialized read.
        native_name: String,
    },
    MemberPastEnd {
        /// Central-directory name retained at the failed payload extent check.
        native_name: String,
    },
    Inflate {
        /// Central-directory name retained at the failed native decompressor.
        native_name: String,
        source: io::Error,
    },
    UnsupportedCompression {
        /// Rejected native wire method, before successful payload decoding.
        native_code: u16,
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
            Self::MissingMember { native_query } => {
                write!(formatter, "archive has no member {native_query:?}")
            }
            Self::CorruptMember { native_name } => {
                write!(formatter, "corrupt zip entry {native_name}")
            }
            Self::MemberPastEnd { native_name } => write!(
                formatter,
                "zip entry {native_name} runs past the end of the archive"
            ),
            Self::Inflate { native_name, .. } => {
                write!(formatter, "inflating zip entry {native_name}")
            }
            Self::UnsupportedCompression { native_code } => write!(
                formatter,
                "unsupported zip compression method {native_code}"
            ),
        }
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

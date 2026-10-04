use super::{
    ArchiveMemberName, ArchiveMemberOrdinal, ZipChecksum, ZipCompressionCode, ZipRecordError,
};
use fabelgeist_fs::NativeFile;
use fabelgeist_storage::{StorageBoundsError, StorageByteLength};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveFileOperation {
    Open,
    Map,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zip64Field {
    UncompressedLength,
    CompressedLength,
    LocalHeaderPosition,
}
#[derive(Debug)]
pub enum ZipEndError {
    MissingEndRecord,
    Record(ZipRecordError),
    Bounds(StorageBoundsError),
    MultipleDisks,
    MissingZip64Locator,
    InvalidZip64RecordLength,
    DirectoryAfterEnd,
}
#[derive(Debug)]
pub enum ZipDirectoryError {
    Record(ZipRecordError),
    Bounds(StorageBoundsError),
    MissingZip64(Zip64Field),
}
#[derive(Debug)]
pub enum ZipMemberError {
    Record(ZipRecordError),
    Bounds(StorageBoundsError),
    Encrypted,
    UnsupportedCompression(ZipCompressionCode),
    Inflate(std::io::Error),
    Size {
        expected: StorageByteLength,
        actual: StorageByteLength,
    },
    Checksum {
        expected: ZipChecksum,
        actual: ZipChecksum,
    },
}
#[derive(Debug)]
pub enum ZipReadError {
    Native {
        file: NativeFile,
        operation: ArchiveFileOperation,
        source: std::io::Error,
    },
    End(ZipEndError),
    Directory {
        ordinal: ArchiveMemberOrdinal,
        source: ZipDirectoryError,
    },
    MissingMember(ArchiveMemberName),
    Member {
        name: ArchiveMemberName,
        source: ZipMemberError,
    },
}
impl std::fmt::Display for ZipReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native {
                file,
                operation,
                source,
            } => write!(
                f,
                "{operation:?} archive {}: {source}",
                file.as_ref().display()
            ),
            Self::End(source) => write!(f, "ZIP directory admission: {source}"),
            Self::Directory { ordinal, source } => {
                write!(f, "ZIP central member {ordinal}: {source}")
            }
            Self::MissingMember(name) => write!(f, "archive has no member {name:?}"),
            Self::Member { name, source } => write!(f, "ZIP member {name}: {source}"),
        }
    }
}
impl std::error::Error for ZipReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Native { source, .. } => Some(source),
            Self::End(source) => Some(source),
            Self::Directory { source, .. } => Some(source),
            Self::Member { source, .. } => Some(source),
            Self::MissingMember(_) => None,
        }
    }
}
impl std::fmt::Display for ZipEndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEndRecord => f.write_str("missing end-of-central-directory record"),
            Self::Record(source) => source.fmt(f),
            Self::Bounds(source) => source.fmt(f),
            Self::MultipleDisks => f.write_str("multiple-disk archives are unsupported"),
            Self::MissingZip64Locator => {
                f.write_str("saturated directory fields require a ZIP64 locator")
            }
            Self::InvalidZip64RecordLength => f.write_str("invalid ZIP64 end-record length"),
            Self::DirectoryAfterEnd => f.write_str("central directory overlaps its end record"),
        }
    }
}
impl std::error::Error for ZipEndError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Record(source) => Some(source),
            Self::Bounds(source) => Some(source),
            _ => None,
        }
    }
}
impl std::fmt::Display for ZipDirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Record(source) => source.fmt(f),
            Self::Bounds(source) => source.fmt(f),
            Self::MissingZip64(field) => write!(f, "missing ZIP64 {field:?}"),
        }
    }
}
impl std::error::Error for ZipDirectoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Record(source) => Some(source),
            Self::Bounds(source) => Some(source),
            _ => None,
        }
    }
}
impl std::fmt::Display for ZipMemberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Record(source) => source.fmt(f),
            Self::Bounds(source) => source.fmt(f),
            Self::Encrypted => f.write_str("encrypted members are unsupported"),
            Self::UnsupportedCompression(code) => {
                write!(f, "unsupported compression method {code}")
            }
            Self::Inflate(source) => write!(f, "deflate failed: {source}"),
            Self::Size { expected, actual } => write!(f, "decoded {actual}, expected {expected}"),
            Self::Checksum { expected, actual } => write!(f, "CRC32 {actual}, expected {expected}"),
        }
    }
}
impl std::error::Error for ZipMemberError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Record(source) => Some(source),
            Self::Bounds(source) => Some(source),
            Self::Inflate(source) => Some(source),
            _ => None,
        }
    }
}

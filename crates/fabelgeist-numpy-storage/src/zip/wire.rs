//! ZIP field tags and metadata admitted from their serialized scalar encodings.

use fabelgeist_storage::{StorageBoundsError, StorageByteLength, StorageByteOffset, StorageView};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZipRecordKind {
    EndDirectory,
    CentralMember,
    LocalMember,
    Zip64End,
    Zip64Locator,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZipSignature(u32);
impl From<u32> for ZipSignature {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl ZipRecordKind {
    pub(crate) fn signature(self) -> ZipSignature {
        ZipSignature(match self {
            Self::EndDirectory => 0x0605_4b50,
            Self::CentralMember => 0x0201_4b50,
            Self::LocalMember => 0x0403_4b50,
            Self::Zip64End => 0x0606_4b50,
            Self::Zip64Locator => 0x0706_4b50,
        })
    }
    pub(crate) fn width(self) -> StorageByteLength {
        StorageByteLength::from(match self {
            Self::EndDirectory => 22u64,
            Self::CentralMember => 46,
            Self::LocalMember => 30,
            Self::Zip64End => 56,
            Self::Zip64Locator => 20,
        })
    }
    pub(crate) fn admit(self, view: StorageView<'_>) -> Result<(), ZipRecordError> {
        view.portion(fabelgeist_storage::StorageByteSpan {
            offset: StorageByteOffset::default(),
            length: self.width(),
        })
        .map_err(ZipRecordError::Bounds)?;
        let actual = ZipSignature::from(
            view.decode_u32(StorageByteOffset::default())
                .map_err(ZipRecordError::Bounds)?,
        );
        if actual == self.signature() {
            Ok(())
        } else {
            Err(ZipRecordError::Signature {
                expected: self,
                actual,
            })
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZipRecordError {
    Bounds(StorageBoundsError),
    Signature {
        expected: ZipRecordKind,
        actual: ZipSignature,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZipCompressionCode(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZipCompression {
    Stored,
    Deflated,
    Unsupported(ZipCompressionCode),
}
impl From<u16> for ZipCompression {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Stored,
            8 => Self::Deflated,
            code => Self::Unsupported(ZipCompressionCode(code)),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZipProtection {
    Plain,
    Encrypted,
}
impl From<u16> for ZipProtection {
    fn from(flags: u16) -> Self {
        if flags & 1 == 0 {
            Self::Plain
        } else {
            Self::Encrypted
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZipChecksum(u32);
impl From<u32> for ZipChecksum {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<StorageView<'_>> for ZipChecksum {
    fn from(view: StorageView<'_>) -> Self {
        Self(crc32fast::hash(view.as_ref()))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveMemberCount(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveMemberOrdinal(u64);
impl From<u64> for ArchiveMemberCount {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl ArchiveMemberCount {
    pub(crate) fn ordinals(self) -> ArchiveMemberOrdinals {
        ArchiveMemberOrdinals {
            next: 0,
            end: self.0,
        }
    }
}
pub(crate) struct ArchiveMemberOrdinals {
    next: u64,
    end: u64,
}
impl Iterator for ArchiveMemberOrdinals {
    type Item = ArchiveMemberOrdinal;
    fn next(&mut self) -> Option<ArchiveMemberOrdinal> {
        if self.next == self.end {
            return None;
        }
        let slot = ArchiveMemberOrdinal(self.next);
        self.next += 1;
        Some(slot)
    }
}
impl std::fmt::Display for ZipSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}
impl std::fmt::Display for ZipCompressionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Display for ZipChecksum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}
impl std::fmt::Display for ArchiveMemberOrdinal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Display for ZipRecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bounds(source) => source.fmt(f),
            Self::Signature { expected, actual } => {
                write!(f, "expected {expected:?}, found signature {actual}")
            }
        }
    }
}
impl std::error::Error for ZipRecordError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bounds(source) => Some(source),
            Self::Signature { .. } => None,
        }
    }
}

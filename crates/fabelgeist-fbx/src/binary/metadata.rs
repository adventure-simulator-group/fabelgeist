use fabelgeist_storage::{StorageByteLength, StorageByteOffset};

#[derive(Clone, Copy)]
pub(super) enum HeaderFormat {
    Narrow,
    Wide,
}
impl HeaderFormat {
    pub(super) fn record_width(self) -> StorageByteLength {
        StorageByteLength::from(match self {
            Self::Narrow => 13u64,
            Self::Wide => 25,
        })
    }
    pub(super) fn word_width(self) -> StorageByteLength {
        StorageByteLength::from(match self {
            Self::Narrow => 4u64,
            Self::Wide => 8,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FbxVersion(u32);
impl From<u32> for FbxVersion {
    fn from(version: u32) -> Self {
        Self(version)
    }
}
impl FbxVersion {
    const WIDE_HEADER_VERSION: Self = Self(7500);
    pub(super) fn format(self) -> HeaderFormat {
        if self.0 >= Self::WIDE_HEADER_VERSION.0 {
            HeaderFormat::Wide
        } else {
            HeaderFormat::Narrow
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FbxPropertyCount(pub(super) u64);
impl From<u64> for FbxPropertyCount {
    fn from(count: u64) -> Self {
        Self(count)
    }
}
impl FbxPropertyCount {
    pub(super) fn minimum_bytes(self) -> StorageByteLength {
        StorageByteLength::from(self.0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FbxArrayCount(pub(super) u32);
impl From<u32> for FbxArrayCount {
    fn from(count: u32) -> Self {
        Self(count)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FbxArrayEncodingCode(pub(super) u32);
impl From<u32> for FbxArrayEncodingCode {
    fn from(code: u32) -> Self {
        Self(code)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FbxPropertyTag(pub(super) u8);
impl From<u8> for FbxPropertyTag {
    fn from(tag: u8) -> Self {
        Self(tag)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FbxArrayKind {
    Float32,
    Float64,
    Integer32,
    Integer64,
    Boolean,
}
impl FbxArrayKind {
    pub(super) fn payload_length(self, count: FbxArrayCount) -> StorageByteLength {
        let width = match self {
            Self::Boolean => 1,
            Self::Float32 | Self::Integer32 => 4,
            Self::Float64 | Self::Integer64 => 8,
        };
        StorageByteLength::from(u64::from(count.0) * width)
    }
}
pub(super) struct NodeHeader {
    pub end: StorageByteOffset,
    pub count: FbxPropertyCount,
    pub properties: StorageByteLength,
    pub name: StorageByteLength,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FbxSection {
    Header,
    NodeMetadata,
    NodeName,
    PropertyTag,
    Scalar,
    ArrayMetadata,
    ArrayPayload,
    PropertyPayload,
}

#[derive(Clone, Copy)]
pub(super) enum ScalarKind {
    Integer16,
    Boolean,
    Integer32,
    Integer64,
    Float32,
    Float64,
}
impl ScalarKind {
    pub(super) fn width(self) -> StorageByteLength {
        StorageByteLength::from(match self {
            Self::Boolean => 1u64,
            Self::Integer16 => 2,
            Self::Integer32 | Self::Float32 => 4,
            Self::Integer64 | Self::Float64 => 8,
        })
    }
}
impl FbxPropertyTag {
    pub(super) fn scalar_kind(self) -> Option<ScalarKind> {
        Some(match self.0 {
            b'Y' => ScalarKind::Integer16,
            b'C' => ScalarKind::Boolean,
            b'I' => ScalarKind::Integer32,
            b'L' => ScalarKind::Integer64,
            b'F' => ScalarKind::Float32,
            b'D' => ScalarKind::Float64,
            _ => return None,
        })
    }
}

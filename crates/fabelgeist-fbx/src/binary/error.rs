use super::metadata::{
    FbxArrayCount, FbxArrayEncodingCode, FbxArrayKind, FbxPropertyCount, FbxPropertyTag, FbxSection,
};
use fabelgeist_storage::{StorageBoundsError, StorageByteLength, StorageByteOffset};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FbxFormatViolation {
    Ascii,
    NotBinary,
}
#[derive(Debug)]
pub enum FbxDecodeError {
    Format(FbxFormatViolation),
    Bounds {
        section: FbxSection,
        source: StorageBoundsError,
    },
    NodeExtent {
        end: StorageByteOffset,
        minimum: StorageByteOffset,
        limit: StorageByteOffset,
    },
    PropertyCount {
        count: FbxPropertyCount,
        available: StorageByteLength,
    },
    PropertyLength {
        declared: StorageByteLength,
        actual: StorageByteLength,
    },
    ArrayEncoding(FbxArrayEncodingCode),
    Inflate {
        kind: FbxArrayKind,
        count: FbxArrayCount,
        source: std::io::Error,
    },
    ArrayShort {
        kind: FbxArrayKind,
        count: FbxArrayCount,
        actual: StorageByteLength,
    },
    PropertyTag {
        tag: FbxPropertyTag,
        at: StorageByteOffset,
    },
}
impl std::fmt::Display for FbxDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Format(FbxFormatViolation::Ascii) => {
                f.write_str("this is an ASCII FBX file; re-export it as binary FBX")
            }
            Self::Format(FbxFormatViolation::NotBinary) => f.write_str("not a binary FBX file"),
            Self::Bounds { section, source } => write!(f, "FBX {section:?}: {source}"),
            Self::NodeExtent {
                end,
                minimum,
                limit,
            } => write!(f, "FBX node end {end} is outside {minimum} through {limit}"),
            Self::PropertyCount { count, available } => {
                write!(f, "FBX {count:?} exceeds property block {available}")
            }
            Self::PropertyLength { declared, actual } => write!(
                f,
                "FBX property block declares {declared}, consumed {actual}"
            ),
            Self::ArrayEncoding(code) => write!(f, "unsupported FBX array encoding {code:?}"),
            Self::Inflate {
                kind,
                count,
                source,
            } => write!(f, "inflating FBX {kind:?} array {count:?}: {source}"),
            Self::ArrayShort {
                kind,
                count,
                actual,
            } => write!(f, "FBX {kind:?} array {count:?} is short at {actual}"),
            Self::PropertyTag { tag, at } => write!(f, "unknown FBX property {tag:?} at {at}"),
        }
    }
}
impl std::error::Error for FbxDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bounds { source, .. } => Some(source),
            Self::Inflate { source, .. } => Some(source),
            _ => None,
        }
    }
}

//! Supported array encodings admitted from the four-byte wire metadata field.

use derive_more::{Display, Error};

/// An array encoding code this binary FBX reader does not support.
#[derive(Clone, Copy, Debug, Display, Error, PartialEq, Eq)]
#[display("unsupported FBX array encoding code {reported_code}")]
pub struct UnsupportedFbxArrayEncoding {
    /// The raw 32-bit encoding code reported in the array property header.
    pub reported_code: u32,
}

/// An array encoding supported by this binary FBX reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FbxArrayEncoding {
    Uncompressed,
    Zlib,
}

impl TryFrom<u32> for FbxArrayEncoding {
    type Error = UnsupportedFbxArrayEncoding;

    fn try_from(reported_code: u32) -> Result<Self, Self::Error> {
        match reported_code {
            0 => Ok(Self::Uncompressed),
            1 => Ok(Self::Zlib),
            _ => Err(UnsupportedFbxArrayEncoding { reported_code }),
        }
    }
}

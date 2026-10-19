//! Supported array encodings admitted from the four-byte wire metadata field.

use derive_more::{Display, Error, From};
use num_enum::TryFromPrimitive;

type Result<T> = std::result::Result<T, UnsupportedFbxArrayEncoding>;

/// An array encoding code this binary FBX reader does not support.
#[derive(Clone, Copy, Debug, Display, Error, From, PartialEq, Eq)]
#[display("unsupported FBX array encoding code {reported_code}")]
pub struct UnsupportedFbxArrayEncoding {
    /// The raw 32-bit encoding code reported in the array property header.
    pub reported_code: u32,
}

/// An array encoding supported by this binary FBX reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
#[num_enum(error_type(
    name = UnsupportedFbxArrayEncoding,
    constructor = UnsupportedFbxArrayEncoding::from
))]
pub(super) enum FbxArrayEncoding {
    Uncompressed = 0,
    Zlib = 1,
}

impl FbxArrayEncoding {
    pub(super) fn try_from_le_bytes(bytes: [u8; size_of::<u32>()]) -> Result<Self> {
        Self::try_from(u32::from_le_bytes(bytes))
    }
}

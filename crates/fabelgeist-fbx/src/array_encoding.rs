//! Supported array encodings admitted from the four-byte wire metadata field.

use derive_more::{Display, Error, TryFrom};

/// An array encoding code this binary FBX reader does not support.
#[derive(Clone, Copy, Debug, Display, Error, PartialEq, Eq)]
#[display("unsupported FBX array encoding code {reported_code}")]
pub struct UnsupportedFbxArrayEncoding {
    /// The raw 32-bit encoding code reported in the array property header.
    pub reported_code: u32,
}

/// An array encoding supported by this binary FBX reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFrom)]
#[repr(u32)]
#[try_from(repr)]
pub(super) enum FbxArrayEncoding {
    Uncompressed = 0,
    Zlib = 1,
}

//! Array encoding identity admitted from its four-byte wire metadata field.

/// An open encoding code carried from property admission into array decoding.
/// Zero requests an uncompressed payload. Every nonzero value retains the
/// reader's existing Zlib policy; this is not a supported-code range check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FbxArrayEncodingCode(u32);

impl From<u32> for FbxArrayEncodingCode {
    fn from(code: u32) -> Self {
        Self(code)
    }
}

impl FbxArrayEncodingCode {
    const UNCOMPRESSED: Self = Self(0);

    pub(super) fn is_uncompressed(self) -> bool {
        self == Self::UNCOMPRESSED
    }
}

//! Property tag identities admitted from native one-byte wire fields.

/// An open property tag carried through scalar and array dispatch.
/// Unknown values are retained until the reader's existing rejection point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FbxPropertyTag(u8);

impl From<u8> for FbxPropertyTag {
    fn from(tag: u8) -> Self {
        Self(tag)
    }
}

impl FbxPropertyTag {
    pub(super) const INTEGER16: Self = Self(b'Y');
    pub(super) const BOOLEAN: Self = Self(b'C');
    pub(super) const INTEGER32: Self = Self(b'I');
    pub(super) const INTEGER64: Self = Self(b'L');
    pub(super) const FLOAT32: Self = Self(b'F');
    pub(super) const FLOAT64: Self = Self(b'D');
    pub(super) const ARRAY_FLOAT32: Self = Self(b'f');
    pub(super) const ARRAY_FLOAT64: Self = Self(b'd');
    pub(super) const ARRAY_INTEGER32: Self = Self(b'i');
    pub(super) const ARRAY_INTEGER64: Self = Self(b'l');
    pub(super) const ARRAY_BOOLEAN: Self = Self(b'b');
    pub(super) const STRING: Self = Self(b'S');
    pub(super) const RAW_BYTES: Self = Self(b'R');

    /// Preserve the reader's character representation in unknown-tag errors.
    pub(super) fn diagnostic_character(self) -> char {
        char::from(self.0)
    }
}

//! Declared array cardinality and stored payload length from the wire header.

use derive_more::From;

/// An encoded element count, distinct from a stored payload byte length.
#[derive(Clone, Copy, Debug, From, PartialEq, Eq)]
pub struct FbxArrayCount(u32);

/// Length of the stored array payload, whether raw or Zlib-compressed.
pub(super) struct FbxArrayPayloadLength(u32);

impl FbxArrayCount {
    pub(super) fn from_le_bytes(bytes: [u8; size_of::<u32>()]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }

    /// Host-sized bound for native buffer iteration.
    pub(super) fn native_len(self) -> usize {
        self.0 as usize
    }

    /// Native buffer capacity/check using the selected decoder's input width.
    /// Preserves host multiplication without adding a bound or overflow policy.
    pub(super) fn native_byte_len<const ELEMENT_BYTES: usize>(self) -> usize {
        self.native_len() * ELEMENT_BYTES
    }
}

impl FbxArrayPayloadLength {
    pub(super) fn from_le_bytes(bytes: [u8; size_of::<u32>()]) -> Self {
        Self(u32::from_le_bytes(bytes))
    }

    /// Host-sized byte length at the reader's native slice boundary.
    pub(super) fn native_len(self) -> usize {
        self.0 as usize
    }
}

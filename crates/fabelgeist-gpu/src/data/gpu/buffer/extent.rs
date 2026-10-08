//! Byte quantities at buffer allocation and queue-write boundaries.

/// A logical buffer extent, distinct from an address within that buffer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferByteLength(u64);

/// A byte address within a buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferByteOffset(u64);

impl BufferByteOffset {
    pub const START: Self = Self(0);

    pub fn checked_after(self, length: BufferByteLength) -> Option<Self> {
        self.0.checked_add(length.0).map(Self)
    }

    /// Whether this upload end is at or before the logical byte boundary.
    ///
    /// The end is inclusive for admission, so an empty upload can end exactly
    /// at the logical length. Native allocation, usage and alignment remain
    /// separate checks at the queue boundary.
    pub fn is_end_within(self, length: BufferByteLength) -> bool {
        self.0 <= length.0
    }
}

impl From<u64> for BufferByteLength {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}
impl From<usize> for BufferByteLength {
    fn from(bytes: usize) -> Self {
        Self(bytes as u64)
    }
}
impl From<u64> for BufferByteOffset {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}
impl From<BufferByteLength> for u64 {
    fn from(bytes: BufferByteLength) -> Self {
        bytes.0
    }
}
impl From<BufferByteOffset> for u64 {
    fn from(bytes: BufferByteOffset) -> Self {
        bytes.0
    }
}
impl std::fmt::Display for BufferByteLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Display for BufferByteOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

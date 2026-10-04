//! Buffer byte quantities and checked readback copy ranges.

use std::fmt;

use super::ReadbackError;

/// A buffer's logical byte length, including an empty logical result.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferByteLength(u64);

/// A byte address within a buffer; distinct from a length or element index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferByteOffset(u64);

/// A checked end address for one staged buffer's logical bytes.
#[derive(Clone, Copy, Debug)]
pub struct ReadbackRange {
    pub(super) start: BufferByteOffset,
    pub(super) end: BufferByteOffset,
    pub(super) length: BufferByteLength,
}

impl BufferByteLength {
    /// Round a copy extent to WebGPU's required buffer-copy alignment.
    pub fn copy_aligned(self) -> Result<Self, ReadbackError> {
        let alignment = wgpu::COPY_BUFFER_ALIGNMENT;
        let remainder = self.0 % alignment;
        if remainder == 0 {
            return Ok(self);
        }
        self.0
            .checked_add(alignment - remainder)
            .map(Self)
            .ok_or(ReadbackError::CopyAlignmentOverflow(self))
    }
}

impl std::ops::Add for BufferByteLength {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl BufferByteOffset {
    pub const START: Self = Self(0);

    pub fn checked_after(self, length: BufferByteLength) -> Option<Self> {
        self.0.checked_add(length.0).map(Self)
    }

    pub fn after(self, length: BufferByteLength) -> Result<Self, ReadbackError> {
        self.checked_after(length)
            .ok_or(ReadbackError::RangeOverflow {
                start: self,
                length,
            })
    }
}

impl ReadbackRange {
    pub fn new(start: BufferByteOffset, length: BufferByteLength) -> Result<Self, ReadbackError> {
        Ok(Self {
            start,
            end: start.after(length)?,
            length,
        })
    }

    pub fn start(self) -> BufferByteOffset {
        self.start
    }

    pub fn length(self) -> BufferByteLength {
        self.length
    }
}

impl From<u32> for BufferByteLength {
    fn from(bytes: u32) -> Self {
        Self(bytes as u64)
    }
}
impl From<u32> for BufferByteOffset {
    fn from(bytes: u32) -> Self {
        Self(bytes as u64)
    }
}

impl From<u64> for BufferByteOffset {
    fn from(bytes: u64) -> Self {
        Self(bytes)
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

impl From<BufferByteLength> for u64 {
    fn from(length: BufferByteLength) -> Self {
        length.0
    }
}

impl TryFrom<BufferByteLength> for usize {
    type Error = ReadbackError;

    fn try_from(length: BufferByteLength) -> Result<Self, ReadbackError> {
        match Self::try_from(length.0) {
            Ok(bytes) => Ok(bytes),
            Err(source) => Err(ReadbackError::HostLength { length, source }),
        }
    }
}

impl From<BufferByteOffset> for u64 {
    fn from(offset: BufferByteOffset) -> Self {
        offset.0
    }
}

impl TryFrom<BufferByteOffset> for usize {
    type Error = ReadbackError;

    fn try_from(offset: BufferByteOffset) -> Result<Self, ReadbackError> {
        match Self::try_from(offset.0) {
            Ok(bytes) => Ok(bytes),
            Err(source) => Err(ReadbackError::HostOffset { offset, source }),
        }
    }
}

impl fmt::Display for BufferByteLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for BufferByteOffset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_rounding_and_range_overflow_are_checked() {
        let length = BufferByteLength::from(3u64);
        let first = ReadbackRange::new(BufferByteOffset::START, length).unwrap();
        let next = first.start().after(length.copy_aligned().unwrap()).unwrap();
        assert_eq!(u64::from(next), 4);
        let last = BufferByteOffset::START
            .after(BufferByteLength::from(u64::MAX))
            .unwrap();
        assert!(matches!(
            last.after(BufferByteLength::from(1u64)),
            Err(ReadbackError::RangeOverflow { .. })
        ));
        assert!(matches!(
            BufferByteLength::from(u64::MAX).copy_aligned(),
            Err(ReadbackError::CopyAlignmentOverflow(_))
        ));
    }
}

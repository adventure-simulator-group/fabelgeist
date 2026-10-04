//! Checked positions, lengths, and borrowed views in a serialized storage file.

use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct StorageByteLength(u64);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct StorageByteOffset(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageByteSpan {
    pub offset: StorageByteOffset,
    pub length: StorageByteLength,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageBoundsError {
    Overflow(StorageByteSpan),
    Outside {
        requested: StorageByteSpan,
        available: StorageByteLength,
    },
}
impl From<u64> for StorageByteLength {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<usize> for StorageByteLength {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}
impl From<u64> for StorageByteOffset {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl StorageByteOffset {
    pub fn distance_from(self, start: Self) -> Option<StorageByteLength> {
        self.0.checked_sub(start.0).map(StorageByteLength)
    }
    /// Advances by a byte length, keeping addresses distinct from quantities.
    ///
    /// ```compile_fail
    /// use fabelgeist_storage::StorageByteOffset;
    /// let address = StorageByteOffset::from(4);
    /// address.advance(StorageByteOffset::from(2));
    /// ```
    pub fn advance(self, length: StorageByteLength) -> Result<Self, StorageBoundsError> {
        self.0
            .checked_add(length.0)
            .map(Self)
            .ok_or(StorageBoundsError::Overflow(StorageByteSpan {
                offset: self,
                length,
            }))
    }
    pub fn preceding(self, length: StorageByteLength) -> Option<Self> {
        self.0.checked_sub(length.0).map(Self)
    }
}
impl From<StorageByteLength> for u64 {
    fn from(length: StorageByteLength) -> Self {
        length.0
    }
}
impl StorageByteLength {
    pub fn end_offset(self) -> StorageByteOffset {
        StorageByteOffset(self.0)
    }
    pub fn checked_add(self, other: Self) -> Result<Self, StorageBoundsError> {
        StorageByteOffset(self.0)
            .advance(other)
            .map(|end: StorageByteOffset| -> Self { Self(end.0) })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct StorageView<'a>(&'a [u8]);
impl<'a> From<&'a [u8]> for StorageView<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
}
impl AsRef<[u8]> for StorageView<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}
impl<'a> StorageView<'a> {
    pub fn tail_at(self, offset: StorageByteOffset) -> Result<Self, StorageBoundsError> {
        if offset.0 > self.length().0 {
            return Err(StorageBoundsError::Outside {
                requested: StorageByteSpan {
                    offset,
                    length: StorageByteLength::default(),
                },
                available: self.length(),
            });
        }
        Ok(Self(&self.0[offset.0 as usize..]))
    }
    pub fn length(self) -> StorageByteLength {
        StorageByteLength::from(self.0.len())
    }
    pub fn portion(self, span: StorageByteSpan) -> Result<Self, StorageBoundsError> {
        let end = span.offset.advance(span.length)?;
        if end.0 > self.length().0 {
            return Err(StorageBoundsError::Outside {
                requested: span,
                available: self.length(),
            });
        }
        // Checked against an actual slice length before SDK indexing/conversion.
        Ok(Self(&self.0[span.offset.0 as usize..end.0 as usize]))
    }
    pub fn decode_u16(self, offset: StorageByteOffset) -> Result<u16, StorageBoundsError> {
        let bytes = self.portion(StorageByteSpan {
            offset,
            length: StorageByteLength(2),
        })?;
        Ok(u16::from_le_bytes(
            bytes.0.try_into().expect("checked field width"),
        ))
    }
    pub fn decode_u32(self, offset: StorageByteOffset) -> Result<u32, StorageBoundsError> {
        let bytes = self.portion(StorageByteSpan {
            offset,
            length: StorageByteLength(4),
        })?;
        Ok(u32::from_le_bytes(
            bytes.0.try_into().expect("checked field width"),
        ))
    }
    pub fn decode_u64(self, offset: StorageByteOffset) -> Result<u64, StorageBoundsError> {
        let bytes = self.portion(StorageByteSpan {
            offset,
            length: StorageByteLength(8),
        })?;
        Ok(u64::from_le_bytes(
            bytes.0.try_into().expect("checked field width"),
        ))
    }
}
impl fmt::Display for StorageByteLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bytes", self.0)
    }
}
impl fmt::Display for StorageByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "byte {}", self.0)
    }
}
impl fmt::Display for StorageBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow(span) => write!(
                f,
                "{} plus {} overflows the storage address",
                span.offset, span.length
            ),
            Self::Outside {
                requested,
                available,
            } => write!(
                f,
                "{} of {} exceeds {available}",
                requested.offset, requested.length
            ),
        }
    }
}
impl std::error::Error for StorageBoundsError {}

#[cfg(test)]
mod tests;

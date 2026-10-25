//! Native spacing of consecutive storage-array elements.

use super::BufferByteLength;
use crate::data::PassParameter;
use crate::data::gpu::compute::{InvocationCount, WorkgroupCount};
use std::num::{NonZeroU32, TryFromIntError};
use std::{error::Error, fmt};

/// Byte spacing of records in a native storage array, including padding.
///
/// This is independent of an element's payload size and the array's byte
/// extent. Admit the native array stride from the shader's checked layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferElementStride(NonZeroU32);

/// Complete records in a native storage array.
///
/// Admit a population at its native count boundary, or decode a byte extent
/// using its stride. This quantity does not certify storage contents or identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferElementCount(u32);

/// A storage-array stride cannot be zero.
#[derive(Clone, Copy, Debug)]
pub struct BufferElementStrideError;

impl fmt::Display for BufferElementStrideError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("native storage-array stride must be nonzero")
    }
}

impl Error for BufferElementStrideError {}

impl TryFrom<u32> for BufferElementStride {
    type Error = BufferElementStrideError;

    fn try_from(bytes: u32) -> Result<Self, Self::Error> {
        NonZeroU32::new(bytes).map(Self).ok_or(BufferElementStrideError)
    }
}

impl BufferElementStride {
    pub fn byte_length(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0.get()))
    }
}

impl BufferElementCount {
    /// Scale a native record population without wrapping its address space.
    pub fn checked_mul(self, factor: u32) -> Option<Self> {
        self.0.checked_mul(factor).map(Self)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn is_single(self) -> bool {
        self.0 == 1
    }

    pub fn byte_length(self, stride: BufferElementStride) -> BufferByteLength {
        // Both record count and native stride are u32 quantities.
        BufferByteLength::from(u64::from(self.0) * u64::from(stride.byte_length()))
    }

    /// Decode a logical byte extent using the admitted native array stride.
    /// Trailing bytes do not form another complete record.
    pub fn from_extent(
        bytes: BufferByteLength,
        stride: BufferElementStride,
    ) -> Result<Self, TryFromIntError> {
        let records = u64::from(bytes) / u64::from(stride.byte_length());
        u32::try_from(records).map(Self)
    }
}

impl From<u32> for BufferElementCount {
    fn from(elements: u32) -> Self {
        Self(elements)
    }
}

/// Encode a native record population in a shader's unsigned count field.
impl From<BufferElementCount> for PassParameter {
    fn from(elements: BufferElementCount) -> Self {
        Self::Unsigned(elements.0)
    }
}

/// Native host indexing on the supported 32-bit and 64-bit GPU targets.
impl From<BufferElementCount> for usize {
    fn from(elements: BufferElementCount) -> Self {
        elements.0 as usize
    }
}

impl From<BufferElementCount> for InvocationCount {
    fn from(elements: BufferElementCount) -> Self {
        Self::from(elements.0)
    }
}

/// One stored total for each x-axis workgroup.
impl From<WorkgroupCount> for BufferElementCount {
    fn from(groups: WorkgroupCount) -> Self {
        Self(groups.native_groups())
    }
}

/// A launch that assigns one x-axis workgroup to each stored record.
impl From<BufferElementCount> for WorkgroupCount {
    fn from(elements: BufferElementCount) -> Self {
        Self::from(elements.0)
    }
}

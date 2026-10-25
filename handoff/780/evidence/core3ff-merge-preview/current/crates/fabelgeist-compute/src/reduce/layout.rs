//! Live reduction populations and the reflected native array stride.

use crate::prelude::*;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use std::num::NonZeroU32;

pub(super) const REDUCTION_WORKGROUP_WIDTH: u32 = 64;

/// Values consumed by one reduction pass, independently of scratch capacity.
#[derive(Clone, Copy, Debug)]
pub(super) struct ReductionElements(u32);

/// Byte spacing of records in the shader's output array, including padding.
#[derive(Clone, Copy, Debug)]
pub(super) struct ReductionStride(NonZeroU32);

impl ReductionElements {
    pub(super) fn from_resource(input: &GpuResource, stride: ReductionStride) -> Result<Self> {
        let elements = match input {
            GpuResource::Buffer(buffer) => u64::from(buffer.size) / u64::from(stride.0.get()),
            GpuResource::Texture2d(texture) => u64::from(texture.size.0) * u64::from(texture.size.1),
            GpuResource::Texture3d(texture) => u64::from(texture.size.0)
                .checked_mul(u64::from(texture.size.1))
                .and_then(|plane| plane.checked_mul(u64::from(texture.size.2)))
                .ok_or_else(|| anyhow!("Reduction texture population overflows"))?,
        };
        Ok(Self(u32::try_from(elements)?))
    }

    pub(super) fn is_finished(self) -> bool {
        self.0 <= 1
    }

    /// One output record is written by each workgroup of the current pass.
    pub(super) fn reduced(self) -> Self {
        Self(self.0.div_ceil(REDUCTION_WORKGROUP_WIDTH))
    }

    pub(super) fn covering_workgroups(self) -> WorkgroupGrid {
        WorkgroupGrid::from([self.0.div_ceil(REDUCTION_WORKGROUP_WIDTH), 1, 1])
    }

    pub(super) fn byte_length(self, stride: ReductionStride) -> BufferByteLength {
        // Both operands are admitted native u32 values, so their product fits u64.
        (u64::from(self.0) * u64::from(stride.0.get())).into()
    }
}

impl ReductionStride {
    pub(super) fn from_native_bytes(bytes: u32) -> Result<Self> {
        NonZeroU32::new(bytes)
            .map(Self)
            .ok_or_else(|| anyhow!("Reduction array stride must be nonzero"))
    }

    pub(super) fn byte_length(self) -> BufferByteLength {
        u64::from(self.0.get()).into()
    }
}

//! Host collider cardinality and the retained native allocation are distinct.
use crate::collider::COLLIDER_BYTES;
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferCreationError, BufferDefinition, PassParameters, WgpuContext,
};

/// Full host list cardinality, including rejected replacement diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColliderCount(usize);
impl From<usize> for ColliderCount {
    fn from(colliders: usize) -> Self {
        Self(colliders)
    }
}
impl std::fmt::Display for ColliderCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl ColliderCount {
    pub(super) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("collider_count".into(), (self.0 as u32).into());
    }
}

/// Allocated native collider records, retaining the existing unsigned width.
///
/// ```compile_fail
/// use fabelgeist_physics::{ColliderCapacity, ColliderCount};
/// fn held(_: ColliderCount) {}
/// held(ColliderCapacity::INITIAL);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColliderCapacity(u32);
impl std::fmt::Display for ColliderCapacity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl ColliderCapacity {
    pub const INITIAL: Self = Self(16);

    pub(super) fn for_count(count: ColliderCount) -> Self {
        Self(count.0.next_power_of_two() as u32)
    }
    pub(super) fn fit(self, count: ColliderCount) -> ColliderCapacityFit {
        if count.0 as u32 > self.0 {
            ColliderCapacityFit::GrowthRequired
        } else {
            ColliderCapacityFit::Retained
        }
    }
    pub(super) fn byte_length(self) -> BufferByteLength {
        (self.0 as u64 * COLLIDER_BYTES as u64).into()
    }
    pub(super) fn allocate(self, context: &WgpuContext) -> Result<Buffer, BufferCreationError> {
        Buffer::new(
            context,
            self.byte_length(),
            BufferDefinition::storage().with_label("colliders".into()),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ColliderCapacityFit {
    Retained,
    GrowthRequired,
}

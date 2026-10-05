//! Full host list cardinality and native allocation targets retain distinct roles.
use crate::collider::COLLIDER_BYTES;
use fabelgeist_gpu::prelude::{BufferByteLength, PassParameters};

/// Full host length, retained through update admission and its diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ColliderCount(usize);
impl From<usize> for ColliderCount {
    fn from(colliders: usize) -> Self {
        Self(colliders)
    }
}
impl std::fmt::Display for ColliderCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl ColliderCount {
    pub(super) const EMPTY: Self = Self(0);
    pub(super) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("collider_count", self.0 as u32);
    }
}

/// Native capacity selected by allocation policy, without proving a live buffer.
/// Growth retains host rounding followed by unsigned-word truncation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ColliderCapacity(u32);
impl ColliderCapacity {
    pub(super) const INITIAL: Self = Self(16);
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
        (u64::from(self.0) * COLLIDER_BYTES as u64).into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ColliderCapacityFit {
    Retained,
    GrowthRequired,
}

#[cfg(test)]
mod tests;

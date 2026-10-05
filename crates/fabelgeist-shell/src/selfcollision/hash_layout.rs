//! Particle allocation determines the spatial hash's bucket and sentinel extents.
use fabelgeist_gpu::prelude::{BufferByteLength, PassParameter};
use fabelgeist_xpbd::ParticleCapacity;

#[derive(Clone, Copy)]
pub(super) struct CollisionTableSize(u32);
impl CollisionTableSize {
    const MINIMUM_BUCKETS: Self = Self(64);
    const BUCKETS_PER_PARTICLE: u32 = 2;
    pub(super) fn for_capacity(capacity: ParticleCapacity) -> Self {
        // Roughly two buckets per particle keeps occupancy low. Native unsigned
        // overflow behavior is retained; this does not admit device limits.
        Self(
            (u32::from(capacity) * Self::BUCKETS_PER_PARTICLE)
                .next_power_of_two()
                .max(Self::MINIMUM_BUCKETS.0),
        )
    }
    pub(super) fn starts_bytes(self) -> BufferByteLength {
        ((u64::from(self.0) + 1) * std::mem::size_of::<u32>() as u64).into()
    }
}
impl From<CollisionTableSize> for PassParameter {
    fn from(size: CollisionTableSize) -> Self {
        Self::Unsigned(size.0)
    }
}
impl From<CollisionTableSize> for u32 {
    fn from(size: CollisionTableSize) -> Self {
        size.0
    }
}

#[cfg(test)]
mod tests;

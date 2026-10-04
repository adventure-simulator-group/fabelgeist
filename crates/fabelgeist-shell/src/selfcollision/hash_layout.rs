//! Particle capacity determines the spatial hash's bucket layout.
use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount, PassParameters};
use fabelgeist_xpbd::ParticleCapacity;

pub(super) struct CollisionTableSize(u32);
impl CollisionTableSize {
    pub(super) fn for_capacity(capacity: ParticleCapacity) -> Self {
        let particles = usize::from(capacity.count()) as u32;
        Self((particles * 2).next_power_of_two().max(64))
    }
    pub(super) fn starts_bytes(&self) -> BufferByteLength {
        ((self.0 as u64 + 1) * 4).into()
    }
    pub(super) fn clear_invocations(&self) -> InvocationCount {
        (self.0 + 1).into()
    }
    pub(super) fn bind(&self, parameters: &mut PassParameters) {
        parameters.insert("table_size".into(), self.0.into());
    }
}

//! Transaction entropy projected into the existing strategic surgery score.
use crate::character::CharacterId;
use fabelgeist_determinism::{Seed, StreamId};
use spacetimedb::ReducerContext;

const PROJECTILE_DEPTH: StreamId = StreamId::new("surgery.projectile-depth");
const DEPTH_SAMPLE_STEPS: usize = 151;
const DEPTH_SAMPLES_PER_POINT: f32 = 100.0;

/// Dimensionless extraction difficulty, not a measured penetration depth.
/// Construction samples the existing hundredth-point lattice in `[0, 1.5]`.
pub(super) struct ProjectileDepthContribution(f32);

impl ProjectileDepthContribution {
    pub fn sample(context: &ReducerContext, character: CharacterId) -> Self {
        Self(
            PROJECTILE_DEPTH
                .rng(context.random::<Seed>(), &[character.get()])
                .index(DEPTH_SAMPLE_STEPS) as f32
                / DEPTH_SAMPLES_PER_POINT,
        )
    }

    /// Native scalar handoff to core's dimensionless surgery scoring kernel.
    pub const fn points(self) -> f32 {
        self.0
    }
}

//! One domain-derived seed owns the fixed draw slots of one visual tuft.
use bevy::prelude::*;

pub(super) struct TuftVariation {
    pub jitter: Vec2,
    pub species: u64,
    pub rotation: f32,
    pub shader_seed: u32,
}

impl TuftVariation {
    pub fn new(seed: fabelgeist_determinism::Seed) -> Self {
        let mut rng = seed.rng();
        // Draw every slot before coverage rejection. Nearby tufts, density
        // changes and species selection cannot consume one another's draws.
        Self {
            jitter: Vec2::new(rng.inclusive_unit_f32(), rng.inclusive_unit_f32())
                - Vec2::splat(0.5),
            species: rng.next_u64(),
            rotation: rng.inclusive_unit_f32() * core::f32::consts::TAU,
            shader_seed: rng.next_u64() as u32,
        }
    }
}

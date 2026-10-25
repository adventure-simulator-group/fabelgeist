//! Refinement-local memoization of exact deterministic lattice draws.
//! Cache residency never changes noise framing or interpolation arithmetic.
use super::{lerp, streams};
use bevy::math::Vec2;
use std::{cell::RefCell, collections::HashMap};

// Bound temporary CPU residency independently of scene extent. Clearing the
// memo table only causes deterministic draws to be recomputed.
const MAXIMUM_RETAINED_LATTICE_CONTROLS: usize = 65_536;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct LatticeControl {
    seed: fabelgeist_determinism::Seed,
    east: i32,
    north: i32,
}

pub(super) struct DetailNoise {
    pub(super) fields: DetailFieldSeeds,
    controls: RefCell<HashMap<LatticeControl, f32>>,
}

pub(super) struct DetailFieldSeeds {
    pub(super) broad: fabelgeist_determinism::Seed,
    pub(super) fine: fabelgeist_determinism::Seed,
    pub(super) clod: fabelgeist_determinism::Seed,
    pub(super) rill_warp: fabelgeist_determinism::Seed,
    pub(super) rill_spacing: fabelgeist_determinism::Seed,
    pub(super) creep_warp: fabelgeist_determinism::Seed,
    pub(super) strata_warp: fabelgeist_determinism::Seed,
    pub(super) fracture_a: fabelgeist_determinism::Seed,
    pub(super) fracture_b: fabelgeist_determinism::Seed,
    pub(super) road_rut: fabelgeist_determinism::Seed,
}

impl DetailNoise {
    pub(super) fn new(seed: fabelgeist_determinism::Seed) -> Self {
        Self {
            fields: DetailFieldSeeds {
                broad: streams::BROAD.seed(seed, &[]),
                fine: streams::FINE.seed(seed, &[]),
                clod: streams::CLOD.seed(seed, &[]),
                rill_warp: streams::RILL_WARP.seed(seed, &[]),
                rill_spacing: streams::RILL_SPACING.seed(seed, &[]),
                creep_warp: streams::CREEP_WARP.seed(seed, &[]),
                strata_warp: streams::STRATA_WARP.seed(seed, &[]),
                fracture_a: streams::FRACTURE_A.seed(seed, &[]),
                fracture_b: streams::FRACTURE_B.seed(seed, &[]),
                road_rut: streams::ROAD_RUT.seed(seed, &[]),
            },
            controls: RefCell::default(),
        }
    }

    pub(super) fn sample(&self, seed: fabelgeist_determinism::Seed, point: Vec2) -> f32 {
        let cell = point.floor();
        let local = point - cell;
        let curve = local * local * (Vec2::splat(3.0) - local * 2.0);
        let hash = |offset: Vec2| {
            let coordinate = cell + offset;
            self.control(LatticeControl {
                seed,
                east: coordinate.x as i32,
                north: coordinate.y as i32,
            })
        };
        let bottom = lerp(hash(Vec2::ZERO), hash(Vec2::X), curve.x);
        let top = lerp(hash(Vec2::Y), hash(Vec2::ONE), curve.x);
        lerp(bottom, top, curve.y)
    }

    fn control(&self, key: LatticeControl) -> f32 {
        let mut controls = self.controls.borrow_mut();
        if let Some(value) = controls.get(&key) {
            return *value;
        }
        if controls.len() == MAXIMUM_RETAINED_LATTICE_CONTROLS {
            controls.clear();
        }
        let value = streams::GROUND_MASK_LATTICE
            .rng(
                key.seed,
                &[i64::from(key.east) as u64, i64::from(key.north) as u64],
            )
            .inclusive_unit_f32();
        controls.insert(key, value);
        value
    }
}

// Native scene east/north metres and lattice span retain authored interpolation.
pub(super) fn value_noise(
    seed: fabelgeist_determinism::Seed,
    x: f32,
    z: f32,
    cell_size: f32,
) -> f32 {
    let gx = x / cell_size;
    let gz = z / cell_size;
    let x0 = gx.floor() as i32;
    let z0 = gz.floor() as i32;
    let tx = smoothstep(gx - x0 as f32);
    let tz = smoothstep(gz - z0 as f32);
    let sample = |ix: i32, iz: i32| {
        streams::VALUE_LATTICE
            .rng(seed, &[ix as u32 as u64, iz as u32 as u64])
            .inclusive_unit_f32()
            * 2.0
            - 1.0
    };
    let north = sample(x0, z0) + (sample(x0 + 1, z0) - sample(x0, z0)) * tx;
    let south = sample(x0, z0 + 1) + (sample(x0 + 1, z0 + 1) - sample(x0, z0 + 1)) * tx;
    north + (south - north) * tz
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eviction_and_query_order_preserve_seeded_surface_values() {
        let noise = DetailNoise::new(42.into());
        let queries = [
            (42, Vec2::new(-43.5, -39.5)),
            (47, Vec2::new(-43.5, -39.5)),
            (101, Vec2::new(120.125, -0.875)),
            (42, Vec2::new(-0.001, 0.001)),
        ]
        .map(|(seed, point)| (fabelgeist_determinism::Seed::from_u64(seed), point));
        let expected = queries.map(|(seed, point)| noise.sample(seed, point).to_bits());
        assert_ne!(expected[0], expected[1]);
        for east in 0..=MAXIMUM_RETAINED_LATTICE_CONTROLS {
            noise.sample(900.into(), Vec2::new(east as f32, 900.0));
            assert!(noise.controls.borrow().len() <= MAXIMUM_RETAINED_LATTICE_CONTROLS);
        }
        for index in (0..queries.len()).rev() {
            let (seed, point) = queries[index];
            assert_eq!(noise.sample(seed, point).to_bits(), expected[index]);
            assert_eq!(
                DetailNoise::new(42.into()).sample(seed, point).to_bits(),
                expected[index]
            );
        }
    }
}

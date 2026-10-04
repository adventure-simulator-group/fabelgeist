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
    seed: u64,
    east: i32,
    north: i32,
}

pub(super) struct DetailNoise {
    pub(super) fields: DetailFieldSeeds,
    controls: RefCell<HashMap<LatticeControl, f32>>,
}

pub(super) struct DetailFieldSeeds {
    pub(super) broad: u64,
    pub(super) fine: u64,
    pub(super) clod: u64,
    pub(super) rill_warp: u64,
    pub(super) rill_spacing: u64,
    pub(super) creep_warp: u64,
    pub(super) strata_warp: u64,
    pub(super) fracture_a: u64,
    pub(super) fracture_b: u64,
    pub(super) road_rut: u64,
}

impl DetailNoise {
    pub(super) fn new(seed: u64) -> Self {
        Self {
            fields: DetailFieldSeeds {
                broad: streams::BROAD.seed(seed, &[]).to_u64(),
                fine: streams::FINE.seed(seed, &[]).to_u64(),
                clod: streams::CLOD.seed(seed, &[]).to_u64(),
                rill_warp: streams::RILL_WARP.seed(seed, &[]).to_u64(),
                rill_spacing: streams::RILL_SPACING.seed(seed, &[]).to_u64(),
                creep_warp: streams::CREEP_WARP.seed(seed, &[]).to_u64(),
                strata_warp: streams::STRATA_WARP.seed(seed, &[]).to_u64(),
                fracture_a: streams::FRACTURE_A.seed(seed, &[]).to_u64(),
                fracture_b: streams::FRACTURE_B.seed(seed, &[]).to_u64(),
                road_rut: streams::ROAD_RUT.seed(seed, &[]).to_u64(),
            },
            controls: RefCell::default(),
        }
    }

    pub(super) fn sample(&self, seed: u64, point: Vec2) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eviction_and_query_order_preserve_seeded_surface_values() {
        let noise = DetailNoise::new(42);
        let queries = [
            (42, Vec2::new(-43.5, -39.5)),
            (47, Vec2::new(-43.5, -39.5)),
            (101, Vec2::new(120.125, -0.875)),
            (42, Vec2::new(-0.001, 0.001)),
        ];
        let expected = queries.map(|(seed, point)| noise.sample(seed, point).to_bits());
        assert_ne!(expected[0], expected[1]);
        for east in 0..=MAXIMUM_RETAINED_LATTICE_CONTROLS {
            noise.sample(900, Vec2::new(east as f32, 900.0));
            assert!(noise.controls.borrow().len() <= MAXIMUM_RETAINED_LATTICE_CONTROLS);
        }
        for index in (0..queries.len()).rev() {
            let (seed, point) = queries[index];
            assert_eq!(noise.sample(seed, point).to_bits(), expected[index]);
            assert_eq!(
                DetailNoise::new(42).sample(seed, point).to_bits(),
                expected[index]
            );
        }
    }
}

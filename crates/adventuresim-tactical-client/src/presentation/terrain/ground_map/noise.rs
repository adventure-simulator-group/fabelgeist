//! Reuse exact deterministic lattice values across dense raster samples.
use bevy::math::{FloatExt, IVec2, Vec2};

fn lattice_value(seed: u64, cell: IVec2) -> f32 {
    adventuresim_tactical_core::terrain_streams::GROUND_MASK_LATTICE
        .rng(
            seed.into(),
            &[i64::from(cell.x) as u64, i64::from(cell.y) as u64],
        )
        .inclusive_unit_f32()
}

pub(super) struct GroundMaskNoise {
    origin: IVec2,
    width: usize,
    values: Vec<f32>,
}

impl GroundMaskNoise {
    pub(super) fn new(seed: u64, minimum: Vec2, maximum: Vec2) -> Self {
        let origin = minimum.floor().as_ivec2();
        // Bilinear interpolation also reads the next lattice corner.
        let end = maximum.floor().as_ivec2() + IVec2::ONE;
        let width = (end.x - origin.x + 1) as usize;
        let values = (origin.y..=end.y)
            .flat_map(|y| (origin.x..=end.x).map(move |x| lattice_value(seed, IVec2::new(x, y))))
            .collect();
        Self {
            origin,
            width,
            values,
        }
    }

    pub(super) fn sample(&self, point: Vec2) -> f32 {
        let cell = point.floor();
        let local = point - cell;
        let curve = local * local * (Vec2::splat(3.0) - local * 2.0);
        let index = cell.as_ivec2() - self.origin;
        let index = index.y as usize * self.width + index.x as usize;
        let bottom = self.values[index].lerp(self.values[index + 1], curve.x);
        let top =
            self.values[index + self.width].lerp(self.values[index + self.width + 1], curve.x);
        bottom.lerp(top, curve.y)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ground_mask_noise;
    use super::*;

    #[test]
    fn dense_lookup_preserves_scalar_noise_at_boundaries_and_negative_coordinates() {
        for seed in [0, 42, u64::MAX] {
            let minimum = Vec2::new(-17.3, -9.1);
            let maximum = Vec2::new(19.0, 31.7);
            let noise = GroundMaskNoise::new(seed, minimum, maximum);
            for z in 0..=80 {
                for x in 0..=80 {
                    let point =
                        minimum + (maximum - minimum) * (Vec2::new(x as f32, z as f32) / 80.0);
                    assert_eq!(noise.sample(point), ground_mask_noise(seed, point));
                }
            }
        }
    }

    #[test]
    #[ignore = "explicit CPU microbenchmark; never run alongside browser measurements"]
    fn benchmark_dense_ground_noise() {
        let start = std::time::Instant::now();
        let noise = GroundMaskNoise::new(42, Vec2::ZERO, Vec2::splat(38.0));
        for z in 0..601 {
            for x in 0..601 {
                let point = Vec2::new(x as f32, z as f32) / 6.0 * 0.38;
                std::hint::black_box(noise.sample(point));
            }
        }
        let table_time = start.elapsed();
        let start = std::time::Instant::now();
        for z in 0..601 {
            for x in 0..601 {
                let point = Vec2::new(x as f32, z as f32) / 6.0 * 0.38;
                std::hint::black_box(ground_mask_noise(42, point));
            }
        }
        eprintln!(
            "dense={table_time:?}; scalar={:?}; lattice_values={}",
            start.elapsed(),
            noise.values.len()
        );
    }
}

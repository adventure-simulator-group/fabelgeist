//! Shared fixtures: deterministic geometry the CPU and GPU tests both use.

use fabelgeist_math::Vec3;

use crate::Aabb;

/// xorshift32, so a failure reproduces from the seed in the test name.
pub struct Random(u32);

impl Random {
    pub fn new(seed: u32) -> Self {
        Self(seed | 1)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + self.unit() * (high - low)
    }

    pub fn point(&mut self, low: f32, high: f32) -> Vec3 {
        Vec3::new(
            self.range(low, high),
            self.range(low, high),
            self.range(low, high),
        )
    }
}

/// Scattered boxes of varying size -- the general case.
pub fn scattered_boxes(count: usize, seed: u32) -> Vec<Aabb> {
    let mut random = Random::new(seed);
    (0..count)
        .map(|_| {
            let center = random.point(-10.0, 10.0);
            let half = Vec3::new(
                random.range(0.05, 1.5),
                random.range(0.05, 1.5),
                random.range(0.05, 1.5),
            );
            Aabb::new(center - half, center + half)
        })
        .collect()
}

/// A triangulated sphere: realistic mesh geometry, and every closest-point
/// answer is checkable against the analytic surface.
pub fn sphere_mesh(rings: usize, segments: usize, radius: f32) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let mut positions = Vec::new();
    for ring in 0..=rings {
        let phi = std::f32::consts::PI * ring as f32 / rings as f32;
        for segment in 0..segments {
            let theta = std::f32::consts::TAU * segment as f32 / segments as f32;
            positions.push(Vec3::new(
                radius * phi.sin() * theta.cos(),
                radius * phi.cos(),
                radius * phi.sin() * theta.sin(),
            ));
        }
    }

    let mut triangles = Vec::new();
    for ring in 0..rings {
        for segment in 0..segments {
            let next_segment = (segment + 1) % segments;
            let a = (ring * segments + segment) as u32;
            let b = (ring * segments + next_segment) as u32;
            let c = ((ring + 1) * segments + segment) as u32;
            let d = ((ring + 1) * segments + next_segment) as u32;
            triangles.push([a, c, b]);
            triangles.push([b, c, d]);
        }
    }

    (positions, triangles)
}

/// Every primitive whose box overlaps `query`, found the slow honest way.
pub fn brute_force_overlaps(bounds: &[Aabb], query: &Aabb) -> Vec<u32> {
    bounds
        .iter()
        .enumerate()
        .filter(|(_, b)| b.overlaps(query))
        .map(|(i, _)| i as u32)
        .collect()
}

//! Periodic finite stamps. Recipes own their scales, masks and layer composition.
mod streams;
use crate::TextureParameters;
use bevy::math::{IVec2, Vec2};
use fabelgeist_determinism::StreamId;
#[cfg(test)]
mod tests;

crate::parameters::parameter_block! {
    pub struct Parameters {
        cells: [i32; 2] = [24, 24];
        radius: [f32; 2] = [0.30, 0.30];
        density: f32 = 0.4;
        cluster_cells: [i32; 2] = [3, 3];
        cluster_strength: f32 = 0.5;
        jitter: f32 = 0.8;
        size_variation: f32 = 0.35;
        angle: f32 = 0.0;
        angle_variation: f32 = std::f32::consts::PI;
        edge_width: f32 = 0.25;
        roundness: f32 = 1.0;
        depth: f32 = 0.04;
    }
}

pub(crate) fn hash(params: &TextureParameters, cell: IVec2, cells: IVec2, field_seed: u64) -> f32 {
    params
        .rng(
            streams::LATTICE,
            &[
                field_seed,
                cell.x.rem_euclid(cells.x) as u64,
                cell.y.rem_euclid(cells.y) as u64,
            ],
        )
        .inclusive_unit_f32()
}

pub(crate) fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn noise(params: &TextureParameters, uv: Vec2, cells: IVec2, field_seed: u64) -> f32 {
    let p = uv.rem_euclid(Vec2::ONE) * cells.as_vec2();
    let cell = p.floor().as_ivec2();
    let f = p - p.floor();
    let a = hash(params, cell, cells, field_seed);
    let b = hash(params, cell + IVec2::X, cells, field_seed);
    let c = hash(params, cell + IVec2::Y, cells, field_seed);
    let d = hash(params, cell + IVec2::ONE, cells, field_seed);
    let x = smooth(f.x);
    let y = smooth(f.y);
    (a + (b - a) * x) * (1.0 - y) + (c + (d - c) * x) * y
}

#[derive(Default)]
pub(crate) struct Sample {
    pub bowl: f32,
    pub facet: f32,
    pub edge: f32,
}

impl Parameters {
    /// A two-cell search contains every allowed rotated stamp, including its feather.
    pub(crate) fn sample(&self, params: &TextureParameters, uv: Vec2, field_seed: u64) -> Sample {
        let cells = IVec2::from_array(self.cells);
        let p = uv.rem_euclid(Vec2::ONE) * cells.as_vec2();
        let base = p.floor().as_ivec2();
        let mut out = Sample::default();
        for y in -2..=2 {
            for x in -2..=2 {
                let cell = base + IVec2::new(x, y);
                let random = |purpose: StreamId| {
                    hash(params, cell, cells, purpose.seed(field_seed, &[]).to_u64())
                };
                let site = cell.as_vec2()
                    + Vec2::splat(0.5)
                    + (Vec2::new(random(streams::SITE_X), random(streams::SITE_Y))
                        - Vec2::splat(0.5))
                        * self.jitter;
                // A feature's activation is fixed at its site, so the feature
                // cannot disappear halfway across as the texel moves.
                let cluster = noise(
                    params,
                    site / cells.as_vec2(),
                    IVec2::from_array(self.cluster_cells),
                    params.field_seed(streams::CLUSTER, &[field_seed]),
                );
                let density = self.density
                    * (1.0 - self.cluster_strength + cluster * 2.0 * self.cluster_strength);
                if random(streams::PRESENCE) >= density {
                    continue;
                }
                let angle = self.angle + (random(streams::ANGLE) - 0.5) * self.angle_variation;
                let (sin, cos) = angle.sin_cos();
                let d = p - site;
                let local = Vec2::new(cos * d.x + sin * d.y, -sin * d.x + cos * d.y)
                    / (Vec2::from_array(self.radius)
                        * (1.0 - self.size_variation * random(streams::SIZE)));
                let radius = local.abs().max_element() * (1.0 - self.roundness)
                    + local.length() * self.roundness;
                let coverage = smooth((1.0 - radius) / self.edge_width);
                let depth = self.depth * (1.0 - self.size_variation * random(streams::DEPTH));
                out.bowl = out
                    .bowl
                    .max(coverage * (1.0 - radius * radius).max(0.0) * depth);
                out.facet = out.facet.max(coverage * (1.0 - radius).max(0.0) * depth);
                out.edge = out.edge.max(coverage * (1.0 - coverage) * 4.0 * depth);
            }
        }
        out
    }
}

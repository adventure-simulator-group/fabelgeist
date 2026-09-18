//! Periodic, locally scattered tool stamps. All distances are in cell coordinates.
mod streams;
use super::{Parameters, smooth};
use crate::TextureParameters;
use fabelgeist_determinism::StreamId;

fn random(params: &TextureParameters, x: i32, y: i32, cells: [i32; 2], field_seed: u64) -> f32 {
    params
        .rng(
            streams::LATTICE,
            &[
                field_seed,
                x.rem_euclid(cells[0]) as u64,
                y.rem_euclid(cells[1]) as u64,
            ],
        )
        .inclusive_unit_f32()
}

pub(super) fn noise(
    params: &TextureParameters,
    u: f32,
    v: f32,
    cells: [i32; 2],
    field_seed: u64,
) -> f32 {
    let x = u.rem_euclid(1.0) * cells[0] as f32;
    let y = v.rem_euclid(1.0) * cells[1] as f32;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let tx = smooth(x.fract());
    let ty = smooth(y.fract());
    let a = random(params, ix, iy, cells, field_seed);
    let b = random(params, ix + 1, iy, cells, field_seed);
    let c = random(params, ix, iy + 1, cells, field_seed);
    let d = random(params, ix + 1, iy + 1, cells, field_seed);
    let top = a + (b - a) * tx;
    top + (c + (d - c) * tx - top) * ty
}

#[derive(Clone, Copy)]
pub(super) struct Sample {
    pub height: f32,
    pub cavity: f32,
    pub crown: f32,
}

fn hammer(params: &TextureParameters, u: f32, v: f32) -> (f32, f32) {
    let p = &params.ironwork;
    let cells = p.hammer_cells;
    let x = u.rem_euclid(1.0) * cells[0] as f32;
    let y = v.rem_euclid(1.0) * cells[1] as f32;
    let mut depression = 0.0_f32;
    let mut rim = 0.0_f32;
    for cy in y.floor() as i32 - 1..=y.floor() as i32 + 1 {
        for cx in x.floor() as i32 - 1..=x.floor() as i32 + 1 {
            let rand = |field_seed| random(params, cx, cy, cells, field_seed);
            let dx = x
                - cx as f32
                - 0.5
                - (rand(params.field_seed(streams::HAMMER_X, &[])) - 0.5) * p.hammer_jitter;
            let dy = y
                - cy as f32
                - 0.5
                - (rand(params.field_seed(streams::HAMMER_Y, &[])) - 0.5) * p.hammer_jitter;
            let (sin, cos) = ((rand(params.field_seed(streams::HAMMER_ANGLE, &[])) - 0.5)
                * p.hammer_angle)
                .sin_cos();
            let radius = p.hammer_radius
                * (1.0
                    - p.hammer_size_variation
                        * rand(params.field_seed(streams::HAMMER_RADIUS, &[])));
            let along = (dx * cos + dy * sin) / radius;
            let across = (-dx * sin + dy * cos) / radius;
            // Rounded rectangular dies produce broad planes, rather than circular dents.
            let radius = along.abs().max(across.abs());
            let round = (along * along + across * across).sqrt();
            let distance = radius + (round - radius) * p.hammer_roundness;
            let face = smooth(((1.0 - distance) / p.hammer_roundness).clamp(0.0, 1.0));
            let tilt = (along * (rand(params.field_seed(streams::HAMMER_ALONG_TILT, &[])) - 0.5)
                + across * (rand(params.field_seed(streams::HAMMER_ACROSS_TILT, &[])) - 0.5))
                * p.hammer_tilt;
            let depth = p.hammer_depth
                * (1.0
                    - p.hammer_depth_variation
                        * rand(params.field_seed(streams::HAMMER_DEPTH, &[])));
            let shoulder = face * (1.0 - face) * 4.0;
            let impression = (-depth + tilt) * face + shoulder * p.hammer_rim;
            // A deeper impression obliterates the older shoulder beneath it.
            if impression < depression {
                depression = impression;
                rim = shoulder;
            }
        }
    }
    (depression, rim)
}

struct CavityStamp {
    cells: [i32; 2],
    density: f32,
    radius: f32,
    edge: f32,
    angular: bool,
    field_seed: u64,
}
impl CavityStamp {
    fn sample(&self, params: &TextureParameters, u: f32, v: f32) -> f32 {
        let x = u.rem_euclid(1.0) * self.cells[0] as f32;
        let y = v.rem_euclid(1.0) * self.cells[1] as f32;
        let mut cavity = 0.0_f32;
        let breakup = if self.angular {
            (noise(params, u, v, params.ironwork.grain_cells, self.field_seed) - 0.5)
                * params.ironwork.scale_edge_breakup
        } else {
            0.0
        };
        for cy in y.floor() as i32 - 1..=y.floor() as i32 + 1 {
            for cx in x.floor() as i32 - 1..=x.floor() as i32 + 1 {
                let rand = |purpose: StreamId| {
                    random(
                        params,
                        cx,
                        cy,
                        self.cells,
                        purpose.seed(self.field_seed, &[]).to_u64(),
                    )
                };
                let cluster = noise(
                    params,
                    (cx as f32 + 0.5) / self.cells[0] as f32,
                    (cy as f32 + 0.5) / self.cells[1] as f32,
                    params.ironwork.oxide_cells,
                    params.field_seed(streams::OXIDE, &[]),
                );
                if rand(streams::CAVITY_PRESENCE) >= self.density * cluster {
                    continue;
                }
                let dx = x - cx as f32 - rand(streams::CAVITY_X);
                let dy = y - cy as f32 - rand(streams::CAVITY_Y);
                let (sin, cos) =
                    (rand(streams::CAVITY_ANGLE) * params.ironwork.scale_angle).sin_cos();
                let (dx, dy) = (dx * cos + dy * sin, -dx * sin + dy * cos);
                let radius = self.radius * (0.5 + rand(streams::CAVITY_RADIUS) * 0.5);
                let distance = if self.angular {
                    dx.abs()
                        .max(dy.abs())
                        .max((dx + dy).abs() * std::f32::consts::FRAC_1_SQRT_2)
                } else {
                    dx.hypot(dy)
                } / radius
                    + breakup;
                let edge = self.edge.max(
                    self.cells[0].max(self.cells[1]) as f32
                        / (params.size(super::IRONWORK_TEXTURE_SIZE) as f32 * radius),
                );
                cavity = cavity.max(smooth(((1.0 - distance) / edge).clamp(0.0, 1.0)));
            }
        }
        cavity
    }
}

pub(super) fn sample(params: &TextureParameters, u: f32, v: f32) -> Sample {
    let p: &Parameters = &params.ironwork;
    let body = noise(
        params,
        u,
        v,
        p.body_cells,
        params.field_seed(streams::BODY, &[]),
    );
    let grain = noise(
        params,
        u,
        v,
        p.grain_cells,
        params.field_seed(streams::GRAIN, &[]),
    ) - 0.5;
    let (hammer, crown) = hammer(params, u, v);
    let scale = CavityStamp {
        cells: p.scale_cells,
        density: p.scale_density,
        radius: p.scale_radius,
        edge: p.scale_edge_width,
        angular: true,
        field_seed: params.field_seed(streams::SCALE, &[]),
    }
    .sample(params, u, v);
    let pits = CavityStamp {
        cells: p.pit_cells,
        density: p.pit_density,
        radius: p.pit_radius,
        edge: 1.0,
        angular: false,
        field_seed: params.field_seed(streams::PIT, &[]),
    }
    .sample(params, u, v);
    Sample {
        height: (0.5 + (body - 0.5) * p.body_relief + hammer + grain * p.grain_relief
            - scale * p.scale_depth
            - pits * p.pit_depth)
            .clamp(0.0, 1.0),
        cavity: scale.max(pits),
        crown,
    }
}

pub(super) fn oxide_coverage(params: &TextureParameters, u: f32, v: f32) -> f32 {
    let p = &params.ironwork;
    if p.oxide_fraction >= 1.0 {
        return 1.0;
    }
    if p.oxide_fraction <= 0.0 {
        return 0.0;
    }
    let step = 0.25 / params.size(super::IRONWORK_TEXTURE_SIZE) as f32;
    let mut covered = 0.0;
    for (dx, dy) in [(-step, -step), (step, -step), (-step, step), (step, step)] {
        if noise(
            params,
            u + dx,
            v + dy,
            p.oxide_cells,
            params.field_seed(streams::OXIDE, &[]),
        ) > 1.0 - p.oxide_fraction
        {
            covered += 0.25;
        }
    }
    covered
}

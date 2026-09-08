//! Periodic, locally scattered tool stamps. All distances are in cell coordinates.
use super::{Parameters, smooth};
use crate::TextureParameters;
use fabelgeist_determinism::inclusive_unit_f32;

fn random(params: &TextureParameters, x: i32, y: i32, cells: [i32; 2], salt: u64) -> f32 {
    inclusive_unit_f32(crate::parameters::seeded_hash(
        params,
        salt ^ ((x.rem_euclid(cells[0]) as u64) << 32) ^ y.rem_euclid(cells[1]) as u64,
    ))
}

pub(super) fn noise(params: &TextureParameters, u: f32, v: f32, cells: [i32; 2], salt: u64) -> f32 {
    let x = u.rem_euclid(1.0) * cells[0] as f32;
    let y = v.rem_euclid(1.0) * cells[1] as f32;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let tx = smooth(x.fract());
    let ty = smooth(y.fract());
    let a = random(params, ix, iy, cells, salt);
    let b = random(params, ix + 1, iy, cells, salt);
    let c = random(params, ix, iy + 1, cells, salt);
    let d = random(params, ix + 1, iy + 1, cells, salt);
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
            let rand = |salt| random(params, cx, cy, cells, salt);
            let dx = x - cx as f32 - 0.5 - (rand(0x31c7) - 0.5) * p.hammer_jitter;
            let dy = y - cy as f32 - 0.5 - (rand(0xa579) - 0.5) * p.hammer_jitter;
            let (sin, cos) = ((rand(0x6d21) - 0.5) * p.hammer_angle).sin_cos();
            let radius = p.hammer_radius * (1.0 - p.hammer_size_variation * rand(0x9523));
            let along = (dx * cos + dy * sin) / radius;
            let across = (-dx * sin + dy * cos) / radius;
            // Rounded rectangular dies produce broad planes, rather than circular dents.
            let radius = along.abs().max(across.abs());
            let round = (along * along + across * across).sqrt();
            let distance = radius + (round - radius) * p.hammer_roundness;
            let face = smooth(((1.0 - distance) / p.hammer_roundness).clamp(0.0, 1.0));
            let tilt =
                (along * (rand(0x42e1) - 0.5) + across * (rand(0xb731) - 0.5)) * p.hammer_tilt;
            let depth = p.hammer_depth * (1.0 - p.hammer_depth_variation * rand(0x4519));
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
    salt: u64,
}
impl CavityStamp {
    fn sample(&self, params: &TextureParameters, u: f32, v: f32) -> f32 {
        let x = u.rem_euclid(1.0) * self.cells[0] as f32;
        let y = v.rem_euclid(1.0) * self.cells[1] as f32;
        let mut cavity = 0.0_f32;
        let breakup = if self.angular {
            (noise(params, u, v, params.ironwork.grain_cells, self.salt) - 0.5)
                * params.ironwork.scale_edge_breakup
        } else {
            0.0
        };
        for cy in y.floor() as i32 - 1..=y.floor() as i32 + 1 {
            for cx in x.floor() as i32 - 1..=x.floor() as i32 + 1 {
                let rand = |salt| random(params, cx, cy, self.cells, self.salt ^ salt);
                let cluster = noise(
                    params,
                    (cx as f32 + 0.5) / self.cells[0] as f32,
                    (cy as f32 + 0.5) / self.cells[1] as f32,
                    params.ironwork.oxide_cells,
                    0x39de_b647,
                );
                if rand(0x7301) >= self.density * cluster {
                    continue;
                }
                let dx = x - cx as f32 - rand(0x48f1);
                let dy = y - cy as f32 - rand(0x6329);
                let (sin, cos) = (rand(0x2917) * params.ironwork.scale_angle).sin_cos();
                let (dx, dy) = (dx * cos + dy * sin, -dx * sin + dy * cos);
                let radius = self.radius * (0.5 + rand(0x3291) * 0.5);
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
    let body = noise(params, u, v, p.body_cells, 0x10e4_91a7);
    let grain = noise(params, u, v, p.grain_cells, 0x8365_9ea7) - 0.5;
    let (hammer, crown) = hammer(params, u, v);
    let scale = CavityStamp {
        cells: p.scale_cells,
        density: p.scale_density,
        radius: p.scale_radius,
        edge: p.scale_edge_width,
        angular: true,
        salt: 0x74a1_29dd,
    }
    .sample(params, u, v);
    let pits = CavityStamp {
        cells: p.pit_cells,
        density: p.pit_density,
        radius: p.pit_radius,
        edge: 1.0,
        angular: false,
        salt: 0x4f83_d2a1,
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
        if noise(params, u + dx, v + dy, p.oxide_cells, 0x39de_b647) > 1.0 - p.oxide_fraction {
            covered += 0.25;
        }
    }
    covered
}

//! Intact painted construction, with footprint-filtered features in millimetres.
use crate::document::*;
use std::f32::consts::TAU;
const CANVAS_THREAD_MM: f32 = 1.1;
const HIDE_GRAIN_MM: f32 = 3.5;
const GROUND_SMOOTHING_MM: f32 = 0.35;
const PIGMENT_VARIATION: f32 = 0.065;
const OIL_CREASE_PITCH_MM: f32 = 0.8;
pub(super) struct Sample {
    pub support: f32,
    pub brush: f32,
    pub crease: f32,
    pub pigment: f32,
}
pub(super) fn sample(s: &PaintedSurface, p: [f32; 2], footprint: f32) -> Sample {
    let [x, y] = p;
    let seed = s.seed.0;
    let broad = noise(x / 37.0, y / 37.0, seed);
    let ground = (-s.ground.0 / GROUND_SMOOTHING_MM).exp();
    let covering = match s.covering {
        Covering::None => {
            filtered_wave(x + y * 0.018, 9.0, footprint) * 0.5 + noise(x / 4.0, y / 95.0, seed)
        }
        Covering::Canvas => {
            filtered_wave(x, CANVAS_THREAD_MM, footprint)
                * filtered_wave(y, CANVAS_THREAD_MM, footprint)
        }
        Covering::Hide => {
            noise(x / HIDE_GRAIN_MM, y / HIDE_GRAIN_MM, seed) * visibility(HIDE_GRAIN_MM, footprint)
        }
    };
    let (sin, cos) = s.brush_angle.0.to_radians().sin_cos();
    let across = x * cos + y * sin;
    let along = -x * sin + y * cos;
    let bristle_pitch = s.brush_width.0 / 9.0;
    let brush = filtered_wave(
        across + noise(across / 19.0, along / 65.0, seed),
        bristle_pitch,
        footprint,
    ) * (0.65 + 0.35 * noise(across / s.brush_width.0, along / 28.0, seed));
    Sample {
        support: covering * s.substrate_relief.0 * ground + broad * s.substrate_relief.0 * 0.1,
        brush,
        crease: noise(x / OIL_CREASE_PITCH_MM, y / OIL_CREASE_PITCH_MM, seed)
            * visibility(OIL_CREASE_PITCH_MM, footprint),
        pigment: 1.0 + PIGMENT_VARIATION * (broad + brush * 0.15),
    }
}
fn visibility(pitch: f32, footprint: f32) -> f32 {
    ((pitch / footprint - 2.0) / 2.0).clamp(0.0, 1.0)
}
fn filtered_wave(x: f32, pitch: f32, footprint: f32) -> f32 {
    (x / pitch * TAU).cos() * visibility(pitch, footprint)
}
fn hash(x: i32, y: i32, seed: u32) -> f32 {
    fabelgeist_determinism::StreamId::new("heraldry.workmanship.lattice")
        .rng(u64::from(seed), &[x as u32 as u64, y as u32 as u64])
        .inclusive_unit_f32()
        * 2.0
        - 1.0
}
fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let smooth = |v: f32| v * v * (3.0 - 2.0 * v);
    let u = smooth(x - x.floor());
    let v = smooth(y - y.floor());
    let lerp = |a, b, t| a + (b - a) * t;
    lerp(
        lerp(hash(ix, iy, seed), hash(ix + 1, iy, seed), u),
        lerp(hash(ix, iy + 1, seed), hash(ix + 1, iy + 1, seed), u),
        v,
    )
}

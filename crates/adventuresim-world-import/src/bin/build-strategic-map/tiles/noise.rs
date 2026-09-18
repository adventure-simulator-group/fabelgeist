//! Correlated map lattice fields and organic outline perturbations.
use super::*;

pub(super) fn fractal_noise(mut x: f64, mut y: f64, seed: u64) -> f64 {
    let mut amplitude = 0.58;
    let mut total = 0.0;
    let mut weight = 0.0;
    for octave in 0_u64..4 {
        total +=
            value_noise(x, y, streams::NOISE_OCTAVE.seed(seed, &[octave]).to_u64()) * amplitude;
        weight += amplitude;
        x = x * 2.03 + 17.7;
        y = y * 2.03 - 11.3;
        amplitude *= 0.5;
    }
    total / weight
}

pub(super) fn value_noise(x: f64, y: f64, seed: u64) -> f64 {
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let tx = smoothstep(x - x.floor());
    let ty = smoothstep(y - y.floor());
    let top = lerp(
        lattice_noise(x0, y0, seed),
        lattice_noise(x0 + 1, y0, seed),
        tx,
    );
    let bottom = lerp(
        lattice_noise(x0, y0 + 1, seed),
        lattice_noise(x0 + 1, y0 + 1, seed),
        tx,
    );
    lerp(top, bottom, ty)
}

pub(super) fn lattice_noise(x: i64, y: i64, seed: u64) -> f64 {
    streams::LATTICE.rng(seed, &[x as u64, y as u64]).unit_f64() * 2.0 - 1.0
}

pub(super) fn smoothstep(value: f64) -> f64 {
    value * value * (3.0 - 2.0 * value)
}

pub(super) fn lerp(left: f64, right: f64, amount: f64) -> f64 {
    left + (right - left) * amount
}

pub(super) fn organic_vertex_noise(point: (f64, f64)) -> f64 {
    let x = (point.0 * 16.0).round() as i64 as u64;
    let y = (point.1 * 16.0).round() as i64 as u64;
    streams::ORGANIC_VERTEX.rng(0, &[x, y]).inclusive_unit_f64()
}

//! Gradient noise on the host: Blender's `Noise Texture` for the hair noise
//! operation, and `White Noise Texture` for the clump twist.

use fabelgeist_math::Vec3;

use crate::rng::{hash, hash3};

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// One of the twelve edge gradients of Perlin's improved noise, for a
/// lattice corner.
fn gradient(ix: i32, iy: i32, iz: i32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash3(ix as u32, iy as u32, iz as u32) % 12;
    let (u, v) = match h {
        0..=3 => (x, y),
        4..=7 => (x, z),
        _ => (y, z),
    };
    let u = if h & 1 == 0 { u } else { -u };
    let v = if h & 2 == 0 { v } else { -v };
    u + v
}

/// Improved Perlin noise, in about `[-1, 1]`.
pub fn perlin(p: Vec3) -> f32 {
    let (fx, fy, fz) = (p.x.floor(), p.y.floor(), p.z.floor());
    let (ix, iy, iz) = (fx as i32, fy as i32, fz as i32);
    let (x, y, z) = (p.x - fx, p.y - fy, p.z - fz);
    let (u, v, w) = (fade(x), fade(y), fade(z));
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let g = |dx: i32, dy: i32, dz: i32| {
        gradient(
            ix + dx,
            iy + dy,
            iz + dz,
            x - dx as f32,
            y - dy as f32,
            z - dz as f32,
        )
    };
    let value = lerp(
        lerp(
            lerp(g(0, 0, 0), g(1, 0, 0), u),
            lerp(g(0, 1, 0), g(1, 1, 0), u),
            v,
        ),
        lerp(
            lerp(g(0, 0, 1), g(1, 0, 1), u),
            lerp(g(0, 1, 1), g(1, 1, 1), u),
            v,
        ),
        w,
    );
    // Blender scales its Perlin noise so that it spans about [-1, 1].
    (value * 0.982).clamp(-1.0, 1.0)
}

/// Fractal noise: `octaves` layers, each at twice the frequency and
/// `roughness` times the amplitude of the last, normalised to `[0, 1]`.
pub fn fractal(p: Vec3, octaves: u32, roughness: f32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut total = 0.0;
    let mut q = p;
    for _ in 0..octaves.max(1) {
        sum += perlin(q) * amplitude;
        total += amplitude;
        amplitude *= roughness;
        q *= 2.0;
    }
    0.5 + 0.5 * sum / total
}

/// Blender's `Noise Texture` colour: three decorrelated fractal noises, each
/// in `[0, 1]`, at `scale` cells a unit.
pub fn noise_color(p: Vec3, scale: f32) -> Vec3 {
    let q = p * scale;
    Vec3::new(
        fractal(q, 1, 0.5),
        fractal(q + Vec3::new(31.416, -47.853, 12.679), 1, 0.5),
        fractal(q + Vec3::new(-83.1, 71.3, -21.7), 1, 0.5),
    )
}

/// Blender's `White Noise Texture`: a value in `[0, 1)` per position,
/// constant nowhere.
pub fn white(p: Vec3) -> f32 {
    let h = hash3(
        p.x.to_bits(),
        p.y.to_bits(),
        hash(p.z.to_bits(), 0x5bd1_e995),
    );
    (h >> 8) as f32 / (1u32 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_is_zero_on_the_lattice_and_bounded() {
        assert_eq!(perlin(Vec3::new(3.0, -2.0, 5.0)), 0.0);
        let mut lo: f32 = 0.0;
        let mut hi: f32 = 0.0;
        for i in 0..5000 {
            let t = i as f32 * 0.137;
            let v = perlin(Vec3::new(t, t * 0.31 + 1.7, t * 0.73 - 4.1));
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!(lo > -1.01 && hi < 1.01 && hi - lo > 0.8, "{lo} {hi}");
    }

    #[test]
    fn perlin_is_continuous() {
        let a = perlin(Vec3::new(0.4999, 0.3, 0.2));
        let b = perlin(Vec3::new(0.5001, 0.3, 0.2));
        assert!((a - b).abs() < 1e-3);
    }
}

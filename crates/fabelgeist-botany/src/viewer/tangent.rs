//! Shared unit-vector packing at the branch GPU record boundary.

use fabelgeist_math::Vec3;

/// Octahedral encoding of a unit vector into two snorm16s, as WGSL's
/// `unpack2x16snorm` reads them.
pub fn pack(v: Vec3) -> u32 {
    let n = v / (v.x.abs() + v.y.abs() + v.z.abs()).max(1e-12);
    let sign = |x: f32| if x >= 0.0 { 1.0 } else { -1.0 };
    let (x, y) = if n.z >= 0.0 {
        (n.x, n.y)
    } else {
        ((1.0 - n.y.abs()) * sign(n.x), (1.0 - n.x.abs()) * sign(n.y))
    };
    let snorm = |x: f32| (x.clamp(-1.0, 1.0) * 32767.0).round() as i16 as u16 as u32;
    snorm(x) | (snorm(y) << 16)
}

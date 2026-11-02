//! The polynomial smooth minimum, and what it says about two surfaces that
//! meet: how to grow a fillet between them and how to mix their materials.
//! [`crate::wgsl`] has each function under the same name, `surface_` first.

/// The smooth minimum of distances `a` and `b` over `k`: their union, its
/// crease rounded into a fillet of size about `k` (Quílez's polynomial
/// `smin`). The plain minimum where the two differ by `k` or more.
pub fn smin(a: f32, b: f32, k: f32) -> f32 {
    if a == f32::INFINITY {
        return b;
    }
    if b == f32::INFINITY {
        return a;
    }
    let k = k.max(1e-9);
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// How much of a surface a point is made of, against another it blends
/// with: `own` and `other` are its distances to the two, `k` the blend
/// radius -- the smooth minimum's mix factor. One away from the other
/// surface, a half where the two meet, zero deep in it; the other surface
/// has the rest. Two surfaces asking of the same point get halves that sum
/// to one, which is what makes the seam between them invisible.
pub fn mix_factor(own: f32, other: f32, k: f32) -> f32 {
    (0.5 + 0.5 * (other - own) / k.max(1e-9)).clamp(0.0, 1.0)
}

/// How far to move a point of a surface out along its normal to put it on
/// its smooth union with another surface `other` away: `-smin(0, other, k)`,
/// the fillet. A quarter of `k` where the two meet, nothing `k` away. A point
/// deeper in the other surface than `k` is hidden, and stays where it is.
pub fn lift(other: f32, k: f32) -> f32 {
    if other <= -k || other >= k {
        return 0.0;
    }
    -smin(0.0, other, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_smooth_union_is_a_fillet_and_meets_itself_half_way() {
        let k = 0.2;
        // Where the surfaces meet: out by a quarter of the radius, half of
        // each, and the same whichever side asks.
        assert!((lift(0.0, k) - k / 4.0).abs() < 1e-6);
        assert!((mix_factor(0.0, 0.0, k) - 0.5).abs() < 1e-6);
        assert!((mix_factor(0.1, 0.03, k) + mix_factor(0.03, 0.1, k) - 1.0).abs() < 1e-6);
        // A blend radius away, untouched.
        assert_eq!(lift(k, k), 0.0);
        assert_eq!(mix_factor(0.0, k, k), 1.0);
        assert_eq!(smin(0.3, 0.9, k), 0.3);
        // Continuous, and never pushing a point further than the other
        // surface's own depth.
        let mut last = lift(-k + 1e-4, k);
        for i in 1..200 {
            let d = -k + 2.0 * k * i as f32 / 200.0;
            let now = lift(d, k);
            assert!((now - last).abs() < 0.01 * k, "a step at {d}");
            assert!(now <= (k - d).max(0.0) + 1e-6);
            last = now;
        }
        assert_eq!(smin(f32::INFINITY, 2.0, k), 2.0);
    }
}

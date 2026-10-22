//! The volume two spheres share.

use std::f32::consts::PI;

/// Volume of a sphere of radius `r`.
#[inline]
pub fn sphere_volume(r: f32) -> f32 {
    4.0 / 3.0 * PI * r * r * r
}

/// The volume of the lens two spheres of radii `r1` and `r2` share when their
/// centres are `d` apart.
///
/// Zero when they are apart, the smaller sphere's whole volume when one holds
/// the other, and the closed-form lens in between:
///
/// ```text
/// V = pi (r1 + r2 - d)^2 (d^2 + 2d(r1 + r2) - 3(r1 - r2)^2) / (12 d)
/// ```
#[inline]
pub fn sphere_overlap_volume(r1: f32, r2: f32, d: f32) -> f32 {
    if r1 <= 0.0 || r2 <= 0.0 || d >= r1 + r2 {
        return 0.0;
    }
    if d <= (r1 - r2).abs() {
        return sphere_volume(r1.min(r2));
    }
    let sum = r1 + r2;
    let diff = r1 - r2;
    let gap = sum - d;
    PI * gap * gap * (d * d + 2.0 * d * sum - 3.0 * diff * diff) / (12.0 * d)
}

/// [`sphere_volume`] and [`sphere_overlap_volume`] in WGSL, for kernels to
/// paste in.
pub const SPHERE_OVERLAP_WGSL: &str = r#"
fn sphere_volume(r: f32) -> f32 {
    return 4.18879020479 * r * r * r;
}

fn sphere_overlap_volume(r1: f32, r2: f32, d: f32) -> f32 {
    if (r1 <= 0.0 || r2 <= 0.0 || d >= r1 + r2) {
        return 0.0;
    }
    if (d <= abs(r1 - r2)) {
        return sphere_volume(min(r1, r2));
    }
    let sum = r1 + r2;
    let diff = r1 - r2;
    let gap = sum - d;
    return 3.14159265359 * gap * gap * (d * d + 2.0 * d * sum - 3.0 * diff * diff) / (12.0 * d);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apart_touching_and_nested() {
        assert_eq!(sphere_overlap_volume(1.0, 1.0, 2.5), 0.0);
        assert_eq!(sphere_overlap_volume(1.0, 1.0, 2.0), 0.0);
        let nested = sphere_overlap_volume(2.0, 0.5, 0.3);
        assert!((nested - sphere_volume(0.5)).abs() < 1e-6);
    }

    #[test]
    fn continuous_at_the_nesting_boundary() {
        // At d = r1 - r2 the lens must equal the smaller sphere.
        let (r1, r2) = (1.5, 0.5);
        let lens = sphere_overlap_volume(r1, r2, 1.0 + 1e-4);
        assert!((lens - sphere_volume(r2)).abs() < 1e-3, "{lens}");
    }

    #[test]
    fn equal_spheres_match_two_caps() {
        // Two unit spheres a unit apart: two caps of height 1/2.
        let h: f32 = 0.5;
        let cap = PI * h * h * (3.0 - h) / 3.0;
        let lens = sphere_overlap_volume(1.0, 1.0, 1.0);
        assert!((lens - 2.0 * cap).abs() < 1e-5, "{lens} vs {}", 2.0 * cap);
    }
}

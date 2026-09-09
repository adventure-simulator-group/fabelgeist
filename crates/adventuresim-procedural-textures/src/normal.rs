//! OpenGL tangent-space normals from physical height slopes in image coordinates.

use bevy::math::Vec3;

/// Image rows increase downwards; the OpenGL tangent-space Y axis points up.
/// A height increase down the image therefore tilts the normal towards +Y.
pub(crate) fn from_image_gradient(right_slope: f32, down_slope: f32) -> Vec3 {
    Vec3::new(-right_slope, down_slope, 1.0).normalize()
}

/// Pixel-footprint filtering before differentiation keeps oblique thin ridges
/// from turning into alternating steep and flat normals. The separable binomial
/// kernel has one-texel standard deviation and preserves constant/ramp heights.
/// Apply only to the derivative input; physical height and pigment stay intact.
pub(crate) fn filter_heights_periodic(values: &[f32], size: u32) -> Vec<f32> {
    let side = size as i32;
    let mut current = values.to_vec();
    for vertical in [false, true] {
        let mut next = vec![0.0; values.len()];
        for y in 0..side {
            for x in 0..side {
                for (offset, weight) in [(-2, 1.0), (-1, 4.0), (0, 6.0), (1, 4.0), (2, 1.0)] {
                    let xx = (x + if vertical { 0 } else { offset }).rem_euclid(side);
                    let yy = (y + if vertical { offset } else { 0 }).rem_euclid(side);
                    next[(y * side + x) as usize] +=
                        current[(yy * side + xx) as usize] * weight / 16.0;
                }
            }
        }
        current = next;
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filtering_preserves_resolved_slopes_and_removes_pixel_alternation() {
        let side = 32;
        let ramp: Vec<f32> = (0..side)
            .flat_map(|y| (0..side).map(move |x| 0.2 * x as f32 - 0.3 * y as f32))
            .collect();
        let filtered = filter_heights_periodic(&ramp, side);
        for y in 3..side - 3 {
            for x in 3..side - 3 {
                assert!(
                    (filtered[(y * side + x) as usize] - ramp[(y * side + x) as usize]).abs()
                        < 0.00001
                );
            }
        }
        let alternating: Vec<f32> = (0..side)
            .flat_map(|y| (0..side).map(move |x| ((x + y) % 2) as f32))
            .collect();
        assert!(
            filter_heights_periodic(&alternating, side)
                .iter()
                .all(|v| (*v - 0.5).abs() < 0.00001)
        );
    }
}

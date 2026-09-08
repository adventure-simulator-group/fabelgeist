//! Triangle-interior refinement requests for a constrained material domain.

const MAX_OFFCENTER_INSET_M: f64 = 0.0115;

/// Prefer an equilateral off-center from the boundary-edge midpoint. If that
/// would pass the opposite vertex, use the triangle centroid instead. A CDT
/// material triangle is convex even when the complete garment trim is not.
pub(crate) fn boundary_triangle_site(triangle: [[f32; 2]; 3]) -> [f32; 2] {
    let [a, b, c] = triangle.map(|p| p.map(f64::from));
    let midpoint: [f64; 2] = std::array::from_fn(|axis| (a[axis] + b[axis]) * 0.5);
    let length = (c[0] - midpoint[0]).hypot(c[1] - midpoint[1]);
    let edge = (b[0] - a[0]).hypot(b[1] - a[1]);
    let inset = (edge * 3.0_f64.sqrt() * 0.5).min(MAX_OFFCENTER_INSET_M);
    let fraction = if inset < length {
        inset / length
    } else {
        1.0 / 3.0
    };
    std::array::from_fn(|axis| (midpoint[axis] * (1.0 - fraction) + c[axis] * fraction) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(p: [f32; 2], triangle: [[f32; 2]; 3]) -> bool {
        let cross = |a: [f32; 2], b: [f32; 2], c: [f32; 2]| {
            (f64::from(b[0]) - f64::from(a[0])) * (f64::from(c[1]) - f64::from(a[1]))
                - (f64::from(b[1]) - f64::from(a[1])) * (f64::from(c[0]) - f64::from(a[0]))
        };
        let signs = (0..3)
            .map(|i| cross(triangle[i], triangle[(i + 1) % 3], p))
            .collect::<Vec<_>>();
        signs.iter().all(|s| *s > 0.0) || signs.iter().all(|s| *s < 0.0)
    }

    #[test]
    fn actual_shoulder_offcenter_cannot_extrapolate_outside_its_triangle() {
        // Diagnostic 33 left outer-corner face [408,180,179], reordered so
        // the first two vertices are the true boundary edge.
        let original = [
            [-0.109639056, 1.4786707],
            [-0.11230104, 1.4719832],
            [-0.10988934, 1.4772694],
        ];
        let rejected = [-0.10793936, 1.4807742];
        assert!(!inside(rejected, original));
        for (scale, shear, mirror) in [
            (1.0, 0.0, 1.0),
            (1.0, 0.0, -1.0),
            (2.0, 0.4, 1.0),
            (0.5, -0.2, -1.0),
        ] {
            let triangle =
                original.map(|p| [mirror * scale * p[0] + shear * p[1], scale * p[1] - 0.5]);
            let candidate = boundary_triangle_site(triangle);
            assert!(
                inside(candidate, triangle),
                "{candidate:?} not inside {triangle:?}"
            );
            assert!(triangle.iter().all(|p| *p != candidate));
        }
    }

    #[test]
    fn feasible_offcenter_is_retained_without_moving_boundary() {
        let triangle = [[-0.004, 0.0], [0.004, 0.0], [0.0, 0.02]];
        let candidate = boundary_triangle_site(triangle);
        assert_eq!(candidate[0], 0.0);
        assert!((candidate[1] - 0.004 * 3.0_f32.sqrt()).abs() < 1e-9);
        assert!(inside(candidate, triangle));
    }
}

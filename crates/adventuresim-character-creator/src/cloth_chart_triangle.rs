//! Positive triangle membership at native two-dimensional chart boundaries.
use crate::surface_cut::CORRESPONDENCE_PRECISION;
const SOURCE_WEIGHT_TOLERANCE: f64 = 2. / CORRESPONDENCE_PRECISION as f64;
/// All corners and the query use the same caller-owned chart units/frame.
/// Reject exterior/degenerate triangles; never substitute a nearest triangle.
pub(crate) fn weights([a, b, c]: [[f64; 2]; 3], p: [f64; 2]) -> Option<[f32; 3]> {
    let u = [b[0] - a[0], b[1] - a[1]];
    let v = [c[0] - a[0], c[1] - a[1]];
    let q = [p[0] - a[0], p[1] - a[1]];
    let determinant = u[0] * v[1] - u[1] * v[0];
    if !determinant.is_finite() || determinant == 0. {
        return None;
    }
    let b = (q[0] * v[1] - q[1] * v[0]) / determinant;
    let c = (u[0] * q[1] - u[1] * q[0]) / determinant;
    let weights = [1. - b - c, b, c];
    if weights
        .iter()
        .any(|w| !w.is_finite() || *w < -SOURCE_WEIGHT_TOLERANCE)
    {
        return None;
    }
    let weights = weights.map(|w| w.max(0.));
    let sum: f64 = weights.iter().sum();
    Some(weights.map(|w| (w / sum) as f32))
}

//! Restricted alternative to the free angular Bernstein upper depth field.
//! Every horizontal section is an exact ellipse, not a sampled approximation:
//! z = c_affine(y) + v² sum B³_j(v) C_j
//!   + cos(theta) [b_affine(y) + v² sum B³_j(v) D_j].
//! Positive b certifies convex transverse sections. A separate Bernstein bound
//! makes the center meridian non-bowl through the actual neck opening. Neither
//! assertion is a certificate of full principal convexity or unsampled clearance.

use crate::breastplate_qp::LinearConstraint;

pub(super) const COUNT: usize = 8;
const ELEVATION: [f64; 4] = [0.1, 0.3, 0.6, 1.0];

pub(super) fn fill_rows(
    rows: &mut [Vec<f64>; 6],
    height: &[Vec<f64>; 3],
    v: f64,
    span: f64,
    theta: f64,
    angle: f64,
) {
    for j in 0..4 {
        let h = v * v * height[0][j];
        let dh = (2.0 * v * height[0][j] + v * v * height[1][j]) / span;
        let ddh =
            (2.0 * height[0][j] + 4.0 * v * height[1][j] + v * v * height[2][j]) / span.powi(2);
        rows[0][1 + j] = h;
        rows[2][1 + j] = dh;
        rows[4][1 + j] = ddh;
        rows[0][5 + j] = theta.cos() * h;
        rows[1][5 + j] = -theta.sin() * angle * h;
        rows[2][5 + j] = theta.cos() * dh;
        rows[3][5 + j] = -theta.cos() * angle * angle * h;
        rows[4][5 + j] = theta.cos() * ddh;
        rows[5][5 + j] = -theta.sin() * angle * dh;
    }
}

pub(super) fn shape_bounds(
    seam_depth: f64,
    seam_depth_derivative: f64,
    span: f64,
    neck_v: f64,
) -> Result<Vec<LinearConstraint>, String> {
    if !(0.0..=1.0).contains(&neck_v) || !neck_v.is_finite() {
        return Err("Elliptical upper neck is outside the fitted height domain".into());
    }
    // v² B³_j = [1/10,3/10,3/5,1]_j B⁵_(j+2). The first two
    // Bernstein coefficients are fixed by lower C1 continuity, not solve DOFs.
    if seam_depth < 1e-6 || seam_depth + seam_depth_derivative * span / 5.0 < 1e-6 {
        return Err("Elliptical upper depth has nonpositive fixed seam controls".into());
    }
    let mut bounds = Vec::new();
    let mut center = [[0.0; COUNT]; 6];
    for j in 0..4 {
        let mut coefficients = vec![0.0; COUNT];
        coefficients[4 + j] = ELEVATION[j];
        bounds.push(LinearConstraint {
            coefficients,
            value: 1e-6 - seam_depth - seam_depth_derivative * span * (j + 2) as f64 / 5.0,
        });
        center[j + 2][j] = ELEVATION[j];
        center[j + 2][j + 4] = ELEVATION[j];
    }
    // Center z'' is a cubic Bernstein polynomial (the affine baseline has zero
    // second derivative). Restrict its controls by de Casteljau to [0,neck_v]:
    // region above the neckline is absent, and must not dictate its curvature.
    let mut derivative = (0..4)
        .map(|j| {
            std::array::from_fn::<_, COUNT, _>(|i| {
                20.0 * (center[j + 2][i] - 2.0 * center[j + 1][i] + center[j][i]) / span.powi(2)
            })
        })
        .collect::<Vec<_>>();
    while !derivative.is_empty() {
        bounds.push(LinearConstraint {
            coefficients: derivative[0].iter().map(|a| -a).collect(),
            value: 0.0,
        });
        derivative = derivative
            .windows(2)
            .map(|pair| std::array::from_fn(|i| (1.0 - neck_v) * pair[0][i] + neck_v * pair[1][i]))
            .collect();
    }
    Ok(bounds)
}

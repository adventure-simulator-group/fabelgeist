//! Compact, body-supported meridians for an angular elliptical torso loft.
//!
//! Four positive Bernstein coefficients per radius, with nonpositive second
//! differences, make each radius concave over physical height. Thus the fit
//! cannot add alternating chest/waist dents. The coronal center is affine.
//! Containment is certified only for the supplied anterior torso samples at
//! their supplied heights. Inflating each sample by an axis-aligned square is
//! conservative in that slice, but is NOT a 3D normal-clearance certificate or
//! a guarantee for unsampled body triangles/heights, yokes, or skirt geometry.

use crate::breastplate_qp::{LinearConstraint, QpOptions, solve_dense_qp};

#[derive(Clone, Debug)]
pub(super) struct TorsoSlice {
    pub height: f64,
    /// Local torso coordinates: x is lateral and z is anterior.
    pub points: Vec<[f64; 2]>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LoftFitOptions {
    pub clearance_m: f64,
    pub crown_m: f64,
    /// Extra lateral room AFTER sample inflation, avoiding endpoint pinches.
    pub lateral_guard_m: f64,
    /// Authored change in waist ease, tapering to zero at the upper seam.
    pub waist_radius_delta_m: f64,
    /// Positive multiplier on fairing energy; one preserves the reference fit.
    pub fairing_multiplier: f64,
}

impl Default for LoftFitOptions {
    fn default() -> Self {
        Self {
            clearance_m: 0.01,
            crown_m: 0.02,
            lateral_guard_m: 0.015,
            waist_radius_delta_m: 0.0,
            fairing_multiplier: 1.0,
        }
    }
}

impl LoftFitOptions {
    fn lateral_guard(self, t: f64) -> (f64, f64) {
        // Keep the exact default arithmetic and never trade skin padding for
        // requested narrowness. This floor limits extra room, not clearance.
        let requested = if self.waist_radius_delta_m == 0.0 {
            self.lateral_guard_m
        } else {
            self.lateral_guard_m + self.waist_radius_delta_m * (1.0 - t).powi(2)
        };
        (requested, requested.max(0.25 * self.lateral_guard_m))
    }
}

/// Fairing within the smooth, concave plate family, not anatomical detail gain.
pub(super) fn plate_fairing_multiplier(rigidity: f64) -> f64 {
    0.01 + 0.99 * rigidity * rigidity
}

#[derive(Clone, Debug)]
pub(super) struct LoftFitReport {
    pub anterior_sample_count: usize,
    /// max((inflated_x / a)^2 + (inflated_z / b)^2 - 1, 0).
    pub max_inflated_ellipse_residual: f64,
    pub max_coronal_fit_error_m: f64,
    pub minimum_lateral_reserve_m: f64,
    pub minimum_requested_lateral_guard_m: f64,
    pub minimum_applied_lateral_guard_m: f64,
    pub lateral_guard_saturated_slice_count: usize,
}

#[derive(Clone, Debug)]
pub(super) struct LoftProfiles {
    pub width_controls: [f64; 4],
    pub depth_controls: [f64; 4],
    pub report: LoftFitReport,
    height_min: f64,
    height_span: f64,
    // c(y) = coronal[0] + coronal[1] * normalized physical height.
    coronal: [f64; 2],
}

impl LoftProfiles {
    pub fn height_range(&self) -> [f64; 2] {
        [self.height_min, self.height_min + self.height_span]
    }

    fn parameter(&self, height: f64) -> f64 {
        let t = (height - self.height_min) / self.height_span;
        assert!(
            t.is_finite() && (-1e-10..=1.0 + 1e-10).contains(&t),
            "Compact loft queried outside its fitted physical-height domain"
        );
        t.clamp(0.0, 1.0)
    }

    /// [lateral radius, anterior radius, coronal center]. Panics for queries
    /// outside the fit domain; skirt/yoke continuation belongs to the caller.
    pub fn eval(&self, height: f64) -> [f64; 3] {
        let t = self.parameter(height);
        let basis = bernstein(t);
        [
            dot4(&basis, &self.width_controls),
            dot4(&basis, &self.depth_controls),
            self.coronal[0] + self.coronal[1] * t,
        ]
    }

    /// Physical-height derivatives, in-domain one-sided at the endpoints.
    pub fn derivative(&self, height: f64) -> [f64; 3] {
        let t = self.parameter(height);
        let radial = |controls: &[f64; 4]| {
            3.0 * ((1.0 - t).powi(2) * (controls[1] - controls[0])
                + 2.0 * t * (1.0 - t) * (controls[2] - controls[1])
                + t * t * (controls[3] - controls[2]))
                / self.height_span
        };
        [
            radial(&self.width_controls),
            radial(&self.depth_controls),
            self.coronal[1] / self.height_span,
        ]
    }
}

fn bernstein(t: f64) -> [f64; 4] {
    let s = 1.0 - t;
    [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t]
}

fn dot4(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

fn ellipse_depth_bound(inflated_x: f64, inflated_z: f64, a: f64) -> Result<f64, String> {
    if !a.is_finite() || a <= 0.0 || inflated_x >= a {
        return Err(format!(
            "Lateral sample cannot fit ellipse: inflated |x|={inflated_x} >= a={a}"
        ));
    }
    // No denominator clamping: invalid lateral support is an error.
    let need = inflated_z / (1.0 - (inflated_x / a).powi(2)).sqrt();
    if !need.is_finite() {
        return Err("Nonfinite ellipse depth support".into());
    }
    Ok(need)
}

fn fit_radius(
    samples: &[(f64, f64, f64)],
    length_scale: f64,
    fairing_multiplier: f64,
) -> Result<[f64; 4], String> {
    // Solve in dimensionless radii so the certificate is insensitive to units.
    let mut h = vec![vec![0.0; 4]; 4];
    let mut rhs = vec![0.0; 4];
    let mut bounds = Vec::new();
    let weight = 1.0 / samples.len() as f64;
    for &(t, lower, target) in samples {
        let row = bernstein(t);
        for i in 0..4 {
            rhs[i] += weight * row[i] * target / length_scale;
            for j in 0..4 {
                h[i][j] += weight * row[i] * row[j];
            }
        }
        bounds.push(LinearConstraint {
            coefficients: row.to_vec(),
            value: lower / length_scale,
        });
    }
    for i in 0..4 {
        h[i][i] += 1e-8;
        let mut row = vec![0.0; 4];
        row[i] = 1.0;
        bounds.push(LinearConstraint {
            coefficients: row,
            value: 1e-6,
        });
    }
    // a''(t), b''(t) <= 0 for the entire continuous domain (not sampled only).
    for i in 0..2 {
        let mut row = vec![0.0; 4];
        row[i] = -1.0;
        row[i + 1] = 2.0;
        row[i + 2] = -1.0;
        bounds.push(LinearConstraint {
            coefficients: row,
            value: 0.0,
        });
        // Mild bending energy chooses the simplest supporting meridian.
        for j in 0..4 {
            for k in 0..4 {
                let dj = if j == i || j == i + 2 {
                    1.0
                } else if j == i + 1 {
                    -2.0
                } else {
                    0.0
                };
                let dk = if k == i || k == i + 2 {
                    1.0
                } else if k == i + 1 {
                    -2.0
                } else {
                    0.0
                };
                h[j][k] += (0.002 * fairing_multiplier) * dj * dk;
            }
        }
    }
    let result = solve_dense_qp(
        &h,
        &rhs,
        &[],
        &bounds,
        QpOptions {
            max_sweeps: 20_000,
            primal_tolerance: 1e-10,
            complementarity_tolerance: 1e-10,
            ..Default::default()
        },
    )
    .map_err(|error| {
        format!(
            "Compact radius fit failed: {:?}: {}",
            error.kind, error.message
        )
    })?;
    let mut controls = [0.0; 4];
    for (out, value) in controls.iter_mut().zip(result.coefficients) {
        *out = value * length_scale;
    }
    Ok(controls)
}

fn validated_height_domain(
    slices: &[TorsoSlice],
    coronal_samples: &[[f64; 2]],
    options: LoftFitOptions,
) -> Result<(f64, f64), String> {
    if slices.len() < 2
        || coronal_samples.len() < 2
        || slices.iter().any(|s| {
            !s.height.is_finite()
                || s.points.is_empty()
                || s.points.iter().flatten().any(|x| !x.is_finite())
        })
        || coronal_samples.iter().flatten().any(|x| !x.is_finite())
        || !options.clearance_m.is_finite()
        || options.clearance_m < 0.0
        || !options.crown_m.is_finite()
        || options.crown_m < 0.0
        || !options.lateral_guard_m.is_finite()
        || options.lateral_guard_m <= 0.0
        || !options.waist_radius_delta_m.is_finite()
        || !options.fairing_multiplier.is_finite()
        || options.fairing_multiplier <= 0.0
    {
        return Err("Invalid compact loft samples or options".into());
    }
    let min = slices
        .iter()
        .map(|s| s.height)
        .fold(f64::INFINITY, f64::min);
    let max = slices
        .iter()
        .map(|s| s.height)
        .fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    if !span.is_finite() || span <= 0.0 {
        return Err("Loft needs distinct physical heights".into());
    }
    Ok((min, span))
}

struct DepthSupport {
    samples: Vec<(f64, f64, f64)>,
    anterior_sample_count: usize,
    minimum_lateral_reserve_m: f64,
}

fn assemble_depth_support(
    slices: &[TorsoSlice],
    width_controls: &[f64; 4],
    min: f64,
    span: f64,
    center: impl Fn(f64) -> f64,
    options: LoftFitOptions,
) -> Result<DepthSupport, String> {
    let mut depth_samples = Vec::new();
    let mut count = 0;
    let mut reserve = f64::INFINITY;
    for slice in slices {
        let t = (slice.height - min) / span;
        let a = dot4(&bernstein(t), width_controls);
        let c = center(slice.height);
        let mut required: f64 = 0.0;
        let mut row_count = 0;
        for &[x, z] in &slice.points {
            if z < c {
                continue;
            }
            let inflated_x = x.abs() + options.clearance_m;
            reserve = reserve.min(a - inflated_x);
            let need = ellipse_depth_bound(inflated_x, z - c + options.clearance_m, a)
                .map_err(|error| format!("{error} at height {}", slice.height))?;
            required = required.max(need);
            row_count += 1;
        }
        if row_count == 0 {
            return Err(format!(
                "No anterior torso samples at height {}",
                slice.height
            ));
        }
        count += row_count;
        depth_samples.push((
            t,
            required.max(span * 1e-6),
            required + options.crown_m * 4.0 * t * (1.0 - t),
        ));
    }
    Ok(DepthSupport {
        samples: depth_samples,
        anterior_sample_count: count,
        minimum_lateral_reserve_m: reserve,
    })
}

/// Fit conservative elliptical profiles from already-separated torso slices.
/// No body-component classification, percentile reduction, vertex displacement,
/// or topology-dependent fitting is performed here.
pub(super) fn fit_loft_profiles(
    slices: &[TorsoSlice],
    coronal_samples: &[[f64; 2]],
    options: LoftFitOptions,
) -> Result<LoftProfiles, String> {
    let (min, span) = validated_height_domain(slices, coronal_samples, options)?;
    let mean_t = coronal_samples
        .iter()
        .map(|p| (p[0] - min) / span)
        .sum::<f64>()
        / coronal_samples.len() as f64;
    let mean_c = coronal_samples.iter().map(|p| p[1]).sum::<f64>() / coronal_samples.len() as f64;
    let variance = coronal_samples
        .iter()
        .map(|p| ((p[0] - min) / span - mean_t).powi(2))
        .sum::<f64>();
    if !variance.is_finite() || variance <= 1e-18 {
        return Err("Coronal anchors need distinct heights".into());
    }
    let slope = coronal_samples
        .iter()
        .map(|p| ((p[0] - min) / span - mean_t) * (p[1] - mean_c))
        .sum::<f64>()
        / variance;
    let coronal = [mean_c - slope * mean_t, slope];
    let center = |y: f64| coronal[0] + coronal[1] * (y - min) / span;
    let mut minimum_requested_guard = f64::INFINITY;
    let mut minimum_applied_guard = f64::INFINITY;
    let mut saturated_slices = 0;
    let widths: Vec<_> = slices
        .iter()
        .map(|s| {
            let t = (s.height - min) / span;
            let (requested, applied) = options.lateral_guard(t);
            minimum_requested_guard = minimum_requested_guard.min(requested);
            minimum_applied_guard = minimum_applied_guard.min(applied);
            saturated_slices += usize::from(applied > requested);
            let radius = s.points.iter().map(|p| p[0].abs()).fold(0.0, f64::max)
                + options.clearance_m
                + applied;
            (t, radius, radius)
        })
        .collect();
    let width_controls = fit_radius(&widths, span, options.fairing_multiplier)?;
    let support = assemble_depth_support(slices, &width_controls, min, span, center, options)?;
    let depth_controls = fit_radius(&support.samples, span, options.fairing_multiplier)?;
    let mut profiles = LoftProfiles {
        width_controls,
        depth_controls,
        height_min: min,
        height_span: span,
        coronal,
        report: LoftFitReport {
            anterior_sample_count: support.anterior_sample_count,
            max_inflated_ellipse_residual: 0.0,
            max_coronal_fit_error_m: coronal_samples
                .iter()
                .map(|p| (center(p[0]) - p[1]).abs())
                .fold(0.0, f64::max),
            minimum_lateral_reserve_m: support.minimum_lateral_reserve_m,
            minimum_requested_lateral_guard_m: minimum_requested_guard,
            minimum_applied_lateral_guard_m: minimum_applied_guard,
            lateral_guard_saturated_slice_count: saturated_slices,
        },
    };
    for s in slices {
        let [a, b, c] = profiles.eval(s.height);
        for &[x, z] in &s.points {
            if z < c {
                continue;
            }
            let residual = ((x.abs() + options.clearance_m) / a).powi(2)
                + ((z - c + options.clearance_m) / b).powi(2)
                - 1.0;
            if !residual.is_finite() {
                return Err("Nonfinite ellipse support certificate".into());
            }
            profiles.report.max_inflated_ellipse_residual =
                profiles.report.max_inflated_ellipse_residual.max(residual);
        }
    }
    if profiles.report.max_inflated_ellipse_residual > 1e-7 {
        return Err(format!(
            "Ellipse sample certificate failed: {}",
            profiles.report.max_inflated_ellipse_residual
        ));
    }
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ellipse(scale: f64) -> (Vec<TorsoSlice>, Vec<[f64; 2]>) {
        let slices = (0..9)
            .map(|i| {
                let t = i as f64 / 8.0;
                let height = scale * (1.0 + 0.4 * t);
                let c = scale * (0.03 + 0.02 * t);
                TorsoSlice {
                    height,
                    points: (0..=64)
                        .map(|j| {
                            let angle = std::f64::consts::PI * j as f64 / 64.0;
                            [scale * 0.2 * angle.cos(), c + scale * 0.13 * angle.sin()]
                        })
                        .collect(),
                }
            })
            .collect();
        (
            slices,
            vec![[scale, scale * 0.03], [scale * 1.4, scale * 0.05]],
        )
    }

    #[test]
    fn known_ellipse_is_contained_and_profiles_are_positive_concave() {
        let (slices, anchors) = ellipse(1.0);
        let result = fit_loft_profiles(&slices, &anchors, LoftFitOptions::default()).unwrap();
        assert!(result.report.max_inflated_ellipse_residual < 1e-7);
        // Endpoint sin(pi) and affine-fit roundoff can classify an exact
        // coronal endpoint infinitesimally behind the front half-plane.
        assert!(result.report.anterior_sample_count >= 9 * 63);
        for controls in [result.width_controls, result.depth_controls] {
            assert!(controls.iter().all(|c| *c > 0.0));
            for i in 0..2 {
                assert!(controls[i] - 2.0 * controls[i + 1] + controls[i + 2] < 1e-8);
            }
        }
        for s in slices {
            let [a, b, c] = result.eval(s.height);
            for [x, z] in s.points {
                assert!((x / a).powi(2) + ((z - c) / b).powi(2) <= 1.0 + 1e-8);
            }
        }
    }

    #[test]
    fn affine_coronal_fit_and_endpoint_derivatives_use_physical_height() {
        let (slices, anchors) = ellipse(1.0);
        let result = fit_loft_profiles(&slices, &anchors, LoftFitOptions::default()).unwrap();
        assert_eq!(result.height_range(), [1.0, 1.4]);
        assert!((result.eval(1.2)[2] - 0.04).abs() < 1e-12);
        assert!((result.derivative(1.2)[2] - 0.05).abs() < 1e-12);
        assert!((result.eval(1.0)[0] - result.width_controls[0]).abs() < 1e-12);
        assert!((result.eval(1.4)[1] - result.depth_controls[3]).abs() < 1e-12);
        assert!(std::panic::catch_unwind(|| result.eval(0.8)).is_err());
        assert!(std::panic::catch_unwind(|| result.derivative(1.6)).is_err());
        assert!(result.report.max_coronal_fit_error_m < 1e-12);
    }

    #[test]
    fn scaling_physical_units_scales_entire_fit() {
        let (s1, c1) = ellipse(1.0);
        let (s2, c2) = ellipse(10.0);
        let a = fit_loft_profiles(&s1, &c1, LoftFitOptions::default()).unwrap();
        let b = fit_loft_profiles(
            &s2,
            &c2,
            LoftFitOptions {
                clearance_m: 0.1,
                crown_m: 0.2,
                lateral_guard_m: 0.15,
                ..Default::default()
            },
        )
        .unwrap();
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            for (u, v) in a
                .eval(1.0 + 0.4 * t)
                .into_iter()
                .zip(b.eval(10.0 + 4.0 * t))
            {
                assert!((u * 10.0 - v).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn zero_crown_and_zero_padding_still_support_body() {
        let (slices, anchors) = ellipse(1.0);
        let result = fit_loft_profiles(
            &slices,
            &anchors,
            LoftFitOptions {
                clearance_m: 0.0,
                crown_m: 0.0,
                lateral_guard_m: 0.015,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(result.report.max_inflated_ellipse_residual < 1e-7);
        assert!(result.report.minimum_lateral_reserve_m > 0.0149);
    }

    #[test]
    fn invalid_or_degenerate_inputs_are_rejected() {
        let (mut slices, anchors) = ellipse(1.0);
        assert!(
            fit_loft_profiles(
                &slices,
                &anchors,
                LoftFitOptions {
                    lateral_guard_m: 0.0,
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert!(
            fit_loft_profiles(
                &slices,
                &[[1.0, 0.0], [1.0, 0.1]],
                LoftFitOptions::default()
            )
            .is_err()
        );
        for s in &mut slices {
            s.height = 1.0;
        }
        assert!(fit_loft_profiles(&slices, &anchors, LoftFitOptions::default()).is_err());
    }

    #[test]
    fn lateral_obstruction_is_an_error_not_a_clamped_denominator() {
        assert!(ellipse_depth_bound(0.2, 0.1, 0.2).is_err());
        assert!(ellipse_depth_bound(0.21, 0.1, 0.2).is_err());
        let depth = ellipse_depth_bound(0.12, 0.08, 0.2).unwrap();
        assert!((depth - 0.1).abs() < 1e-12);
    }

    #[test]
    fn default_controls_preserve_unparameterized_radius_targets_exactly() {
        let options = LoftFitOptions::default();
        assert_eq!(plate_fairing_multiplier(1.0).to_bits(), 1.0_f64.to_bits());
        for t in [0.0, 0.1, 0.5, 0.9, 1.0] {
            let (requested, applied) = options.lateral_guard(t);
            assert_eq!(requested.to_bits(), options.lateral_guard_m.to_bits());
            assert_eq!(applied.to_bits(), options.lateral_guard_m.to_bits());
        }
        let (slices, anchors) = ellipse(1.0);
        let result = fit_loft_profiles(&slices, &anchors, options).unwrap();
        let min = slices.first().unwrap().height;
        let span = slices.last().unwrap().height - min;
        let original_targets = slices
            .iter()
            .map(|slice| {
                let radius = slice.points.iter().map(|p| p[0].abs()).fold(0.0, f64::max)
                    + options.clearance_m
                    + options.lateral_guard_m;
                ((slice.height - min) / span, radius, radius)
            })
            .collect::<Vec<_>>();
        let original_width = fit_radius(&original_targets, span, 1.0).unwrap();
        assert_eq!(
            result.width_controls.map(f64::to_bits),
            original_width.map(f64::to_bits)
        );
        assert_eq!(result.report.lateral_guard_saturated_slice_count, 0);
    }

    fn assert_inflated_containment(profiles: &LoftProfiles, slices: &[TorsoSlice], clearance: f64) {
        assert!(profiles.report.max_inflated_ellipse_residual <= 1e-7);
        for slice in slices {
            let [a, b, c] = profiles.eval(slice.height);
            for &[x, z] in &slice.points {
                if z >= c {
                    assert!(
                        ((x.abs() + clearance) / a).powi(2) + ((z - c + clearance) / b).powi(2)
                            <= 1.0 + 1e-7
                    );
                }
            }
        }
        for controls in [profiles.width_controls, profiles.depth_controls] {
            assert!(controls.iter().all(|v| *v > 0.0));
            for i in 0..2 {
                assert!(controls[i] - 2.0 * controls[i + 1] + controls[i + 2] <= 1e-8);
            }
        }
    }

    #[test]
    fn waist_ease_has_ordered_response_without_changing_body_padding() {
        let (slices, anchors) = ellipse(1.0);
        let mut waist_radii = Vec::new();
        for width in [550, 740, 1_000] {
            let options = LoftFitOptions {
                waist_radius_delta_m: 0.055 * 0.5 * (f64::from(width) - 740.0) / 1_000.0,
                ..Default::default()
            };
            let result = fit_loft_profiles(&slices, &anchors, options).unwrap();
            assert_inflated_containment(&result, &slices, options.clearance_m);
            assert_eq!(result.report.lateral_guard_saturated_slice_count, 0);
            waist_radii.push(result.eval(slices[0].height)[0]);
        }
        assert!(waist_radii[1] - waist_radii[0] > 0.004);
        assert!(waist_radii[2] - waist_radii[1] > 0.005);
    }

    #[test]
    fn extreme_narrow_ease_reports_reserve_saturation_and_contains_body() {
        let (slices, anchors) = ellipse(1.0);
        let options = LoftFitOptions {
            waist_radius_delta_m: -0.1,
            ..Default::default()
        };
        let result = fit_loft_profiles(&slices, &anchors, options).unwrap();
        assert!(result.report.lateral_guard_saturated_slice_count > 0);
        assert!(result.report.minimum_requested_lateral_guard_m < 0.0);
        assert_eq!(result.report.minimum_applied_lateral_guard_m, 0.00375);
        assert_inflated_containment(&result, &slices, options.clearance_m);
    }

    fn radius_bending_energy(controls: [f64; 4]) -> f64 {
        (0..2)
            .map(|i| (controls[i] - 2.0 * controls[i + 1] + controls[i + 2]).powi(2))
            .sum()
    }

    #[test]
    fn rigidity_decreases_bending_energy_for_fixed_targets_and_constraints() {
        let samples = (0..=16)
            .map(|i| {
                let t = f64::from(i) / 16.0;
                (t, 0.1, 0.2 + 0.04 * 4.0 * t * (1.0 - t))
            })
            .collect::<Vec<_>>();
        let mut energies = Vec::new();
        let mut fits = Vec::new();
        for rigidity in [0.0, 0.5, 1.0] {
            let controls = fit_radius(&samples, 0.4, plate_fairing_multiplier(rigidity)).unwrap();
            energies.push(radius_bending_energy(controls));
            fits.push(controls);
            assert!(
                samples
                    .iter()
                    .all(|&(t, lower, _)| dot4(&bernstein(t), &controls) >= lower)
            );
        }
        assert!(energies[0] > energies[1] && energies[1] > energies[2]);
        assert!(
            fits[0]
                .iter()
                .zip(fits[2])
                .any(|(a, b)| (a - b).abs() > 1e-5)
        );
    }

    #[test]
    fn rigidity_changes_supported_loft_without_removing_concavity_or_padding() {
        let (slices, anchors) = ellipse(1.0);
        let mut depths = Vec::new();
        for rigidity in [0.0, 0.5, 1.0] {
            let options = LoftFitOptions {
                fairing_multiplier: plate_fairing_multiplier(rigidity),
                ..Default::default()
            };
            let result = fit_loft_profiles(&slices, &anchors, options).unwrap();
            assert_inflated_containment(&result, &slices, options.clearance_m);
            depths.push(result.depth_controls);
        }
        assert!(
            depths[0]
                .iter()
                .zip(depths[2])
                .any(|(a, b)| (a - b).abs() > 1e-5)
        );
    }
}

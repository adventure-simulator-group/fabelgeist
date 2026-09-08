use super::*;

fn field(angle: f64) -> GlobalUpperLoft {
    GlobalUpperLoft {
        elliptical_depth: false,
        seam_y: 1.3,
        span: 0.2,
        angle,
        arc_scale: 0.27,
        seam: [0.2, 0.13, 0.02],
        seam_derivative: [0.01, -0.03, -0.01],
        radius_correction: [0.01, -0.006],
        prior_depth_correction: 0.0,
        depth_controls: vec![0.005; COUNT],
    }
}

#[test]
fn global_upper_matches_lower_value_derivative_and_normals_at_seam() {
    for angle in [
        std::f64::consts::FRAC_PI_2,
        f64::from(std::f32::consts::FRAC_PI_2),
        1.48,
    ] {
        let surface = field(angle);
        for u in [0.0, 0.3, 0.7, 1.0] {
            let theta = u * angle;
            let (p, n) = surface.evaluate(u, surface.seam_y);
            assert!((p[0] - surface.seam[0] * theta.sin()).abs() < 1e-14);
            assert!((p[2] - surface.seam[2] - surface.seam[1] * theta.cos()).abs() < 1e-14);
            let e = 1e-7;
            let q = surface.evaluate(u, surface.seam_y + e).0;
            assert!(((q[0] - p[0]) / e - surface.seam_derivative[0] * theta.sin()).abs() < 1e-6);
            assert!(
                ((q[2] - p[2]) / e
                    - surface.seam_derivative[2]
                    - surface.seam_derivative[1] * theta.cos())
                .abs()
                    < 1e-6
            );
            let nn = surface.evaluate(u, surface.seam_y + e).1;
            assert!(n.iter().zip(nn).map(|(a, b)| a * b).sum::<f64>() > 1.0 - 1e-10);
        }
    }
}

#[test]
fn global_upper_normals_are_actual_parametric_tangents() {
    let surface = field(1.48);
    for u in [0.1, 0.5, 0.9] {
        for y in [1.31, 1.4, 1.49] {
            let e = 1e-6;
            let (_, n) = surface.evaluate(u, y);
            let a = surface.evaluate(u - e, y).0;
            let b = surface.evaluate(u + e, y).0;
            let c = surface.evaluate(u, y - e).0;
            let d = surface.evaluate(u, y + e).0;
            for tangent in [
                std::array::from_fn::<_, 3, _>(|i| (b[i] - a[i]) / (2.0 * e)),
                std::array::from_fn(|i| (d[i] - c[i]) / (2.0 * e)),
            ] {
                assert!(n.iter().zip(tangent).map(|(a, b)| a * b).sum::<f64>().abs() < 1e-8);
            }
        }
    }
}

#[test]
fn trimmed_angular_coverage_is_respected_by_radius_fit_and_inverse() {
    let mut surface = field(1.48);
    let guides = [UpperGuide {
        xy: [0.215, 1.42],
        target: 0.08,
        ceiling: None,
        normal: None,
    }];
    surface.fit_radius(&guides, &[]).unwrap();
    assert!(surface.parameter(guides[0].xy).unwrap() < 1.0);
    for u in [0.0, 0.3, 0.8, 1.0] {
        let p = surface.evaluate(u, 1.42).0;
        assert!((surface.parameter([p[0], p[1]]).unwrap() - u).abs() < 1e-12);
    }
    assert!(surface.parameter([surface.radius(1.42).0, 1.42]).is_err());
}

#[test]
fn constrained_global_fit_supports_body_and_is_not_an_old_yoke_target() {
    // Actual legacy lower-field representation, not independently rounded f64.
    let mut surface = field(f64::from(std::f32::consts::FRAC_PI_2));
    surface.seam_derivative = [0.0; 3];
    let guides = (0..=8)
        .map(|i| {
            let u = i as f64 / 8.0;
            UpperGuide {
                xy: [0.2 * (u * surface.angle).sin(), 1.5 - 0.1 * u],
                target: 0.02 + 0.13 * (u * surface.angle).cos(),
                ceiling: None,
                normal: None,
            }
        })
        .collect::<Vec<_>>();
    let mut outline = guides.iter().map(|g| g.xy).collect::<Vec<_>>();
    outline.push([0.2 * surface.angle.sin(), 1.3]);
    outline.push([-0.2 * surface.angle.sin(), 1.3]);
    outline.extend(guides.iter().rev().map(|g| [-g.xy[0], g.xy[1]]));
    let body =
        |x: f64, _: f64| (x.abs() < 0.18).then(|| 0.02 + 0.10 * (1.0 - (x / 0.18).powi(2)).sqrt());
    let fitted = surface.fit(&guides, &outline, 0.002, 1.0, body).unwrap();
    let endpoint = fitted.evaluate(1.0, 1.4).0;
    assert_eq!(fitted.parameter([endpoint[0], endpoint[1]]).unwrap(), 1.0);
    let mut outside = fitted.clone();
    outside.angle += 1e-8;
    assert!(outside.fit(&guides, &outline, 0.002, 1.0, body).is_err());
    for u in [0.0, 0.2, 0.5, 0.7] {
        for y in [1.31, 1.35, 1.39] {
            let p = fitted.evaluate(u, y).0;
            assert!(p[2] + 1e-7 >= body(p[0], y).unwrap() + 0.002);
        }
    }
    assert!(fitted.depth_controls.iter().all(|v| v.is_finite()));
}

#[test]
fn radius_fit_covers_actual_inverse_queries_between_guide_stations() {
    let mut surface = field(f64::from(std::f32::consts::FRAC_PI_2));
    let guides = [UpperGuide {
        xy: [0.18, 1.4],
        target: 0.08,
        ceiling: None,
        normal: None,
    }];
    surface.fit_radius(&guides, &[]).unwrap();
    // Same relative overrun as the real integration failure: the query is not
    // one of the sparse guides, so a guide-only fit cannot certify it.
    let query = [surface.radius(1.31).0 * 1.0000073913836252, 1.31];
    let error = surface.parameter(query).unwrap_err();
    assert!(error.contains("xy=") && error.contains("radius=") && error.contains("dy="));
    let outline = [query, [0.18, 1.4]];
    surface.fit_radius(&guides, &outline).unwrap();
    for i in 0..=100 {
        let t = f64::from(i) / 100.0;
        let xy = std::array::from_fn(|j| outline[0][j] * (1.0 - t) + outline[1][j] * t);
        assert!(surface.parameter(xy).unwrap() <= 1.0);
    }
}

#[test]
#[ignore = "requires BREASTPLATE_GLOBAL_UPPER_FIXTURE pointing at profile.global-inputs.json"]
fn actual_global_upper_radius_inputs_can_be_replayed_without_creator() {
    let path = std::env::var_os("BREASTPLATE_GLOBAL_UPPER_FIXTURE").unwrap();
    let input: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let mut surface: GlobalUpperLoft = serde_json::from_value(input["surface"].clone()).unwrap();
    let guides: Vec<UpperGuide> = serde_json::from_value(input["guides"].clone()).unwrap();
    let outline: Vec<[f64; 2]> = serde_json::from_value(input["outline_xy"].clone()).unwrap();
    let mut legacy = surface.clone();
    legacy.fit_radius_policy(&guides, &outline, true).unwrap();
    eprintln!(
        "minimum-bending ablation radius corrections={:?} top={}",
        legacy.radius_correction,
        legacy.radius(legacy.seam_y + legacy.span).0
    );
    surface.fit_radius(&guides, &outline).unwrap();
    eprintln!(
        "replayed upper radius corrections={:?} seam/top={}/{}",
        surface.radius_correction,
        surface.radius(surface.seam_y).0,
        surface.radius(surface.seam_y + surface.span).0
    );
    for xy in outline.iter().chain(guides.iter().map(|g| &g.xy)) {
        if xy[1] > surface.seam_y + 1e-6 {
            assert!(surface.parameter(*xy).unwrap() <= 1.0);
        }
    }
}

#[test]
fn intermediate_body_depth_targets_do_not_author_upper_plate_waviness() {
    let surface = field(1.48);
    let mut guides = [
        UpperGuide {
            xy: [0.0, 1.45],
            target: 0.12,
            ceiling: None,
            normal: None,
        },
        UpperGuide {
            xy: [0.08, 1.48],
            target: 0.10,
            ceiling: None,
            normal: Some([0.1, 0.3, 0.95]),
        },
        UpperGuide {
            xy: [0.10, 1.49],
            target: 0.09,
            ceiling: None,
            normal: Some([0.15, 0.25, 0.925]),
        },
        UpperGuide {
            xy: [0.12, 1.50],
            target: 0.08,
            ceiling: None,
            normal: Some([0.2, 0.2, 0.9]),
        },
        UpperGuide {
            xy: [0.19, 1.40],
            target: 0.06,
            ceiling: None,
            normal: None,
        },
    ];
    let mut outline = guides.iter().map(|g| g.xy).collect::<Vec<_>>();
    outline.extend([[0.195, 1.3], [-0.195, 1.3]]);
    outline.extend(guides.iter().rev().map(|g| [-g.xy[0], g.xy[1]]));
    let original = surface
        .clone()
        .fit(&guides, &outline, 0.002, 0.01, |_, _| Some(0.0))
        .unwrap();
    guides[2].target += 0.035;
    guides[4].target -= 0.030;
    let changed = surface
        .fit(&guides, &outline, 0.002, 0.01, |_, _| Some(0.0))
        .unwrap();
    assert_eq!(
        original
            .depth_controls
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        changed
            .depth_controls
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>()
    );
}

#[test]
fn exact_elliptical_upper_matches_closed_sections_and_analytic_normals() {
    for angle in [1.48, f64::from(std::f32::consts::FRAC_PI_2)] {
        let mut surface = field(angle);
        surface.elliptical_depth = true;
        surface.depth_controls = vec![-0.02, -0.01, -0.03, -0.02, 0.03, 0.01, 0.02, 0.04];
        for v in [0.0, 0.17, 0.53, 0.91] {
            let y = surface.seam_y + v * surface.span;
            let h = bernstein(3, v);
            let c = surface.seam[2]
                + surface.seam_derivative[2] * v * surface.span
                + v * v
                    * h.iter()
                        .zip(&surface.depth_controls[..4])
                        .map(|(a, b)| a * b)
                        .sum::<f64>();
            let b = surface.seam[1]
                + surface.seam_derivative[1] * v * surface.span
                + v * v
                    * h.iter()
                        .zip(&surface.depth_controls[4..])
                        .map(|(a, b)| a * b)
                        .sum::<f64>();
            for u in [0.0, 0.2, 0.6, 0.9, 1.0] {
                let (p, n) = surface.evaluate(u, y);
                assert!((p[2] - c - b * (u * angle).cos()).abs() < 1e-14);
                assert!(
                    ((p[0] / surface.radius(y).0).powi(2) + ((p[2] - c) / b).powi(2) - 1.0).abs()
                        < 1e-13
                );
                let e = 1e-6;
                for (a, b) in [
                    (surface.evaluate(u - e, y).0, surface.evaluate(u + e, y).0),
                    (surface.evaluate(u, y - e).0, surface.evaluate(u, y + e).0),
                ] {
                    let dot = (0..3)
                        .map(|j| n[j] * (b[j] - a[j]) / (2.0 * e))
                        .sum::<f64>();
                    assert!(dot.abs() < 1e-8);
                }
                if v == 0.0 {
                    let rows = surface.rows(u, y);
                    assert!(
                        (rows[2][0]
                            + rows[2][1..]
                                .iter()
                                .zip(&surface.depth_controls)
                                .map(|(a, b)| a * b)
                                .sum::<f64>()
                            - surface.seam_derivative[2]
                            - surface.seam_derivative[1] * (u * angle).cos())
                        .abs()
                            < 1e-14
                    );
                }
            }
        }
    }
}

#[test]
fn elliptical_shape_bounds_certify_positive_depth_and_non_bowl_center_continuously() {
    let mut surface = field(1.48);
    surface.elliptical_depth = true;
    let constraints = elliptical::shape_bounds(
        surface.seam[1],
        surface.seam_derivative[1],
        surface.span,
        0.8,
    )
    .unwrap();
    // A desired positive center hump that unconstrained fitting would bend
    // inward. The hard shape family must override that desired coefficient.
    let result = solve_dense_qp(
        &(0..8)
            .map(|i| (0..8).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect::<Vec<_>>(),
        &[0.2, 0.4, 0.1, 0.0, 0.2, 0.4, 0.1, 0.0],
        &[],
        &constraints,
        QpOptions::default(),
    )
    .unwrap();
    surface.depth_controls = result.coefficients;
    for row in &constraints {
        assert!(
            row.coefficients
                .iter()
                .zip(&surface.depth_controls)
                .map(|(a, b)| a * b)
                .sum::<f64>()
                >= row.value - 1e-10
        );
    }
    for j in 0..=1000 {
        let v = j as f64 / 1000.0;
        let h = bernstein(3, v);
        let b = surface.seam[1]
            + surface.seam_derivative[1] * surface.span * v
            + v * v
                * h.iter()
                    .zip(&surface.depth_controls[4..])
                    .map(|(a, b)| a * b)
                    .sum::<f64>();
        assert!(b >= 1e-6 - 1e-10);
        if v <= 0.8 {
            let rows = surface.rows(0.0, surface.seam_y + surface.span * v);
            assert!(
                rows[4][1..]
                    .iter()
                    .zip(&surface.depth_controls)
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
                    <= 1e-8
            );
        }
    }
    assert!(elliptical::shape_bounds(0.01, -1.0, 0.2, 0.8).is_err());
    assert!(elliptical::shape_bounds(0.13, 0.0, 0.2, 1.01).is_err());
}

#[test]
fn elliptical_fit_retains_body_floors_ceilings_and_seam_without_global_convexity_claim() {
    for fairing in [1.0, 0.01] {
        let mut surface = field(1.48);
        surface.seam_derivative = [0.0; 3];
        let guides = (0..=8)
            .map(|i| {
                let u = i as f64 / 8.0;
                let z = 0.02 + 0.13 * (u * surface.angle).cos();
                UpperGuide {
                    xy: [0.2 * (u * surface.angle).sin(), 1.5 - 0.1 * u],
                    target: z,
                    ceiling: Some(z + 0.015),
                    normal: None,
                }
            })
            .collect::<Vec<_>>();
        let mut outline = guides.iter().map(|g| g.xy).collect::<Vec<_>>();
        outline.extend([
            [0.2 * surface.angle.sin(), 1.3],
            [-0.2 * surface.angle.sin(), 1.3],
        ]);
        outline.extend(guides.iter().rev().map(|g| [-g.xy[0], g.xy[1]]));
        let body = |x: f64, _: f64| {
            (x.abs() < 0.18).then(|| 0.02 + 0.10 * (1.0 - (x / 0.18).powi(2)).sqrt())
        };
        let fitted = surface
            .fit_with_policy(
                &guides,
                &outline,
                0.002,
                fairing,
                body,
                FitPolicy {
                    minimum_bending_radius: false,
                    dense_guides: false,
                    elliptical_depth: true,
                },
            )
            .unwrap();
        assert_eq!(fitted.depth_controls.len(), 8);
        for guide in &guides {
            let p = fitted
                .evaluate(fitted.parameter(guide.xy).unwrap(), guide.xy[1])
                .0;
            assert!(p[2] <= guide.ceiling.unwrap() + 1e-8);
            if let Some(b) = body(p[0], p[1]) {
                assert!(p[2] >= b + 0.002 - 1e-8);
            }
        }
        for u in [0.0, 0.3, 0.6, 0.9] {
            let p = fitted.evaluate(u, fitted.seam_y).0;
            assert!((p[2] - 0.02 - 0.13 * (u * fitted.angle).cos()).abs() < 1e-14);
        }
        assert!(
            fitted
                .diagnostic_json()
                .contains("global-upper-elliptical-v1")
        );
    }
}

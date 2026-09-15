//! Authored outline contracts, including the actual catalog control endpoints.
use super::*;
use serde_json::json;

pub(super) fn endpoint(id: &str, endpoint: Option<&str>) -> Recipe {
    let preset = crate::authoring::authoring_catalog()["presets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap();
    let mut definition = preset["definition"].clone();
    if let Some(endpoint) = endpoint {
        for control in preset["controls"].as_array().unwrap() {
            let paths = control["paths"]
                .as_array()
                .cloned()
                .unwrap_or_else(|| vec![control["path"].clone()]);
            for path in paths {
                let pointer = format!("/{}", path.as_str().unwrap().replace('.', "/"));
                *definition.pointer_mut(&pointer).unwrap() = control[endpoint].clone();
            }
        }
    }
    serde_json::from_value(definition).unwrap()
}

fn separation(a: PlanarPoint, b: PlanarPoint) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[test]
fn glaive_has_one_acute_apex_and_continuous_asymmetric_shoulders() {
    let mut p: GlaiveParameters = serde_json::from_value(
        json!({"length":0.54,"width":0.105,"thickness":0.01,"curvature":0.13,"root":0.032}),
    )
    .unwrap();
    let points = blades::glaive_outline(&p, Detail::Medium);
    let apex: Vec<_> = points
        .iter()
        .enumerate()
        .filter(|(_, p)| (p[1] - 0.54).abs() < 1e-9)
        .collect();
    assert_eq!(apex.len(), 1);
    let i = apex[0].0;
    let a = [
        points[i - 1][0] - points[i][0],
        points[i - 1][1] - points[i][1],
    ];
    let b = [
        points[i + 1][0] - points[i][0],
        points[i + 1][1] - points[i][1],
    ];
    assert!(points[i - 1][1] >= 0.54 * 0.74 && points[i + 1][1] >= 0.54 * 0.74);
    assert!(
        ((a[0] * b[0] + a[1] * b[1]) / (a[0].hypot(a[1]) * b[0].hypot(b[1])))
            .acos()
            .to_degrees()
            < 35.0
    );
    p.curvature = Some(Metres::new(0.07).unwrap());
    let points = blades::glaive_outline(&p, Detail::Medium);
    let roots: Vec<_> = points.iter().filter(|p| p[1].abs() < 1e-9).collect();
    assert_eq!(roots.len(), 2);
    assert!(roots.iter().all(|p| (p[0].abs() - 0.016).abs() < 1e-9));
    let belly: Vec<_> = points
        .iter()
        .filter(|p| p[1] > 0.15 && p[1] < 0.3)
        .collect();
    assert!(belly.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max) > 0.07);
    assert!(belly.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min) > -0.04);
    assert!(points.windows(2).all(|p| separation(p[0], p[1]) > 1e-8));
}

#[test]
fn glaive_endpoint_boundaries_preserve_density_and_single_apex() {
    for endpoint_name in [None, Some("min"), Some("max")] {
        let recipe = endpoint("glaive", endpoint_name);
        let p = recipe
            .components
            .iter()
            .find_map(|c| {
                if let Shape::Glaive(p) = &c.shape {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let points = blades::glaive_outline(p, detail);
            assert_eq!(
                points
                    .iter()
                    .filter(|q| (q[1] - p.length.get()).abs() < 1e-9)
                    .count(),
                1
            );
            let body: Vec<_> = points.into_iter().filter(|p| p[1] >= 0.0).collect();
            assert!(body.windows(2).all(|q| {
                let chord = separation(q[0], q[1]);
                chord > 1e-8 && chord <= detail.error(p.length.get() / 28.0) * 1.01
            }));
        }
    }
}

#[test]
fn bill_hook_sampling_and_cubic_joins_survive_catalog_endpoints() {
    for endpoint_name in [None, Some("min"), Some("max")] {
        let recipe = endpoint("hooked-bill", endpoint_name);
        let p = recipe
            .components
            .iter()
            .find_map(|c| {
                if let Shape::Bill(p) = &c.shape {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        let spans = polls::bill_spans(p);
        for i in [1, 2, 4] {
            assert_eq!(spans[i][3], spans[i + 1][0]);
            let a = [
                spans[i][3][0] - spans[i][2][0],
                spans[i][3][1] - spans[i][2][1],
            ];
            let b = [
                spans[i + 1][1][0] - spans[i + 1][0][0],
                spans[i + 1][1][1] - spans[i + 1][0][1],
            ];
            assert!(a[0] * b[0] + a[1] * b[1] > 0.0);
            assert!(separation(a, b) < 1e-10);
        }
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let points = polls::bill_outline(p, detail);
            assert_eq!(
                points
                    .iter()
                    .filter(|q| (q[1] - p.length.get()).abs() < 1e-9)
                    .count(),
                1
            );
            assert!(points.iter().filter(|q| q[0] >= p.width.get()).count() >= 12);
            assert!(
                points
                    .iter()
                    .map(|q| q[0])
                    .fold(f64::NEG_INFINITY, f64::max)
                    >= p.width.get() + p.hook.get() * 0.99
            );
            assert!(points.windows(2).all(|q| separation(q[0], q[1]) > 1e-8));
            for span in spans {
                let quality = CurveQuality {
                    minimum_segments: 3,
                    max_chord: p.length.get() / 28.0,
                    max_deviation: p.width.get().min(p.hook.get()) / 90.0,
                };
                let samples = cubic_bezier(span, quality, detail);
                assert!(
                    samples
                        .windows(2)
                        .all(|q| separation(q[0], q[1]) <= detail.error(quality.max_chord) * 1.001)
                );
                for i in 0..=256 {
                    let t = i as f64 / 256.0;
                    let u = 1.0 - t;
                    let q = std::array::from_fn(|a| {
                        u * u * u * span[0][a]
                            + 3.0 * u * u * t * span[1][a]
                            + 3.0 * u * t * t * span[2][a]
                            + t * t * t * span[3][a]
                    });
                    let distance = samples
                        .windows(2)
                        .map(|s| {
                            let d = [s[1][0] - s[0][0], s[1][1] - s[0][1]];
                            let v = (((q[0] - s[0][0]) * d[0] + (q[1] - s[0][1]) * d[1])
                                / (d[0] * d[0] + d[1] * d[1]))
                                .clamp(0.0, 1.0);
                            separation(q, [s[0][0] + v * d[0], s[0][1] + v * d[1]])
                        })
                        .fold(f64::INFINITY, f64::min);
                    assert!(distance <= detail.error(quality.max_deviation) * 1.08);
                }
            }
        }
    }
}

#[test]
fn mace_flange_concavity_increases_without_duplicate_cusps() {
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let mut deviations = Vec::new();
        for concavity in [0.15, 0.5, 0.92] {
            let p:MaceParameters=serde_json::from_value(json!({"length":0.25,"rootRadius":0.009,"shoulderRadius":0.0065,"cuspRadius":0.06,"cuspHeight":0.75,"profileSamples":10,"concavity":concavity,"flanges":6,"flangeThickness":0.004})).unwrap();
            let outline = maces::flange_outer(&p, detail);
            assert!(outline.len() > detail.samples(10, 1) * 2);
            let cusp = outline
                .iter()
                .position(|p| (p[0] - 0.06).abs() < 1e-9)
                .unwrap();
            assert_eq!(
                outline
                    .iter()
                    .filter(|p| (p[0] - 0.06).abs() < 1e-9)
                    .count(),
                1
            );
            let target = (-0.125 + 0.0625) / 2.0;
            let midpoint = outline[..=cusp]
                .iter()
                .min_by(|a, b| (a[1] - target).abs().total_cmp(&(b[1] - target).abs()))
                .unwrap();
            deviations.push(0.009 + (0.06 - 0.009) * (midpoint[1] + 0.125) / 0.1875 - midpoint[0]);
        }
        assert!(
            deviations[0] > 0.0
                && deviations[1] > deviations[0] * 1.5
                && deviations[2] > deviations[1]
        );
    }
}

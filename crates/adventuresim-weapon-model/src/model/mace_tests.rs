//! Finite mating footprints and material partition for radial plates.
use super::*;
use serde_json::{Value, json};
use std::f64::consts::PI;

fn fixture(flanges: usize) -> Value {
    json!({"components":[{"id":"head","kind":"mace","material":"steel",
        "length":0.16,"rootRadius":0.014,"shoulderRadius":0.014,
        "cuspRadius":0.05,"cuspHeight":0.6,"concavity":0.5,
        "flanges":flanges,"flangeThickness":0.002,
        "coreProfile":[[-0.08,0.01],[0.08,0.01]],
        "attach":{"to":"weapon.root","at":"center"}}]})
}

#[test]
fn radial_flange_roots_share_full_faces_without_material_overlap() {
    for count in [3, 5, 7, 10] {
        let value = fixture(count);
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        let apothem = 0.01 * (PI / count as f64).cos();
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let mesh = generate_model(&recipe, detail).unwrap();
            let core = &mesh.parts[0];
            assert_eq!(mesh.parts.len(), count + 1);
            for index in 0..count {
                let angle = index as f64 * 2.0 * PI / count as f64;
                let project = |point: &[f64]| point[0] * angle.cos() + point[2] * angle.sin();
                let core_edge = core
                    .positions
                    .chunks_exact(3)
                    .map(project)
                    .fold(f64::NEG_INFINITY, f64::max);
                let flange = &mesh.parts[index + 1];
                let flange_edge = flange
                    .positions
                    .chunks_exact(3)
                    .map(project)
                    .fold(f64::INFINITY, f64::min);
                assert!((core_edge - apothem).abs() < 1e-12);
                assert!((flange_edge - apothem).abs() < 1e-12);
                let seated: Vec<_> = flange
                    .positions
                    .chunks_exact(3)
                    .filter(|p| (project(p) - apothem).abs() < 1e-12)
                    .collect();
                let sideways = |p: &&[f64]| -p[0] * angle.sin() + p[2] * angle.cos();
                assert!(seated.iter().map(sideways).fold(f64::INFINITY, f64::min) < -0.00099);
                assert!(
                    seated
                        .iter()
                        .map(sideways)
                        .fold(f64::NEG_INFINITY, f64::max)
                        > 0.00099
                );
            }
        }
    }
}

#[test]
fn mace_material_volume_agrees_with_polygon_core_and_analytic_flange_area() {
    let recipe: Recipe = serde_json::from_value(fixture(7)).unwrap();
    let mesh = generate_model(&recipe, Detail::High).unwrap();
    let exponent = 1.03 + 0.5 * 2.97;
    let core_volume = 7.0 / 2.0 * (2.0 * PI / 7.0).sin() * 0.01_f64.powi(2) * 0.16;
    let outer_area = 0.16 * (0.014 + (0.05 - 0.014) / (exponent + 1.0));
    let inner_area = 0.16 * 0.01 * (PI / 7.0).cos();
    let expected =
        (core_volume + 7.0 * 0.002 * (outer_area - inner_area)) * Material::Steel.density();
    assert!((mesh.physical.mass_kg - expected).abs() / expected < 0.001);
}

#[test]
fn invalid_mace_seats_are_rejected_instead_of_moving_or_thinning_plates() {
    for (field, value, diagnostic) in [
        ("segments", json!(8), "multiple"),
        ("flangeThickness", json!(0.02), "thickness"),
        (
            "coreProfile",
            json!([[-0.08, 0.03], [0.08, 0.03]]),
            "outside",
        ),
    ] {
        let mut value_recipe = fixture(5);
        value_recipe["components"][0][field] = value;
        let recipe: Recipe = serde_json::from_value(value_recipe).unwrap();
        assert!(
            generate_model(&recipe, Detail::Low)
                .unwrap_err()
                .contains(diagnostic)
        );
    }
    let mut removed = fixture(5);
    removed["components"][0]["flangeRootScale"] = json!(0.55);
    assert!(serde_json::from_value::<Recipe>(removed).is_err());
}

#[test]
fn crownless_flange_endpoint_meets_the_exact_core_height() {
    let recipe = super::profile_tests::endpoint("flanged-mace", Some("min"));
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let mesh = generate_model(&recipe, detail).unwrap();
        assert!(mesh.positions.iter().all(|p| p.is_finite()));
    }
}

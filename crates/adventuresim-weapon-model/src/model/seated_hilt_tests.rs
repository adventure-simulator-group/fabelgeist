use super::longsword_tests::closed;
use super::*;

#[test]
fn mounted_blade_rejects_section_engagement_and_pose_mismatches() {
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../../review/museum/met-14.25.1096.json")).unwrap();
    let recipe: Recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    placement::resolve(&recipe).unwrap();
    for (field, value) in [
        ("width", 0.043),
        ("thickness", 0.005),
        ("ricasso", 0.001),
        ("singleEdge", 0.2),
    ] {
        let mut value_recipe = study["definition"].clone();
        value_recipe["components"][3][field] = value.into();
        let invalid: Recipe = serde_json::from_value(value_recipe).unwrap();
        assert!(placement::resolve(&invalid).is_err());
    }
    for field in ["rotation", "offset"] {
        let mut value_recipe = study["definition"].clone();
        if field == "rotation" {
            value_recipe["components"][3][field] = serde_json::json!([0, 1, 0]);
        } else {
            value_recipe["components"][3]["attach"][field] = serde_json::json!([0.001, 0, 0]);
        }
        let invalid: Recipe = serde_json::from_value(value_recipe).unwrap();
        assert!(placement::resolve(&invalid).is_err());
    }
    for (field, value) in [("curvature", 0.01), ("belly", 0.2)] {
        let mut value_recipe = study["definition"].clone();
        value_recipe["components"][3][field] = value.into();
        let valid: Recipe = serde_json::from_value(value_recipe).unwrap();
        placement::resolve(&valid).unwrap();
    }
}

#[test]
fn sub_resolution_receiving_cuts_fail_without_silent_repair() {
    for seat in [0.023 - 1e-10, 0.023 + 1e-10] {
        let p: WheelPommelParameters = serde_json::from_value(serde_json::json!({
            "diameter":0.062,"thickness":0.020,"faceDiameter":0.046,
            "rimThickness":0.012,"seatHeight":seat
        }))
        .unwrap();
        let mut rejected = false;
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            match wheel::construct(&p, detail) {
                Ok(solid) => closed(&solid),
                Err(_) => rejected = true,
            }
        }
        assert!(rejected);
    }
}

#[test]
fn wheel_receiving_cuts_retain_closed_float32_boundaries() {
    for (diameter, face, seat) in [
        (0.062, 0.046, 0.015),
        (0.062, 0.046, 0.023),
        (0.062, 0.046, 0.027),
        (0.080, 0.056, 0.020),
    ] {
        let p: WheelPommelParameters = serde_json::from_value(serde_json::json!({
            "diameter":diameter,"thickness":0.020,"faceDiameter":face,
            "rimThickness":0.012,"seatHeight":seat
        }))
        .unwrap();
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = wheel::construct(&p, detail).unwrap();
            closed(&solid);
            assert!(solid.positions.iter().all(|v| v[1] <= seat));
            assert!(
                solid
                    .faces
                    .iter()
                    .any(|f| f.iter().all(|&i| solid.positions[i][1] == seat))
            );
        }
    }
}

#[test]
fn mortise_is_open_and_closed_material_surrounds_its_seat() {
    let p: MortisedGuardParameters = serde_json::from_value(serde_json::json!({
        "width":0.17,"height":0.012,"thickness":0.024,"sweep":0.025,
        "shoulderHeight":0.008,"edgeBevel":0.001,"terminalScale":1.2,
        "mortise":{"width":0.05,"thickness":0.006,"bevelWidthRatio":0.2}
    }))
    .unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = mortised_guard::construct(&p, detail).unwrap();
        closed(&solid);
        for point in &solid.positions {
            if point[0].abs() < 0.02 && point[1] > 0.012 {
                assert!(point[2].abs() >= 0.003);
            }
        }
        for sign in [-1.0, 1.0] {
            assert!(solid.faces.iter().any(|f| f.iter().all(|&i| {
                let v = solid.positions[i];
                v[0].abs() <= 0.02 && v[1] >= 0.012 && v[2] == sign * 0.003
            })));
        }
    }
}

fn ribbed_definition() -> serde_json::Value {
    serde_json::json!({"components":[{"id":"grip","kind":"profileGrip",
        "material":"wood","length":0.12,
        "profile":[{"at":0,"width":0.024,"depth":0.016},
            {"at":0.5,"width":0.034,"depth":0.024},
            {"at":1,"width":0.028,"depth":0.020}],
        "ribs":{"count":16,"depth":0.001},
        "cover":{"material":"darkLeather","thickness":0.0008},
        "attach":{"to":"weapon.root","at":"base"}}]})
}

#[test]
fn ribbed_core_and_cover_partition_the_same_crest_envelope() {
    let recipe: Recipe = serde_json::from_value(ribbed_definition()).unwrap();
    recipe.validate().unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let resolved = placement::resolve(&recipe).unwrap();
        let parts = shapes::construct(&resolved.components[0], detail).unwrap();
        for part in &parts {
            closed(&part.solid);
        }
        let mut whole = recipe.clone();
        let Shape::ProfileGrip(p) = &mut whole.components[0].shape else {
            unreachable!()
        };
        p.cover = None;
        let resolved = placement::resolve(&whole).unwrap();
        let solids = shapes::construct(&resolved.components[0], detail).unwrap();
        let volume = parts.iter().map(|p| p.solid.volume()).sum::<f64>();
        assert!((volume - solids[0].solid.volume()).abs() < 1e-10);
        let Shape::ProfileGrip(p) = &mut whole.components[0].shape else {
            unreachable!()
        };
        p.ribs = None;
        let resolved = placement::resolve(&whole).unwrap();
        let smooth = shapes::construct(&resolved.components[0], detail).unwrap();
        assert!(volume < smooth[0].solid.volume());
    }
}

#[test]
fn ribs_reject_collapsed_cores_and_anatomically_oversized_crests() {
    for (field, value) in [
        ("count", 0.0),
        ("count", 65.0),
        ("depth", 0.0),
        ("depth", 0.003),
    ] {
        let mut fixture = ribbed_definition();
        fixture["components"][0]["ribs"][field] = if field == "count" {
            serde_json::json!(value as u16)
        } else {
            value.into()
        };
        let recipe: Recipe = serde_json::from_value(fixture).unwrap();
        assert!(recipe.validate().is_err());
    }
    let mut fixture = ribbed_definition();
    fixture["components"][0]["profile"][1]["depth"] = 0.03.into();
    assert!(
        serde_json::from_value::<Recipe>(fixture)
            .unwrap()
            .validate()
            .is_err()
    );
}

use super::longsword_tests::{blade_definition, closed};
use super::*;

#[test]
fn separated_axial_grooves_and_opposite_face_shoulders_remain_closed() {
    for same_lane in [false, true] {
        let mut value = blade_definition();
        let fuller = &mut value["components"][0]["fuller"];
        let mut first = fuller["grooves"][0].clone();
        first["mouthWidth"] = 0.012.into();
        first["end"] = 0.1.into();
        first["exitLength"] = 0.025.into();
        first["lateralPosition"] = (-0.25).into();
        let mut second = first.clone();
        if same_lane {
            second["start"] = 0.15.into();
            second["end"] = 0.25.into();
        } else {
            first["faces"] = "front".into();
            second["faces"] = "back".into();
            first["depth"] = 0.0036.into();
            second["depth"] = 0.0036.into();
            second["lateralPosition"] = 0.10.into();
        }
        fuller["grooves"] = serde_json::json!([first, second]);
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        recipe.validate().unwrap();
        let Shape::LoftedBlade(p) = &recipe.components[0].shape else {
            unreachable!()
        };
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = lofted_blade::blade(p, detail).unwrap();
            closed(&solid);
            closed(&solid.transform([23.0, 47.0, 61.0], [19.0, -18.0, 17.0]));
        }
    }
}

#[test]
fn coincident_active_lanes_and_opposing_through_cuts_reject() {
    for opposite in [false, true] {
        let mut value = blade_definition();
        let f = &mut value["components"][0]["fuller"];
        let mut a = f["grooves"][0].clone();
        let mut b = a.clone();
        if opposite {
            a["faces"] = "front".into();
            b["faces"] = "back".into();
            a["depth"] = 0.004.into();
            b["depth"] = 0.004.into();
        }
        f["grooves"] = serde_json::json!([a, b]);
        assert!(
            serde_json::from_value::<Recipe>(value)
                .unwrap()
                .validate()
                .is_err()
        );
    }
}

#[test]
fn capped_profile_body_partitions_material_and_retains_grip_limits() {
    let value = serde_json::json!({"components":[{"id":"body","kind":"profileBody","material":"wood","attach":{"to":"weapon.root","at":"base"},"length":0.08,
        "profile":[{"at":0,"width":0.07,"depth":0.026},{"at":1,"width":0.07,"depth":0.026}],
        "cover":{"material":"brass","thickness":0.0007,"endCap":0.002}}]});
    let recipe: Recipe = serde_json::from_value(value.clone()).unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let resolved = placement::resolve(&recipe).unwrap();
        let parts = shapes::construct(&resolved.components[0], detail).unwrap();
        for part in &parts {
            closed(&part.solid);
        }
        let core_end = parts[0]
            .solid
            .positions
            .iter()
            .map(|p| p[1])
            .fold(0.0, f64::max);
        assert!((core_end - 0.078).abs() < 1e-14);
        let mut homogeneous = recipe.clone();
        let Shape::ProfileBody(p) = &mut homogeneous.components[0].shape else {
            unreachable!()
        };
        p.cover = None;
        let resolved = placement::resolve(&homogeneous).unwrap();
        let whole = shapes::construct(&resolved.components[0], detail).unwrap();
        assert!(
            (parts.iter().map(|p| p.solid.volume()).sum::<f64>() - whole[0].solid.volume()).abs()
                < 1e-12
        );
        assert_eq!(parts[0].material, Material::Wood);
        assert_eq!(parts[1].material, Material::Brass);
    }
    let mut grip = value;
    grip["components"][0]["kind"] = "profileGrip".into();
    assert_eq!(
        serde_json::from_value::<Recipe>(grip).unwrap().validate(),
        Err(RecipeError::Grip)
    );
}

#[test]
fn narrow_deep_grooves_report_precision_limits_without_relaxing_clearance() {
    let mut value = blade_definition();
    let groove = &mut value["components"][0]["fuller"]["grooves"][0];
    groove["faces"] = "front".into();
    groove["mouthWidth"] = 0.0035.into();
    groove["depth"] = 0.0045.into();
    groove["floorWidthRatio"] = 0.4.into();
    groove["end"] = 0.075.into();
    groove["entryLength"] = 0.004.into();
    groove["exitLength"] = 0.015.into();
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    recipe.validate().unwrap();
    assert!(
        generate_model(&recipe, Detail::High)
            .unwrap_err()
            .contains("normal-error budget")
    );
}

#[test]
fn shared_profile_sampling_matches_every_material_seat_at_each_lod() {
    for count in [12, 28, 48] {
        let value = serde_json::json!({"components":[
            {"id":"body","kind":"profileBody","length":0.07,"radialSegments":count,"material":"wood","profile":[{"at":0,"width":0.07,"depth":0.024},{"at":1,"width":0.028,"depth":0.02}],"cover":{"material":"brass","thickness":0.0007},"attach":{"to":"weapon.root","at":"base"}},
            {"id":"grip","kind":"profileGrip","length":0.08,"radialSegments":count,"material":"wood","profile":[{"at":0,"width":0.028,"depth":0.02},{"at":1,"width":0.026,"depth":0.018}],"cover":{"material":"brass","thickness":0.0007},"attach":{"to":"body.top","at":"base"}}
        ]});
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        recipe.validate().unwrap();
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let resolved = placement::resolve(&recipe).unwrap();
            let body = shapes::construct(&resolved.components[0], detail).unwrap();
            let grip = shapes::construct(&resolved.components[1], detail).unwrap();
            let boundary = |part: &PartSource, y: f64| {
                part.solid
                    .positions
                    .iter()
                    .filter(|p| p[1] == y)
                    .map(|p| [p[0].to_bits(), p[2].to_bits()])
                    .collect::<std::collections::BTreeSet<_>>()
            };
            for material in 0..2 {
                assert_eq!(
                    boundary(&body[material], 0.07),
                    boundary(&grip[material], 0.0)
                );
                closed(&body[material].solid);
                closed(&grip[material].solid);
            }
        }
    }
}

#[test]
fn paired_relief_retains_closed_faces_at_the_float32_tail_cutoff() {
    for (length, entry) in [(0.438, 0.04), (0.52, 0.07)] {
        let value = serde_json::json!({"components":[{
            "id":"blade","kind":"loftedBlade","length":length,"width":0.034,"thickness":0.012,
            "attach":{"to":"weapon.root","at":"base"},
            "curvature":0,"plan":"straight","section":"recessed","samples":40,
            "taper":0.45,"singleEdge":0,"belly":0,"ricasso":0,
            "point":{"start":0.91,"roundness":0.8},
            "fuller":{"bevelWidthRatio":0.25,"grooves":[
                {"faces":"both","lateralPosition":-0.5,"mouthWidth":0.0035,"depth":0.001,
                "floorWidthRatio":0.3,"start":0,"end":0.4,"entryLength":entry,"exitLength":0.06},
                {"faces":"both","lateralPosition":0.5,"mouthWidth":0.0035,"depth":0.001,
                "floorWidthRatio":0.3,"start":0,"end":0.4,"entryLength":entry,"exitLength":0.06}
            ]}
        }]});
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        recipe.validate().unwrap();
        let Shape::LoftedBlade(p) = &recipe.components[0].shape else {
            unreachable!()
        };
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = lofted_blade::blade(p, detail).unwrap();
            closed(&solid);
            closed(&solid.transform([23.0, 47.0, 61.0], [19.0, -18.0, 17.0]));
        }
    }
}

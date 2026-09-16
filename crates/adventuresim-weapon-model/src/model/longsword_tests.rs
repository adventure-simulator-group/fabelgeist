use super::*;
use std::collections::BTreeMap;

fn blade_definition() -> serde_json::Value {
    serde_json::json!({"components":[{
        "attach":{"to":"weapon.root","at":"origin"},"id":"blade","kind":"loftedBlade","length":0.902,"width":0.047,"thickness":0.007,
        "curvature":0,"plan":"straight","section":"recessed","samples":48,"taper":0.2,"singleEdge":0,"belly":0,"ricasso":0,
        "point":{"start":0.90,"roundness":0.6},
        "fuller":{"faces":"both","mouthWidth":0.012,"depth":0.001,"floorWidthRatio":0.45,"bevelWidthRatio":0.2,
            "start":0.0,"end":0.46,"entryLength":0.012,"exitLength":0.055}
    }]})
}

fn closed(solid: &Solid) {
    for f32_output in [false, true] {
        let key = |p: Point| {
            p.map(|v| {
                let x = if f32_output { v as f32 as f64 } else { v };
                if x == 0.0 { 0 } else { x.to_bits() }
            })
        };
        let mut edges = BTreeMap::<_, Vec<_>>::new();
        for face in &solid.faces {
            let points = face
                .map(|i| solid.positions[i].map(|v| if f32_output { v as f32 as f64 } else { v }));
            assert!(points.iter().flatten().all(|v| v.is_finite()));
            assert!(
                magnitude(cross(sub(points[1], points[0]), sub(points[2], points[0]))) > 0.0,
                "degenerate {points:?}"
            );
            for [a, b] in [
                [points[0], points[1]],
                [points[1], points[2]],
                [points[2], points[0]],
            ] {
                let (a, b) = (key(a), key(b));
                edges
                    .entry(if a < b { [a, b] } else { [b, a] })
                    .or_default()
                    .push(a < b);
            }
        }
        assert!(
            edges
                .values()
                .all(|uses| uses.len() == 2 && uses[0] != uses[1]),
            "nonmanifold float32={f32_output}"
        );
    }
    assert!(solid.volume() > 0.0);
}

#[test]
fn recessed_fuller_and_rounded_point_are_closed_at_all_lods() {
    for faces in ["front", "back", "both"] {
        for roundness in [0.0, 0.5, 1.0] {
            let mut value = blade_definition();
            value["components"][0]["fuller"]["faces"] = faces.into();
            value["components"][0]["point"]["roundness"] = roundness.into();
            let recipe: Recipe = serde_json::from_value(value).unwrap();
            recipe.validate().unwrap();
            let Shape::LoftedBlade(p) = &recipe.components[0].shape else {
                unreachable!()
            };
            for detail in [Detail::Low, Detail::Medium, Detail::High] {
                closed(&lofted_blade::blade(p, detail).unwrap());
            }
        }
    }
}

#[test]
fn invalid_fuller_clearance_and_transition_reject_before_generation() {
    for (field, value) in [
        ("depth", 0.004),
        ("mouthWidth", 0.05),
        ("exitLength", 0.0),
        ("floorWidthRatio", 1.0),
    ] {
        let mut recipe = blade_definition();
        recipe["components"][0]["fuller"][field] = value.into();
        assert_eq!(
            serde_json::from_value::<Recipe>(recipe).unwrap().validate(),
            Err(if field == "exitLength" {
                RecipeError::Dimension
            } else {
                RecipeError::Proportion
            }),
            "{field}"
        );
    }
    serde_json::from_value::<Recipe>(blade_definition())
        .unwrap()
        .validate()
        .unwrap();
}

#[test]
fn unresolvable_fuller_floor_rejects_instead_of_emitting_collapsed_strips() {
    let mut value = blade_definition();
    value["components"][0]["fuller"]["floorWidthRatio"] = 0.000001.into();
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    recipe.validate().unwrap();
    assert!(
        generate_model(&recipe, Detail::High)
            .unwrap_err()
            .contains("fuller strips cannot be resolved")
    );
    generate_model(
        &serde_json::from_value(blade_definition()).unwrap(),
        Detail::High,
    )
    .unwrap();
}

#[test]
fn long_fuller_closure_stays_resolved_through_a_rounded_point() {
    let mut value = blade_definition();
    value["components"][0]["fuller"]["end"] = 0.865.into();
    value["components"][0]["fuller"]["mouthWidth"] = 0.008.into();
    value["components"][0]["fuller"]["depth"] = 0.00045.into();
    value["components"][0]["fuller"]["start"] = 0.015.into();
    value["components"][0]["fuller"]["entryLength"] = 0.025.into();
    value["components"][0]["fuller"]["exitLength"] = 0.18.into();
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    let Shape::LoftedBlade(p) = &recipe.components[0].shape else {
        unreachable!()
    };
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = lofted_blade::blade(p, detail).unwrap();
        closed(&solid);
        closed(&solid.clone().transform([0.0; 3], [0.31, 1.175, -0.42]));
    }
}

#[test]
fn fuller_can_end_at_the_true_tip_without_losing_interior_metal() {
    for point in [None, Some(0.0), Some(1.0)] {
        let mut value = blade_definition();
        let blade = &mut value["components"][0];
        blade["fuller"]["end"] = 0.902.into();
        blade["fuller"]["exitLength"] = 0.22.into();
        if let Some(roundness) = point {
            blade["point"]["roundness"] = roundness.into();
        } else {
            blade.as_object_mut().unwrap().remove("point");
        }
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        recipe.validate().unwrap();
        let Shape::LoftedBlade(p) = &recipe.components[0].shape else {
            unreachable!()
        };
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = lofted_blade::blade(p, detail).unwrap();
            closed(&solid);
            closed(&solid.clone().transform([0.0; 3], [0.31, 1.175, -0.42]));
            closed(&solid.transform([23.0, 47.0, 61.0], [19.0, -18.0, 17.0]));
        }
    }
}

#[test]
fn section_blade_shares_recessed_point_geometry_with_straight_loft() {
    let recipe: Recipe = serde_json::from_value(blade_definition()).unwrap();
    let Shape::LoftedBlade(loft) = &recipe.components[0].shape else {
        unreachable!()
    };
    let section = SectionBladeParameters {
        length: loft.length,
        width: loft.width,
        thickness: loft.thickness,
        taper: Some(loft.taper),
        section: Some(loft.section),
        fuller: loft.fuller.clone(),
        point: loft.point.clone(),
    };
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let a = lofted_blade::blade(loft, detail).unwrap();
        let b = blades::section_blade(&section, detail).unwrap();
        assert_eq!(a.positions, b.positions);
        assert_eq!(a.faces, b.faces);
        closed(&b);
    }
}

#[test]
fn shallow_fuller_without_a_point_has_bounded_sampling_and_plane_normals() {
    let mut value = blade_definition();
    let blade = &mut value["components"][0];
    blade["fuller"] = serde_json::json!({"faces":"both","mouthWidth":0.014,"depth":0.0005,"floorWidthRatio":0.45,
        "bevelWidthRatio":0.2,"start":0.015,"end":0.285,"entryLength":0.025,"exitLength":0.08});
    let mut section = blade.clone();
    section["kind"] = "sectionBlade".into();
    for field in [
        "point",
        "curvature",
        "plan",
        "samples",
        "singleEdge",
        "belly",
        "ricasso",
    ] {
        section.as_object_mut().unwrap().remove(field);
    }
    let p: SectionBladeParameters = serde_json::from_value({
        let mut v = section.clone();
        for f in ["kind", "id", "attach"] {
            v.as_object_mut().unwrap().remove(f);
        }
        v
    })
    .unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = blades::section_blade(&p, detail).unwrap();
        assert!(solid.faces.len() < 20000);
        closed(&solid);
    }
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    let model = generate_model(&recipe, Detail::High).unwrap();
    let part = &model.parts[0];
    // At the mouth of the plateau the land is flat and the groove wall has a
    // distinct geometric normal; shallow corners must not be averaged away.
    let normals: Vec<_> = part
        .positions
        .as_chunks::<3>()
        .0
        .iter()
        .zip(part.normals.as_chunks::<3>().0.iter())
        .filter(|(p, _)| (p[0] - 0.007).abs() < 1e-10 && p[1] > 0.05 && p[1] < 0.15)
        .map(|(_, n)| n[0])
        .collect();
    assert!(normals.iter().any(|n| n.abs() < 0.001));
    assert!(normals.iter().any(|n| n.abs() > 0.10));
}

#[test]
fn flat_guard_block_covers_the_complete_profile_grip_end_without_overlap() {
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../../review/museum/cma-1921.1253.json")).unwrap();
    let recipe: Recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let resolved = placement::resolve(&recipe).unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let guard = &resolved.components[2];
        let parts = shapes::construct(guard, detail).unwrap();
        let block = &parts.last().unwrap().solid;
        closed(block);
        let bottom = block
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min);
        let points: Vec<_> = block
            .positions
            .iter()
            .filter(|p| (p[1] - bottom).abs() < 1e-12)
            .collect();
        let x = points.iter().map(|p| p[0].abs()).fold(0.0, f64::max);
        let z = points.iter().map(|p| p[2].abs()).fold(0.0, f64::max);
        let grip = &resolved.components[1];
        let sources = shapes::construct(grip, detail).unwrap();
        let top = sources
            .iter()
            .flat_map(|p| p.solid.positions.iter())
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((top + grip.offset[1] - bottom - guard.offset[1]).abs() < 1e-12);
        for point in sources
            .iter()
            .flat_map(|p| p.solid.positions.iter())
            .filter(|p| (p[1] - top).abs() < 1e-12)
        {
            assert!(point[0].abs() <= x + 1e-12 && point[2].abs() <= z + 1e-12);
        }
    }
}

#[test]
fn profiled_grip_preserves_authored_ends_and_partitions_material() {
    let recipe:Recipe=serde_json::from_value(serde_json::json!({"gripClearance":0.05,"components":[
        {"attach":{"to":"weapon.root","at":"origin"},"id":"grip","kind":"profileGrip","length":0.21,"material":"wood","profile":[
            {"at":0,"width":0.02,"depth":0.016},{"at":0.25,"width":0.033,"depth":0.024},
            {"at":0.52,"width":0.024,"depth":0.02},{"at":0.82,"width":0.033,"depth":0.024},
            {"at":1,"width":0.029,"depth":0.022}],"cover":{"material":"darkLeather","thickness":0.001}}
    ]})).unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let resolved = placement::resolve(&recipe).unwrap();
        let sources = shapes::construct(&resolved.components[0], detail).unwrap();
        assert_eq!(sources.len(), 2);
        for source in &sources {
            closed(&source.solid);
        }
        assert_eq!(sources[0].material, Material::Wood);
        assert_eq!(sources[1].material, Material::DarkLeather);
        let mut homogeneous = resolved.components[0].component.clone();
        if let Shape::ProfileGrip(p) = &mut homogeneous.shape {
            p.cover = None;
        }
        let single_recipe = Recipe {
            components: vec![homogeneous],
            ..recipe.clone()
        };
        let single = placement::resolve(&single_recipe).unwrap();
        let outer = shapes::construct(&single.components[0], detail).unwrap();
        assert!(
            (sources.iter().map(|p| p.solid.volume()).sum::<f64>() - outer[0].solid.volume()).abs()
                < 1e-12
        );
        let model = generate_model(&recipe, detail).unwrap();
        assert!((model.physical.control_point[1] - 0.16).abs() < 1e-12);
    }
}

#[test]
fn axial_guard_terminals_join_outside_the_arm_and_inherit_material() {
    let value = serde_json::json!({"components":[{"id":"guard","kind":"guard","attach":{"to":"weapon.root","at":"origin"},
        "material":"darkSteel","width":0.24,"height":0.02,"thickness":0.01,"section":"flat","sectionWidth":0.009,"sectionDepth":0.006,"tipScale":0.7,
        "terminal":"profile","terminalProfile":{"interpolation":"smooth","stations":[[0,0.005],[0.002,0.005],[0.002,0.006],[0.006,0.006],[0.01,0.004],[0.012,0.002]]}}]});
    let recipe: Recipe = serde_json::from_value(value.clone()).unwrap();
    recipe.validate().unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let resolved = placement::resolve(&recipe).unwrap();
        let parts = shapes::construct(&resolved.components[0], detail).unwrap();
        assert_eq!(parts.len(), 4);
        for part in &parts {
            closed(&part.solid);
            assert_eq!(part.material, Material::DarkSteel);
        }
        for (side, part) in [(-1.0, &parts[1]), (1.0, &parts[2])] {
            assert!(
                part.solid
                    .positions
                    .iter()
                    .all(|p| side * p[0] >= 0.12 - 1e-12)
            );
        }
    }
    let mut invalid = value;
    invalid["components"][0]["terminalProfile"]["stations"][0][1] = 0.0001.into();
    let invalid: Recipe = serde_json::from_value(invalid).unwrap();
    assert!(
        generate_model(&invalid, Detail::High)
            .unwrap_err()
            .contains("receiving quillon")
    );
}

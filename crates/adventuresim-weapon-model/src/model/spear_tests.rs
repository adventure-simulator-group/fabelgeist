use super::*;
use std::collections::BTreeMap;

fn fixture() -> serde_json::Value {
    serde_json::json!({"shaft":{"length":1.903,"radius":0.020,"topScale":0.965,
        "bottomScale":0.9,"tenon":{"length":0.075,"tipRadius":0.0125}},
        "components":[{"id":"head","kind":"spear","mount":"shaft-top",
            "length":0.382,"width":0.049,"thickness":0.008,"rootWidth":0.031,
            "bellyPosition":0.16230366492146597,"shoulderRoundness":1.0,
            "socket":{"length":0.12,"neckLength":0.035,"baseRadius":0.022,"wall":0.002,
                "cavityDepth":0.08,"insertionDepth":0.075,"boreTipRadius":0.013,
                "stops":{"span":0.095,"centerHeight":0.07,"endHeight":0.014,
                    "rootHeight":0.035,"thickness":0.008,"rootBlend":0.003,"orientation":0.0}}}]})
}

fn closed(solid: &Solid) {
    for float32 in [false, true] {
        let point = |index: usize| {
            solid.positions[index].map(|v| if float32 { v as f32 as f64 } else { v })
        };
        let key = |p: Point| p.map(|v| (v * 1e9).round() as i64);
        let mut edges = BTreeMap::<_, Vec<_>>::new();
        for &[a, b, c] in &solid.faces {
            let [a, b, c] = [point(a), point(b), point(c)];
            assert!(
                magnitude(cross(sub(b, a), sub(c, a))) > 1e-17,
                "degenerate {float32}: {a:?} {b:?} {c:?}"
            );
            for [a, b] in [[key(a), key(b)], [key(b), key(c)], [key(c), key(a)]] {
                assert_ne!(a, b, "collapsed edge");
                edges
                    .entry(if a < b { [a, b] } else { [b, a] })
                    .or_default()
                    .push(a < b);
            }
        }
        for (edge, directions) in edges {
            assert_eq!(directions.len(), 2, "edge {edge:?}, float32 {float32}");
            assert_ne!(directions[0], directions[1], "winding {edge:?}");
        }
    }
    assert!(solid.volume() > 0.0);
}

#[test]
fn socketed_leaf_and_integral_stops_are_closed_at_all_details() {
    for orientation in [0.0, 0.000001, 37.0, 89.999999, 90.0, 359.999999] {
        for stops in [true, false] {
            let mut definition = fixture();
            definition["components"][0]["socket"]["stops"]["orientation"] = orientation.into();
            if !stops {
                definition["components"][0]["socket"]
                    .as_object_mut()
                    .unwrap()
                    .remove("stops");
            }
            let recipe: Recipe = serde_json::from_value(definition).unwrap();
            recipe.validate().unwrap();
            let Shape::Spear(p) = &recipe.components[0].shape else {
                unreachable!()
            };
            for detail in [Detail::Low, Detail::Medium, Detail::High] {
                closed(
                    &spears::spear(p, detail)
                        .unwrap()
                        .transform([0.0; 3], [0.0, 1.948, 0.0]),
                );
            }
        }
    }
}

#[test]
fn shaft_wrappings_are_closed_tapered_strips_in_both_handednesses() {
    for pattern in ["leftHanded", "rightHanded", "crossed"] {
        let mut definition = fixture();
        definition["shaft"]["wrappings"] = serde_json::json!([{
            "start":1.67,"length":0.15,"pitch":0.08,"width":0.009,
            "thickness":0.0015,"phase":25,"pattern":pattern,"material":"leather"}]);
        let recipe: Recipe = serde_json::from_value(definition).unwrap();
        recipe.validate().unwrap();
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            for part in
                shaft_wrapping::parts(recipe.shaft.as_ref().unwrap(), "shaft", "shaft", detail)
                    .unwrap()
            {
                closed(&part.solid);
            }
        }
    }
}

#[test]
fn socket_material_is_excluded_from_the_working_blade_length() {
    let mut value = fixture();
    value["components"][0]["role"] = "Head".into();
    let mut design = crate::default_design("spear").unwrap();
    design.recipe = serde_json::from_value(value).unwrap();
    let native = crate::generate(&design).unwrap();
    let physical = crate::derive_properties(&design).unwrap();
    assert!((native.derived.striking_head_length_m - 0.382).abs() < 1e-6);
    assert_eq!(native.derived, physical);
}

#[test]
fn receiving_tenon_and_declared_insertion_share_the_same_frame() {
    let recipe: Recipe = serde_json::from_value(fixture()).unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let model = generate_model(&recipe, detail).unwrap();
        assert!((model.resolved_definition.frames["head.socketRim"][1] - 1.828).abs() < 1e-12);
        assert!((model.resolved_definition.frames["head.tip"][1] - 2.33).abs() < 1e-12);
    }
    let mut invalid = fixture();
    invalid["shaft"]["tenon"]["tipRadius"] = 0.018.into();
    let recipe = serde_json::from_value(invalid).unwrap();
    assert!(
        generate_model(&recipe, Detail::High)
            .unwrap_err()
            .contains("complete receiving shaft")
    );
}

#[test]
fn explicit_rotated_shaft_parent_enforces_the_same_socket_fit() {
    for bad_fit in [false, true] {
        let mut value = fixture();
        let mut shaft = value["shaft"].clone();
        if bad_fit {
            shaft["tenon"]["tipRadius"] = 0.018.into();
        }
        shaft["kind"] = "shaft".into();
        shaft["id"] = "receiver".into();
        shaft["rotation"] = serde_json::json!([0, 0, 90]);
        shaft["attach"] = serde_json::json!({"to":"weapon.root","at":"base"});
        let mut head = value["components"][0].clone();
        head.as_object_mut().unwrap().remove("mount");
        head["rotation"] = serde_json::json!([0, 0, 90]);
        head["attach"] =
            serde_json::json!({"to":"receiver.top","at":"origin","offset":[-0.045,0,0]});
        value.as_object_mut().unwrap().remove("shaft");
        value["components"] = serde_json::json!([shaft, head]);
        let result = generate_model(&serde_json::from_value(value).unwrap(), Detail::High);
        if bad_fit {
            assert!(result.unwrap_err().contains("complete receiving shaft"));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn museum_study_loads_through_the_same_control_validation_as_the_editor() {
    for study in crate::authoring::museum_studies().as_array().unwrap() {
        let recipe: Recipe = serde_json::from_value(study["definition"].clone()).unwrap();
        crate::authoring::validate_controls(&recipe, study["controls"].as_array().unwrap())
            .unwrap();
        generate_model(&recipe, Detail::High).unwrap();
    }
}

#[test]
fn rounded_leaves_keep_dimensional_landmarks_and_change_shoulder_tangents() {
    for roundness in [0.0, 0.5, 1.0] {
        let mut recipe = fixture();
        recipe["components"][0]["shoulderRoundness"] = roundness.into();
        let recipe: Recipe = serde_json::from_value(recipe).unwrap();
        let Shape::Spear(p) = &recipe.components[0].shape else {
            unreachable!()
        };
        assert!((spears::half_width(p, 0.0) - 0.0155).abs() < 1e-12);
        assert!((spears::half_width(p, p.belly_position.unwrap().get()) - 0.0245).abs() < 1e-12);
        assert_eq!(spears::half_width(p, 1.0), 0.0);
    }
}

#[test]
fn socketed_construction_generalizes_to_narrow_broad_and_scaled_heads() {
    for (width, root, belly, length, roundness, socket_length, shaft_length) in [
        (0.033, 0.027, 0.05, 0.46, 0.0, 0.12, 1.6),
        (0.063, 0.033, 0.38, 0.28, 1.0, 0.20, 2.2),
        (0.040, 0.030, 0.20, 0.31, 0.5, 0.16, 1.8),
    ] {
        let mut value = fixture();
        value["shaft"]["length"] = shaft_length.into();
        value["shaft"]["tenon"]["tipRadius"] = 0.0105.into();
        let head = &mut value["components"][0];
        head["width"] = width.into();
        head["rootWidth"] = root.into();
        head["bellyPosition"] = belly.into();
        head["length"] = length.into();
        head["shoulderRoundness"] = roundness.into();
        head["socket"]["length"] = socket_length.into();
        head["socket"]["boreTipRadius"] = 0.0115.into();
        head["socket"]["stops"]
            .as_object_mut()
            .unwrap()
            .remove("rootBlend");
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        let Shape::Spear(p) = &recipe.components[0].shape else {
            unreachable!()
        };
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            generate_model(&recipe, detail).unwrap();
            closed(&spears::spear(p, detail).unwrap());
        }
    }
    for section in ["flat", "diamond"] {
        let mut value = fixture();
        value["components"][0]
            .as_object_mut()
            .unwrap()
            .remove("socket");
        value["components"][0]["section"] = section.into();
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        let Shape::Spear(p) = &recipe.components[0].shape else {
            unreachable!()
        };
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            closed(&spears::spear(p, detail).unwrap());
        }
    }
}

fn radial_interval(solid: &Solid, height: f64, angle: f64) -> Option<[f64; 2]> {
    let origin = [0.0, height, 0.0];
    let direction = [angle.cos(), 0.0, angle.sin()];
    let mut hits = Vec::new();
    for &[a, b, c] in &solid.faces {
        let [a, b, c] = [solid.positions[a], solid.positions[b], solid.positions[c]];
        let edge = sub(b, a);
        let other = sub(c, a);
        let h = cross(direction, other);
        let determinant = dot(edge, h);
        if determinant.abs() < 1e-14 {
            continue;
        }
        let relative = sub(origin, a);
        let u = dot(relative, h) / determinant;
        let q = cross(relative, edge);
        let v = dot(direction, q) / determinant;
        let distance = dot(other, q) / determinant;
        if u >= -1e-9 && v >= -1e-9 && u + v <= 1.0 + 1e-9 && distance > 0.0 {
            hits.push(distance);
        }
    }
    hits.sort_by(f64::total_cmp);
    hits.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    (hits.len() >= 2).then(|| [hits[0], *hits.last().unwrap()])
}

#[test]
fn actual_crossing_triangles_do_not_interpenetrate() {
    for (pitch, phase, width) in [
        (0.08, 25.0_f64, 0.009),
        (0.06, 180.0, 0.012),
        (0.10, 90.0, 0.012),
    ] {
        let mut value = fixture();
        value["shaft"]["wrappings"] = serde_json::json!([{
            "start":1.668,"length":0.16,"pitch":pitch,"width":width,"thickness":0.0015,
            "phase":phase,"pattern":"crossed","material":"leather"}]);
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        let center = 1.668 + width / 2.0 + pitch / 2.0;
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let parts =
                shaft_wrapping::parts(recipe.shaft.as_ref().unwrap(), "shaft", "shaft", detail)
                    .unwrap();
            let mut shared = 0;
            for row in -8..=8 {
                for column in -8..=8 {
                    let y = center + row as f64 * width / 16.0;
                    let angle = phase.to_radians() + std::f64::consts::PI + column as f64 * 0.035;
                    if let (Some(a), Some(b)) = (
                        radial_interval(&parts[0].solid, y, angle),
                        radial_interval(&parts[1].solid, y, angle),
                    ) {
                        shared += 1;
                        assert!(
                            a[1] <= b[0] + 1e-8,
                            "crossing overlap {pitch} {phase} {detail:?}: {a:?} {b:?}"
                        );
                    }
                }
            }
            assert!(shared > 10, "crossing test must exercise shared rays");
        }
    }
}

#[test]
fn fillet_cannot_overrun_the_available_stop_projection() {
    let mut value = fixture();
    value["components"][0]["socket"]["stops"]["span"] = 0.045.into();
    let recipe: Recipe = serde_json::from_value(value.clone()).unwrap();
    generate_model(&recipe, Detail::High).unwrap();
    value["components"][0]["socket"]["stops"]["rootBlend"] = 0.008.into();
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    assert_eq!(recipe.validate(), Err(RecipeError::Proportion));
}

#[test]
fn small_stop_fillets_remain_closed_within_the_construction_budget() {
    for blend in [0.0001, 0.0005, 0.001, 0.003] {
        let mut value = fixture();
        value["components"][0]["socket"]["stops"]["rootBlend"] = blend.into();
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        recipe.validate().unwrap();
        let Shape::Spear(p) = &recipe.components[0].shape else {
            unreachable!()
        };
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = spears::spear(p, detail)
                .unwrap()
                .transform([0.0; 3], [0.0, 1.948, 0.0]);
            closed(&solid);
        }
    }
}

#[test]
fn round_socket_to_diamond_neck_has_convex_emitted_sections() {
    let recipe: Recipe = serde_json::from_value(fixture()).unwrap();
    let Shape::Spear(p) = &recipe.components[0].shape else {
        unreachable!()
    };
    let socket = p.socket.as_ref().unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = spears::spear(p, detail).unwrap();
        let mut sections = BTreeMap::<_, Vec<Point>>::new();
        for &point in &solid.positions {
            if point[1] > -socket.neck_length.get() && point[1] < 0.0 {
                sections
                    .entry((point[1] * 1e12).round() as i64)
                    .or_default()
                    .push(point);
            }
        }
        assert!(sections.len() > 8);
        for (_, mut points) in sections {
            points.sort_by(|a, b| a[2].atan2(a[0]).total_cmp(&b[2].atan2(b[0])));
            points.dedup();
            for i in 0..points.len() {
                let [a, b, c] = [
                    points[i],
                    points[(i + 1) % points.len()],
                    points[(i + 2) % points.len()],
                ];
                let turn = (b[0] - a[0]) * (c[2] - b[2]) - (b[2] - a[2]) * (c[0] - b[0]);
                assert!(turn >= -1e-15, "concave neck section at {}: {turn}", a[1]);
            }
        }
    }
}

#[test]
fn oversized_neck_tangents_reject_before_sampling_at_every_detail() {
    let mut value = fixture();
    value.as_object_mut().unwrap().remove("shaft");
    value["components"][0]
        .as_object_mut()
        .unwrap()
        .remove("mount");
    value["components"][0]["attach"] = serde_json::json!({"to":"weapon.root","at":"origin"});
    value["components"][0]["rootWidth"] = 0.02.into();
    value["components"][0]["socket"] = serde_json::json!({
        "baseRadius":0.03,"length":0.20,"neckLength":0.17,"wall":0.001,
        "cavityDepth":0.01,"insertionDepth":0.008,"boreTipRadius":0.008
    });
    let invalid: Recipe = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(invalid.validate(), Err(RecipeError::Proportion));
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        assert!(generate_model(&invalid, detail).is_err());
    }
    // The width axis remains positive here, while the thinner depth axis
    // crosses zero between its endpoints.
    value["components"][0]["socket"]["neckLength"] = 0.14.into();
    value["components"][0]["thickness"] = 0.002.into();
    let invalid_depth: Recipe = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(invalid_depth.validate(), Err(RecipeError::Proportion));
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        assert!(generate_model(&invalid_depth, detail).is_err());
    }
    value["components"][0]["socket"]["neckLength"] = 0.05.into();
    let valid: Recipe = serde_json::from_value(value).unwrap();
    valid.validate().unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        generate_model(&valid, detail).unwrap();
    }
}

#[test]
fn smooth_neck_interiors_have_no_off_axis_split_normals() {
    let recipe: Recipe = serde_json::from_value(fixture()).unwrap();
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let model = generate_model(&recipe, detail).unwrap();
        let part = model
            .parts
            .iter()
            .find(|part| part.component_id == "head")
            .unwrap();
        let mut vertices = BTreeMap::<_, Point>::new();
        for (position, normal) in part
            .positions
            .as_chunks::<3>()
            .0
            .iter()
            .zip(part.normals.as_chunks::<3>().0)
        {
            if position[1] <= 1.913
                || position[1] >= 1.948
                || position[0].abs() < 1e-7
                || position[2].abs() < 1e-7
            {
                continue;
            }
            let key = position.map(|v| (v * 1e9).round() as i64);
            if let Some(previous) = vertices.insert(key, *normal) {
                assert!(
                    magnitude(sub(previous, *normal)) < 1e-6,
                    "split neck normal {detail:?} at {position:?}"
                );
            }
        }
        assert!(vertices.len() > 100);
    }
}

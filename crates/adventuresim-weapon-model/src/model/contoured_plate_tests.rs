use super::*;
use std::collections::BTreeMap;

fn blank() -> serde_json::Value {
    serde_json::json!({
        "kind":"contouredPlate", "width":0.08,"length":0.12,"start":[-0.5,0],
        "boundary":[{"kind":"line","to":[0.5,0]},
            {"kind":"line","to":[0.5,1]}, {"kind":"line","to":[-0.5,1]},
            {"kind":"line","to":[-0.5,0]}],
        "thickness":[
            {"at":0,"edge":0.002,"ridge":0.010,"ridgeHalfWidth":0.020,"flatHalfWidth":0},
            {"at":1,"edge":0.002,"ridge":0.010,"ridgeHalfWidth":0.020,"flatHalfWidth":0}]
    })
}

fn parameters(value: serde_json::Value) -> ContouredPlateParameters {
    let Shape::ContouredPlate(p) = serde_json::from_value(value).unwrap() else {
        panic!("expected a plate")
    };
    p
}

fn closed(solid: &Solid) {
    for export in [false, true] {
        let point = |p| {
            if export {
                add(rotate(p, [17.0, 31.0, -23.0]), [17.0, -18.0, 19.0]).map(|v| v as f32 as f64)
            } else {
                p
            }
        };
        let key = |p: Point| p.map(|v| if v == 0.0 { 0 } else { v.to_bits() });
        let mut edges = BTreeMap::<_, Vec<_>>::new();
        for face in &solid.faces {
            let p = face.map(|i| point(solid.positions[i]));
            assert!(p.iter().flatten().all(|v| v.is_finite()));
            assert!(
                magnitude(cross(sub(p[1], p[0]), sub(p[2], p[0]))) > 0.0,
                "collapsed {p:?}"
            );
            for i in 0..3 {
                let [a, b] = [key(p[i]), key(p[(i + 1) % 3])];
                edges
                    .entry(if a < b { [a, b] } else { [b, a] })
                    .or_default()
                    .push(a < b);
            }
        }
        assert!(
            edges.values().all(|v| v.len() == 2 && v[0] != v[1]),
            "nonmanifold export={export}"
        );
    }
}

#[test]
fn plate_ridge_preserves_exact_section_volume_and_closed_export() {
    let p = parameters(blank());
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = contoured_plate::construct(&p, detail).unwrap();
        closed(&solid);
        let expected = 0.12 * (0.08 * 0.002 + 0.020 * 0.008);
        assert!((solid.volume() - expected).abs() < expected * 1e-10);
    }
}

#[test]
fn relieved_plate_slopes_preserve_analytic_material_volume() {
    for depths in [[0.001, 0.001], [0.0, 0.002], [0.002, 0.0005]] {
        let mut value = blank();
        for (i, depth) in depths.into_iter().enumerate() {
            value["thickness"][i]["hollowDepth"] = depth.into();
        }
        let p = parameters(value);
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = contoured_plate::construct(&p, detail).unwrap();
            closed(&solid);
            let expected = 0.12 * (0.08 * 0.002 + 0.020 * 0.008 - 0.020 * (depths[0] + depths[1]));
            assert!((solid.volume() - expected).abs() < expected * 1e-10);
        }
    }
}

#[test]
fn relieved_plate_rejects_negative_or_inverted_slopes() {
    for depth in [-0.001, 0.00201] {
        let mut value = blank();
        value["thickness"][0]["hollowDepth"] = depth.into();
        value["id"] = "blank".into();
        value["attach"] = serde_json::json!({"to":"weapon.root","at":"base"});
        let recipe: Recipe = serde_json::from_value(serde_json::json!({
            "components":[value]
        }))
        .unwrap();
        assert!(recipe.validate().is_err());
    }
}

#[test]
fn relieved_head_variations_fit_the_unchanged_allocation_budget() {
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../../review/museum/met-08.261.2.json")).unwrap();
    for (length, ridge) in [(0.72, 0.0077), (0.646, 0.006)] {
        let mut value = study["definition"].clone();
        value["components"][3]["length"] = length.into();
        value["components"][3]["thickness"][5]["ridge"] = ridge.into();
        let recipe: Recipe = serde_json::from_value(value).unwrap();
        generate_model(&recipe, Detail::High).unwrap();
    }
}

#[test]
fn plate_point_closes_the_complete_thickness_section() {
    let mut value = blank();
    value["boundary"] = serde_json::json!([
        {"kind":"line","to":[0.5,0]},
        {"kind":"line","to":[0,1]},
        {"kind":"line","to":[-0.5,0]}
    ]);
    value["thickness"][1] =
        serde_json::json!({"at":1,"edge":0,"ridge":0,"ridgeHalfWidth":0,"flatHalfWidth":0});
    let p = parameters(value);
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = contoured_plate::construct(&p, detail).unwrap();
        closed(&solid);
        assert!(
            solid
                .positions
                .iter()
                .filter(|v| v[1] == p.length.get())
                .all(|v| *v == [0.0, p.length.get(), 0.0])
        );
        let expected = 0.12 / 3.0 * (0.08 * 0.002 + 0.020 * 0.008);
        assert!((solid.volume() - expected).abs() < expected * 1e-10);
    }
}

#[test]
fn plate_rejects_collapsed_sections_and_nonunique_zero_ends() {
    let recipe = |shape: serde_json::Value| -> Recipe {
        serde_json::from_value(serde_json::json!({"components":[shape]})).unwrap()
    };
    let mut value = blank();
    value["thickness"][1] =
        serde_json::json!({"at":1,"edge":0,"ridge":0,"ridgeHalfWidth":0,"flatHalfWidth":0});
    assert!(recipe(value).validate().is_err());
    let mut value = blank();
    value["thickness"][0]["flatHalfWidth"] = 0.02.into();
    assert!(recipe(value).validate().is_err());
    let mut value = blank();
    value["thickness"][0]["edge"] = 0.into();
    assert!(recipe(value).validate().is_err());
}

#[test]
fn plate_cut_preserves_closed_geometry_or_rejects_sub_resolution_separations() {
    for separation in [-1e-10, 1e-10] {
        let mut value = blank();
        for station in value["thickness"].as_array_mut().unwrap() {
            station["ridgeHalfWidth"] = (0.02 + separation).into();
        }
        let p = parameters(value);
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            match contoured_plate::construct(&p, detail) {
                Ok(solid) => closed(&solid),
                Err(error) => assert!(error.contains("sub-resolution")),
            }
        }
    }
}

#[test]
fn moving_ridge_respects_surface_deviation_between_vertices() {
    let mut value = blank();
    value["thickness"][0]["ridgeHalfWidth"] = 0.012.into();
    value["thickness"][1]["ridgeHalfWidth"] = 0.042.into();
    let p = parameters(value);
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = contoured_plate::construct(&p, detail).unwrap();
        for face in &solid.faces {
            let points = face.map(|i| solid.positions[i]);
            if points.iter().any(|p| p[2] < 0.0) {
                continue;
            }
            let area = (points[1][0] - points[0][0]) * (points[2][1] - points[0][1])
                - (points[2][0] - points[0][0]) * (points[1][1] - points[0][1]);
            if area <= 0.0 {
                continue;
            }
            for a in 0..=8 {
                for b in 0..=8 - a {
                    let weights = [a as f64 / 8.0, b as f64 / 8.0, (8 - a - b) as f64 / 8.0];
                    let point: Point =
                        std::array::from_fn(|i| (0..3).map(|j| points[j][i] * weights[j]).sum());
                    let half_width = 0.012 + 0.03 * point[1] / 0.12;
                    let expected = 0.001 + 0.004 * (1.0 - point[0].abs() / half_width).max(0.0);
                    assert!((point[2] - expected).abs() <= detail.error(0.00005) + 1e-12);
                }
            }
        }
    }
}

#[test]
fn curved_cells_preserve_closed_export_with_and_without_a_ridge() {
    for ridge in [0.002, 0.005] {
        let mut value = blank();
        value["length"] = 0.1.into();
        value["boundary"] = serde_json::json!([
            {"kind":"line","to":[0.5,0]}, {"kind":"line","to":[0.5,1]},
            {"kind":"cubic","controls":[[0.25,0.55],[0.1,0.55]],"to":[0,0.85]},
            {"kind":"cubic","controls":[[-0.1,0.55],[-0.3,0.55]],"to":[-0.5,1]},
            {"kind":"line","to":[-0.5,0]}
        ]);
        for station in value["thickness"].as_array_mut().unwrap() {
            station["ridgeHalfWidth"] = 0.012.into();
            station["ridge"] = ridge.into();
        }
        let p = parameters(value);
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            closed(&contoured_plate::construct(&p, detail).unwrap());
        }
    }
}

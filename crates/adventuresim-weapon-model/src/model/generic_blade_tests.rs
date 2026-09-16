use super::*;
use std::collections::BTreeMap;

fn parameters() -> BladeParameters {
    serde_json::from_value(serde_json::json!({
        "length":0.254,"width":0.030,"thickness":0.005,"curvature":0,
        "taper":0.45,"singleEdge":1,"tipWidth":0.01,"belly":0.1,
        "point":{"start":0.9,"roundness":0.2}
    }))
    .unwrap()
}

fn closed(solid: &Solid) {
    for transformed in [false, true] {
        let point = |p: Point| {
            let p = if transformed {
                add(rotate(p, [17.0, 31.0, -23.0]), [17.0, -18.0, 19.0])
            } else {
                p
            };
            p.map(|v| if transformed { v as f32 as f64 } else { v })
        };
        let key = |p: Point| p.map(|v| if v == 0.0 { 0 } else { v.to_bits() });
        let mut edges = BTreeMap::<_, Vec<_>>::new();
        for face in &solid.faces {
            let p = face.map(|i| point(solid.positions[i]));
            assert!(p.iter().flatten().all(|v| v.is_finite()));
            assert!(
                magnitude(cross(sub(p[1], p[0]), sub(p[2], p[0]))) > 0.0,
                "degenerate {p:?}"
            );
            for [a, b] in [[p[0], p[1]], [p[1], p[2]], [p[2], p[0]]] {
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
            "open transformed={transformed}"
        );
    }
    assert!(solid.volume() > 0.0);
}

#[test]
fn generic_points_close_both_section_axes_across_asymmetry_and_detail() {
    for single in [-1.0, -0.9999, -0.5, 0.0, 0.5, 0.9999, 1.0] {
        for roundness in [0.0, 0.5, 1.0] {
            let mut p = parameters();
            p.single_edge = Some(Ratio::new(single).unwrap());
            p.point.as_mut().unwrap().roundness = Ratio::new(roundness).unwrap();
            for detail in [Detail::Low, Detail::Medium, Detail::High] {
                let solid = generic_blade::blade(&p, detail).unwrap();
                closed(&solid);
                let tip: Vec<_> = solid
                    .positions
                    .iter()
                    .filter(|v| v[1] == p.length.get())
                    .collect();
                assert!(!tip.is_empty());
                assert!(tip.iter().all(|v| **v == [0.0, p.length.get(), 0.0]));
                if single == 1.0 {
                    assert!(solid.positions.iter().all(|p| p[0] >= 0.0));
                }
            }
        }
    }
}

#[test]
fn generic_point_preserves_join_section_and_thickness_tangent() {
    let p = parameters();
    let start = p.point_start().unwrap();
    let curve = p.point_curve().unwrap().unwrap();
    let at = p.section_dimensions(start, Some(&curve));
    assert_eq!(at, p.section_dimensions(start, None));
    let h = 1e-7;
    let left = p.section_dimensions(start - h, Some(&curve));
    let right = p.section_dimensions(start + h, Some(&curve));
    for axis in 0..3 {
        let before = (at[axis] - left[axis]) / h;
        let after = (right[axis] - at[axis]) / h;
        assert!(
            (before - after).abs() < 1e-4,
            "axis {axis}: {before} {after}"
        );
    }
}

#[test]
fn generic_blunt_end_retains_authored_width_and_depth() {
    let mut p = parameters();
    p.point = None;
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = generic_blade::blade(&p, detail).unwrap();
        closed(&solid);
        let tip: Vec<_> = solid
            .positions
            .iter()
            .filter(|v| v[1] == p.length.get())
            .collect();
        let extent = |axis| {
            tip.iter()
                .map(|v| v[axis])
                .fold(f64::NEG_INFINITY, f64::max)
                - tip.iter().map(|v| v[axis]).fold(f64::INFINITY, f64::min)
        };
        assert!((extent(0) - 0.0003).abs() < 1e-12);
        assert!((extent(2) - 0.00175).abs() < 1e-12);
    }
}

#[test]
fn generic_point_changes_with_width_depth_taper_and_start() {
    let base = parameters();
    let baseline = generic_blade::blade(&base, Detail::High).unwrap().volume();
    for (width, thickness, taper, start) in [(0.023, 0.005, 0.6, 0.8), (0.034, 0.008, 0.35, 0.95)] {
        let mut p = base.clone();
        p.width = Metres::new(width).unwrap();
        p.thickness = Metres::new(thickness).unwrap();
        p.taper = Some(Ratio::new(taper).unwrap());
        p.point.as_mut().unwrap().start = Ratio::new(start).unwrap();
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = generic_blade::blade(&p, detail).unwrap();
            closed(&solid);
            assert!((solid.volume() - baseline).abs() > 1e-8);
        }
    }
}

#[test]
fn generic_point_invalid_domains_reject_before_meshing() {
    for start in [0.0, 1.0] {
        let mut p = parameters();
        p.point.as_mut().unwrap().start = Ratio::new(start).unwrap();
        assert_eq!(p.validate_form(), Err(RecipeError::Proportion));
    }
    let mut p = parameters();
    p.point = None;
    p.tip_width = Some(Ratio::new(0.0).unwrap());
    assert_eq!(p.validate_form(), Err(RecipeError::Proportion));
    p.point = parameters().point;
    p.validate_form().unwrap();
    closed(&generic_blade::blade(&p, Detail::High).unwrap());
    p.thickness = Metres::new(0.0017).unwrap();
    assert_eq!(p.validate_form(), Err(RecipeError::Proportion));
    p.thickness = Metres::new(0.0018).unwrap();
    p.validate_form().unwrap();
    p.tip_width = Some(Ratio::new(1.0).unwrap());
    p.belly = Some(Ratio::new(0.0).unwrap());
    closed(&generic_blade::blade(&p, Detail::High).unwrap());
    p.belly = Some(Ratio::new(0.2).unwrap());
    p.point.as_mut().unwrap().start = Ratio::new(0.25).unwrap();
    assert_eq!(p.validate_form(), Err(RecipeError::Proportion));
}

#[test]
fn generic_heel_anchor_tracks_width_asymmetry_and_rotation() {
    for width in [0.023, 0.034] {
        for single in [-1.0, 0.0, 0.5, 1.0] {
            for yaw in [0.0, 90.0] {
                let mut blade = serde_json::to_value(parameters()).unwrap();
                blade["kind"] = "blade".into();
                blade["id"] = "blade".into();
                blade["width"] = width.into();
                blade["singleEdge"] = single.into();
                blade["rotation"] = serde_json::json!([0, yaw, 0]);
                blade["attach"] = serde_json::json!({"to":"guard.top","at":"heel-center"});
                let recipe: Recipe = serde_json::from_value(serde_json::json!({"components":[
                    {"kind":"shaft","id":"guard","length":0.01,"radius":0.03,
                     "attach":{"to":"weapon.root","at":"base"}},blade]}))
                .unwrap();
                let model = generate_model(&recipe, Detail::High).unwrap();
                assert_eq!(
                    model.resolved_definition.frames["blade.heelCenter"],
                    model.resolved_definition.frames["guard.top"]
                );
                let part = model
                    .parts
                    .iter()
                    .find(|p| p.component_id == "blade")
                    .unwrap();
                let heel: Vec<_> = part
                    .positions
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .filter(|p| (p[1] - 0.01).abs() < 1e-12)
                    .collect();
                assert!(heel.iter().all(|p| p[0].hypot(p[2]) < 0.03));
                for axis in [0, 2] {
                    let lo = heel.iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min);
                    let hi = heel
                        .iter()
                        .map(|p| p[axis])
                        .fold(f64::NEG_INFINITY, f64::max);
                    assert!((lo + hi).abs() < 1e-12);
                }
            }
        }
    }
}

#[test]
fn generic_loft_volume_matches_continuous_wedge_sections() {
    let mut p = parameters();
    p.point = None;
    let n = 16384;
    for single in [-1.0, 0.0, 1.0] {
        p.single_edge = Some(Ratio::new(single).unwrap());
        let area = |t: f64| {
            let w = 0.030
                * (0.01 + 0.99 * (1.0 - t).powf(0.45))
                * (1.0 + 0.1 * (std::f64::consts::PI * t).sin());
            w * (0.005 * (1.0 - 0.65 * t) + 0.0006) / 2.0
        };
        let mut integral = area(0.0) + area(1.0);
        for i in 1..n {
            integral += (if i % 2 == 0 { 2.0 } else { 4.0 }) * area(i as f64 / n as f64);
        }
        integral *= 0.254 / (3.0 * n as f64);
        let measured = generic_blade::blade(&p, Detail::High).unwrap().volume();
        assert!((measured - integral).abs() / integral < 0.0002);
    }
}

#[test]
fn diamond_section_preserves_its_proportions_and_closes_a_single_point() {
    for ratio in [0.4, 1.0, 1.6] {
        let mut p = parameters();
        p.section = Some(ForgedBladeSection::Diamond);
        p.single_edge = Some(Ratio::new(0.0).unwrap());
        p.thickness = Metres::new(p.width.get() * ratio).unwrap();
        p.curvature = Some(Metres::new(-0.025).unwrap());
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let solid = generic_blade::blade(&p, detail).unwrap();
            closed(&solid);
            let mut sections = BTreeMap::<_, Vec<Point>>::new();
            for &point in &solid.positions {
                sections.entry(point[1].to_bits()).or_default().push(point);
            }
            for section in sections.values() {
                let span = |axis| {
                    section
                        .iter()
                        .map(|p| p[axis])
                        .fold(f64::NEG_INFINITY, f64::max)
                        - section
                            .iter()
                            .map(|p| p[axis])
                            .fold(f64::INFINITY, f64::min)
                };
                assert!((span(2) - ratio * span(0)).abs() < 1e-12);
            }
        }
    }
}

#[test]
fn diamond_frustum_conserves_analytic_volume_and_rejects_an_asymmetric_edge() {
    let mut p = parameters();
    p.section = Some(ForgedBladeSection::Diamond);
    p.single_edge = Some(Ratio::new(0.0).unwrap());
    p.point = None;
    p.taper = Some(Ratio::new(1.0).unwrap());
    p.belly = Some(Ratio::new(0.0).unwrap());
    p.tip_width = Some(Ratio::new(0.5).unwrap());
    let volume =
        0.5 * p.width.get() * p.thickness.get() * p.length.get() * (1.0 + 0.5 + 0.25) / 3.0;
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = generic_blade::blade(&p, detail).unwrap();
        assert!((solid.volume() / volume - 1.0).abs() < 1e-12);
        closed(&solid);
    }
    p.single_edge = Some(Ratio::new(0.1).unwrap());
    assert!(generic_blade::blade(&p, Detail::High).is_err());
}

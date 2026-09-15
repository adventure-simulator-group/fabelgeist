//! Cross-section and transport contracts migrated from browser helper tests.
use super::*;

#[test]
fn path_refinement_preserves_authored_stations_scales_and_twist_progress() {
    let solid = Solid::sweep(
        &[[0.0; 3], [0.0, 0.01, 0.0], [0.0, 0.11, 0.0]],
        &Sweep {
            section: Section::Flat,
            width: 0.012,
            depth: 0.004,
            twist: 90.0,
            ring_scales: Some(vec![1.0, 0.5, 0.2]),
            ..Sweep::default()
        },
        Detail::Medium,
    )
    .unwrap();
    let at = |y: f64| {
        solid
            .positions
            .iter()
            .filter(move |p| (p[1] - y).abs() < 1e-12)
    };
    let middle: Vec<_> = at(0.01).collect();
    assert!(!middle.is_empty());
    let width = middle.iter().map(|p| p[0].abs()).fold(0.0, f64::max);
    let depth = middle.iter().map(|p| p[2].abs()).fold(0.0, f64::max);
    assert!((width - depth).abs() < 1e-12);
    assert!((width - 0.004 * 0.5_f64.sqrt()).abs() < 1e-12);
    let end: Vec<_> = at(0.11).collect();
    assert!(!end.is_empty());
    assert!(
        (end.iter().map(|p| p[0].hypot(p[2])).fold(0.0, f64::max) - 0.006_f64.hypot(0.002) * 0.2)
            .abs()
            < 1e-12
    );
    assert!((end.iter().map(|p| p[0].abs()).fold(0.0, f64::max) - 0.002 * 0.2).abs() < 1e-12);
    assert!((end.iter().map(|p| p[2].abs()).fold(0.0, f64::max) - 0.006 * 0.2).abs() < 1e-12);
}

#[test]
fn d_section_has_one_simple_flat_back_and_curved_belly() {
    let outline = Section::DShape.outline(0.032, 0.036, 16, Detail::Medium);
    Region::triangulate(&outline, true).unwrap();
    assert_eq!(
        outline
            .iter()
            .filter(|p| (p[0] + 0.016).abs() < 1e-9)
            .count(),
        2
    );
    assert!(outline.iter().any(|p| p[0] > 0.015));
    for pair in outline.windows(2) {
        assert!((pair[0][0] - pair[1][0]).hypot(pair[0][1] - pair[1][1]) > 1e-8);
    }
    for (i, a) in outline.iter().enumerate() {
        for b in &outline[i + 1..] {
            assert!((a[0] - b[0]).hypot(a[1] - b[1]) > 1e-8);
        }
    }
}

#[test]
fn round_bars_meet_physical_section_chord_and_sagitta_budgets() {
    for radius in [0.004, 0.006, 0.009, 0.014, 0.025] {
        for requested in [8, 12, 24] {
            let count = tube_segments(radius, requested, Detail::Medium);
            assert!(count >= 12.max(requested));
            assert!(2.0 * radius * (PI / count as f64).sin() <= 0.006 + 1e-12);
            assert!(radius * (1.0 - (PI / count as f64).cos()) <= 0.0003 + 1e-12);
            let solid = Solid::sweep(
                &[[0.0; 3], [0.0, 0.1, 0.0]],
                &Sweep {
                    width: radius * 2.0,
                    depth: radius * 2.0,
                    radial_segments: requested,
                    ..Sweep::default()
                },
                Detail::Medium,
            )
            .unwrap();
            assert!(solid.volume() > 0.0);
            let mut ring: Vec<_> = solid
                .positions
                .iter()
                .filter(|p| p[1].abs() < 1e-12 && p[0].hypot(p[2]) > radius * 0.99)
                .collect();
            ring.sort_by(|a, b| a[2].atan2(a[0]).total_cmp(&b[2].atan2(b[0])));
            ring.dedup_by(|a, b| magnitude(sub(**a, **b)) < 1e-12);
            assert_eq!(ring.len(), count);
            for i in 0..ring.len() {
                let a = ring[i];
                let b = ring[(i + 1) % ring.len()];
                assert!(magnitude(sub(*a, *b)) <= 0.006 + 1e-12);
                assert!(
                    radius - ((a[0] + b[0]) / 2.0).hypot((a[2] + b[2]) / 2.0) <= 0.0003 + 1e-12
                );
            }
            assert!(
                solid.faces.len() > count * 2,
                "longitudinal refinement must survive section construction"
            );
        }
    }
}

#[test]
fn parallel_transport_sections_remain_finite_through_global_z() {
    for section in [
        Section::Round,
        Section::Oval,
        Section::Diamond,
        Section::Flat,
        Section::Triangular,
    ] {
        let solid = Solid::sweep(
            &[
                [0.0; 3],
                [0.0001, 0.0, 0.03],
                [0.0, 0.0, 0.06],
                [-0.0001, 0.0, 0.09],
            ],
            &Sweep {
                section,
                width: 0.012,
                depth: 0.008,
                twist: 80.0,
                ..Sweep::default()
            },
            Detail::Medium,
        )
        .unwrap();
        crate::construction::surface_tests::closed(solid.clone());
        assert!(solid.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(solid.volume() > 0.0);
        let model = crate::model::ModelPart::from_source(crate::model::output::PartSource::new(
            solid,
            crate::Material::Steel,
            "member",
            "member",
        ));
        assert!(model.normals.iter().all(|v| v.is_finite()));
        if matches!(section, Section::Round | Section::Oval) {
            assert!(model.positions.len() / 3 < model.indices.len() / 2);
        } else {
            let mut corner = false;
            for i in (0..model.positions.len()).step_by(3) {
                for j in (i + 3..model.positions.len()).step_by(3) {
                    if model.positions[i..i + 3] == model.positions[j..j + 3]
                        && (0..3)
                            .map(|a| model.normals[i + a] * model.normals[j + a])
                            .sum::<f64>()
                            < 0.8
                    {
                        corner = true;
                    }
                }
            }
            assert!(corner);
        }
    }
}

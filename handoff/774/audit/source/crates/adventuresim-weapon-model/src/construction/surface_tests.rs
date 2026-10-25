//! Solid closure and shading assertions migrated from browser primitive tests.
use super::*;
use crate::model::output::PartSource;
use crate::{Material, ModelPart};
use std::collections::BTreeMap;

fn model(solid: Solid) -> ModelPart {
    ModelPart::from_source(PartSource::new(
        solid,
        Material::Steel,
        "fixture",
        "fixture",
    ))
}
pub(super) fn closed(solid: Solid) {
    assert!(solid.volume() > 0.0);
    let mesh = model(solid);
    assert!(mesh.positions.iter().all(|n| n.is_finite()));
    assert!(
        mesh.normals
            .as_chunks::<3>()
            .0
            .iter()
            .all(|n| n.iter().all(|v| v.is_finite())
                && (n.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-10)
    );
    for float32 in [false, true] {
        let mut edges = BTreeMap::<_, (u32, i32)>::new();
        for face in mesh.indices.as_chunks::<3>().0 {
            let coordinates = |i: u32| {
                std::array::from_fn::<_, 3, _>(|a| {
                    let v = mesh.positions[i as usize * 3 + a];
                    if float32 { v as f32 as f64 } else { v }
                })
            };
            let a = coordinates(face[0]);
            let b = coordinates(face[1]);
            let c = coordinates(face[2]);
            assert!(magnitude(cross(sub(b, a), sub(c, a))) > 0.0);
            let point = |i: u32| coordinates(i).map(|v| if v == 0.0 { 0 } else { v.to_bits() });
            for i in 0..3 {
                let a = point(face[i]);
                let b = point(face[(i + 1) % 3]);
                assert_ne!(a, b);
                let key = if a < b { (a, b) } else { (b, a) };
                let e = edges.entry(key).or_default();
                e.0 += 1;
                e.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(edges.values().all(|e| *e == (2, 0)));
    }
}

#[test]
fn refined_concave_prisms_and_zero_radius_lathe_poles_are_closed() {
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        closed(
            Solid::prism(
                &[
                    [0.0, 0.0],
                    [0.12, 0.0],
                    [0.12, 0.04],
                    [0.04, 0.04],
                    [0.04, 0.12],
                    [0.0, 0.12],
                ],
                0.006,
                detail,
            )
            .unwrap(),
        );
        for profile in [
            vec![[0.0, 0.0], [0.04, 0.02], [0.08, 0.0]],
            vec![[0.0, 0.02], [0.04, 0.0]],
            vec![[0.0, 0.0], [0.04, 0.02]],
        ] {
            closed(Solid::lathe(&profile, 12, 1.0, false, detail).unwrap());
        }
    }
}

#[test]
fn closed_tapered_cylinders_keep_periodic_seams_after_float32_conversion() {
    for radius in [0.0045, 0.02, 1.0] {
        for length in [0.76, 1.82] {
            for segments in [8, 9, 16] {
                closed(
                    Solid::lathe(
                        &[[0.0, radius], [length, radius * 0.92]],
                        segments,
                        1.0,
                        true,
                        Detail::Medium,
                    )
                    .unwrap(),
                );
            }
        }
    }
}

#[test]
fn round_surfaces_share_radial_normals_while_caps_and_octagonal_flats_split() {
    let mesh =
        model(Solid::lathe(&[[0.0, 0.02], [0.2, 0.02]], 24, 1.0, false, Detail::Medium).unwrap());
    assert!(mesh.positions.len() / 3 < mesh.indices.len() / 2);
    let mut sides = 0;
    let mut caps = 0;
    for (p, n) in mesh
        .positions
        .as_chunks::<3>()
        .0
        .iter()
        .zip(mesh.normals.as_chunks::<3>().0.iter())
    {
        if n[1].abs() > 0.99 {
            caps += 1;
        } else {
            sides += 1;
            assert!(n[1].abs() < 1e-6);
            assert!(n[0] * p[0] / 0.02 + n[2] * p[2] / 0.02 > 0.999);
        }
    }
    assert!(sides > 0 && caps > 0);
    let octagon =
        model(Solid::lathe(&[[0.0, 0.02], [0.2, 0.02]], 8, 1.0, true, Detail::Medium).unwrap());
    let normals: Vec<_> = octagon
        .positions
        .as_chunks::<3>()
        .0
        .iter()
        .zip(octagon.normals.as_chunks::<3>().0.iter())
        .filter(|(p, n)| (p[0] - 0.02).abs() < 1e-8 && p[1].abs() < 1e-8 && n[1].abs() < 0.01)
        .map(|(_, n)| n)
        .collect();
    assert_eq!(normals.len(), 2);
    assert!((0..3).map(|i| normals[0][i] * normals[1][i]).sum::<f64>() < 0.8);
}

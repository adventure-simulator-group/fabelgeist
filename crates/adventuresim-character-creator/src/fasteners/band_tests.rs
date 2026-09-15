use super::*;
use adventuresim_armor_model::{ArmorDetail, ArmorLod};

#[test]
fn exported_band_chords_clear_rotated_eccentric_support_at_every_lod() {
    for lod in [ArmorLod::Lod4, ArmorLod::Lod5, ArmorLod::Lod6] {
        for rotation in [0.0_f32, 0.12, 0.37, 0.8] {
            for size in [0.04, 0.15, 0.3] {
                let mut plate = cuboid([size, 0.05, size * 0.6]);
                for p in &mut plate.positions {
                    let [x, z] = [p[0], p[2]];
                    p[0] = x * rotation.cos() - z * rotation.sin();
                    p[2] = x * rotation.sin() + z * rotation.cos();
                }
                let section = ClosureSection::new(
                    &[],
                    &[],
                    &plate.positions,
                    plate.indices.as_chunks::<3>().0,
                    0.0,
                    0.02,
                    0.0,
                )
                .unwrap();
                let strap = band(
                    &section,
                    0.0,
                    0.02,
                    0.002,
                    0.1,
                    5.9,
                    ArmorDetail::Runtime(lod),
                    |_| 0.0,
                )
                .unwrap();
                strap.normals().unwrap();
                for pair in strap
                    .positions
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .collect::<Vec<_>>()
                    .windows(2)
                {
                    for step in 0..=16 {
                        let t = step as f32 / 16.0;
                        let x = pair[0][0][0] + (pair[1][0][0] - pair[0][0][0]) * t;
                        let z = pair[0][0][2] + (pair[1][0][2] - pair[0][0][2]) * t;
                        assert!(
                            x.hypot(z) > section.radius(x.atan2(z)).unwrap(),
                            "an exported strap triangle cuts the supporting plate"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn complete_closure_clears_convex_corners_through_fold_and_free_band() {
    for lod in [ArmorLod::Lod4, ArmorLod::Lod5, ArmorLod::Lod6] {
        for rotation in [0.0_f32, 0.12, 0.37, 0.8] {
            for size in [0.04, 0.15, 0.3] {
                let mut plate = cuboid([size, 0.05, size * 0.6]);
                for p in &mut plate.positions {
                    let [x, z] = [p[0], p[2]];
                    p[0] = x * rotation.cos() - z * rotation.sin();
                    p[2] = x * rotation.sin() + z * rotation.cos();
                }
                let section = ClosureSection::new(
                    &[],
                    &[],
                    &plate.positions,
                    plate.indices.as_chunks::<3>().0,
                    0.0,
                    0.02,
                    0.0,
                )
                .unwrap();
                let (strap, _) = closure(
                    &section,
                    0.0,
                    &StrapDesign::default(),
                    ArmorDetail::Runtime(lod),
                )
                .unwrap();
                strap.normals().unwrap();
                for face in strap.indices.as_chunks::<3>().0.iter() {
                    for i in 0..3 {
                        let a = strap.positions[face[i] as usize];
                        let b = strap.positions[face[(i + 1) % 3] as usize];
                        for step in 0..=8 {
                            let t = step as f32 / 8.0;
                            let x = a[0] + (b[0] - a[0]) * t;
                            let z = a[2] + (b[2] - a[2]) * t;
                            assert!(
                                x.hypot(z) > section.radius(x.atan2(z)).unwrap(),
                                "complete closure cuts support: size={size}, rotation={rotation}, lod={lod:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn radial_fold_segments_accept_coincident_and_short_angular_projections() {
    for size in [0.035, 0.1, 0.3] {
        let plate = cuboid([size, 0.05, size * 0.6]);
        let section = ClosureSection::new(
            &[],
            &[],
            &plate.positions,
            plate.indices.as_chunks::<3>().0,
            0.0,
            0.02,
            0.0,
        )
        .unwrap();
        for delta in [0.0, 0.0000002, 0.000002] {
            let radii = section.chord_radii(&[0.6, 1.2, 1.2 + delta, 0.6]).unwrap();
            assert!(radii.iter().all(|r| r.is_finite() && *r > 0.0));
        }
    }
}

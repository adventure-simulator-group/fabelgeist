use super::*;

#[test]
fn closed_solids_retain_volume_at_every_detail() {
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = Solid::cuboid([0.04, 0.2, 0.01], detail).unwrap();
        assert!((solid.volume() - 0.00008).abs() < 1e-12);
        let rotated = solid.transform([23.0, -71.0, 164.0], [1.2, -3.0, 0.43]);
        assert!((rotated.volume() - 0.00008).abs() < 1e-12);
    }
}

#[test]
fn concave_plate_boundary_is_preserved_by_refinement() {
    let outline = [
        [0.0, 0.0],
        [0.04, 0.0],
        [0.04, 0.15],
        [0.02, 0.1],
        [0.0, 0.15],
    ];
    let area = signed_area(&outline);
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = Solid::prism(&outline, 0.0013, detail).unwrap();
        assert!((solid.volume() - area * 0.0013).abs() < 1e-12);
    }
}

#[test]
fn inverse_rotation_recovers_the_authored_frame() {
    let point = [0.312, -0.551, 1.793];
    let rotation = [137.31, -52.7, 62.91];
    let recovered = inverse_rotate(rotate(point, rotation), rotation);
    assert!(magnitude(sub(point, recovered)) < 1e-14);
}

#[test]
fn self_intersecting_outline_is_rejected() {
    assert!(Region::triangulate(&[[0.0, 0.0], [1.0, 1.0], [0.0, 1.0], [1.0, 0.0]], false).is_err());
    assert!(
        Region::triangulate(
            &[[0.0, 0.0], [0.0009, 0.0009], [0.0, 0.0009], [0.0009, 0.0]],
            false
        )
        .is_err()
    );
    assert!(Region::triangulate(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]], false).is_err());
    assert!(
        Region::triangulate(
            &[
                [0.0, 0.0],
                [0.04, 0.0],
                [0.04, 0.04],
                [0.02, 0.0],
                [0.0, 0.04]
            ],
            false
        )
        .is_err()
    );
}

#[test]
fn equivalent_symmetric_diagonals_keep_topology_under_roundoff() {
    let outline: Vec<_> = (0..16)
        .map(|i| {
            let angle = i as f64 * std::f64::consts::TAU / 16.0;
            [angle.cos() * 0.03, angle.sin() * 0.01]
        })
        .collect();
    let original = Region::triangulate(&outline, false).unwrap().refine(0.012);
    for perturbation in [-1e-16, 1e-16] {
        let changed: Vec<_> = outline
            .iter()
            .enumerate()
            .map(|(i, p)| {
                [
                    p[0] + perturbation * (i as f64).cos(),
                    p[1] + perturbation * (i as f64).sin(),
                ]
            })
            .collect();
        let changed = Region::triangulate(&changed, false).unwrap().refine(0.012);
        assert_eq!(original.triangles, changed.triangles);
    }
}

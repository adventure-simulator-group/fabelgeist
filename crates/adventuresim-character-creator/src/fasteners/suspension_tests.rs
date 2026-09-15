use super::{mesh, suspension::SuspensionDesign};
use adventuresim_armor_model::{ArmorComponentRole, Millimeters};
use bevy::math::Vec3;

#[test]
fn buckles_remain_rigid_and_seated_below_projecting_faulds() {
    for width in [12, 18, 24] {
        for inset in [20, 40, 60] {
            for slope in [-0.3, 0.0, 0.4] {
                for projection in [0.02, 0.10, 0.25] {
                    let design = SuspensionDesign {
                        width: Millimeters(width),
                        thickness: Millimeters(2),
                        count_per_panel: 2,
                        fauld_inset: Millimeters(30),
                        tasset_inset: Millimeters(inset),
                        leather_color: [100, 30, 20],
                    };
                    let mut fauld = mesh::cuboid([0.25, 0.10, 0.15 + projection]);
                    for p in &mut fauld.positions {
                        p[1] += 0.95;
                    }
                    let mut tassets = mesh::cuboid([0.25, 0.15, 0.15]);
                    for p in &mut tassets.positions {
                        p[1] += 0.65;
                        p[2] += slope * (p[1] - 0.65);
                    }
                    let result = design.generate(&tassets, &fauld).unwrap();
                    result.normals().unwrap();
                    let hardware = result
                        .components
                        .iter()
                        .find(|c| c.role == ArmorComponentRole::Buckles)
                        .unwrap();
                    let frame =
                        &result.positions[hardware.vertices.start..hardware.vertices.start + 16];
                    let original = mesh::buckle_shape(design.width.metres());
                    for i in 0..16 {
                        for j in i + 1..16 {
                            let old = Vec3::from_array(original.positions[i])
                                .distance(Vec3::from_array(original.positions[j]));
                            let new =
                                Vec3::from_array(frame[i]).distance(Vec3::from_array(frame[j]));
                            assert!((old - new).abs() < 1e-6, "deformed buckle");
                        }
                        let p = frame[i];
                        let stand_off = p[2] - (0.15 + slope * (p[1] - 0.65));
                        assert!(
                            (0.004..0.015).contains(&stand_off),
                            "buckle lifted off its plate: {stand_off}"
                        );
                        assert!(p[1] < 0.80, "buckle escaped tasset top");
                    }
                }
            }
        }
    }
}

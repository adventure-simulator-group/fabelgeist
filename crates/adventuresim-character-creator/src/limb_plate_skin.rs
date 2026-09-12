//! Rigid upper-arm and thigh plate ownership; proportions use fit morphs.
use adventuresim_armor_model::{ArmorComponentRole, GeneratedArmor};
use adventuresim_character_creator::armor_frames::Side;
use anyhow::{Context, Result, ensure};

#[derive(Clone, Copy)]
pub(super) enum LimbPlate {
    UpperArm,
    Thigh,
}

pub(super) fn attach(
    kind: LimbPlate,
    placement: &str,
    names: &[String],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let name = match (kind, Side::from_placement(placement)?) {
        (LimbPlate::UpperArm, Side::Left) => "l_uparm",
        (LimbPlate::UpperArm, Side::Right) => "r_uparm",
        (LimbPlate::Thigh, Side::Left) => "l_upleg",
        (LimbPlate::Thigh, Side::Right) => "r_upleg",
    };
    let anchor = names
        .iter()
        .position(|n| n == name)
        .with_context(|| format!("missing rigid limb plate attachment joint {name}"))?
        as u32;
    ensure!(
        armor
            .components
            .iter()
            .any(|c| c.role == ArmorComponentRole::Plate),
        "rigid limb plate has no plate attachment component"
    );
    for component in &armor.components {
        if component.role == ArmorComponentRole::Plate {
            armor.joint_indices[component.vertices.clone()].fill([anchor; 8]);
            armor.joint_weights[component.vertices.clone()]
                .fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::*;

    fn armor(kind: LimbPlate, length: Permille) -> GeneratedArmor {
        let frame = PartFrame {
            origin: [0.0, 1.0, 0.0],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.055, 0.14, 0.05],
        };
        let design = match kind {
            LimbPlate::UpperArm => LimbArmorDesign::Rerebrace(RerebraceDesign {
                length,
                ..Default::default()
            }),
            LimbPlate::Thigh => LimbArmorDesign::Cuisse(CuisseDesign {
                length,
                ..Default::default()
            }),
        };
        let mesh = generate_limb_armor(&design, &frame).unwrap();
        let count = mesh.positions.len();
        GeneratedArmor {
            plate_edges: mesh.plate_edges(),
            components: mesh.components.clone(),
            design_hash: [0; 32],
            surface_domain: "test".into(),
            normals: mesh.normals().unwrap(),
            positions: mesh.positions,
            texcoords: vec![[0.2, 0.7]; count],
            indices: mesh.indices,
            joint_indices: vec![[2; 8]; count],
            joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; count],
            morphs: vec![],
        }
    }

    #[test]
    fn plate_remains_rigid_under_its_limb_and_independent_helper_rotations() {
        use bevy::math::{Quat, Vec3};
        for kind in [LimbPlate::UpperArm, LimbPlate::Thigh] {
            for side in ["left", "right"] {
                let prefix = if side == "left" { "l" } else { "r" };
                let segments = match kind {
                    LimbPlate::UpperArm => ["uparm", "uparm_twist4_proc", "lowarm"],
                    LimbPlate::Thigh => ["upleg", "upleg_twist4_proc", "lowleg"],
                };
                let names = segments.map(|name| format!("{prefix}_{name}"));
                let mut plate = armor(kind, Permille(900));
                let original = plate.clone();
                attach(kind, side, &names, &mut plate).unwrap();
                for root_angle in [0.0_f32, 0.7] {
                    let rotations = [
                        Quat::from_rotation_x(root_angle),
                        Quat::from_rotation_z(50.574_f32.to_radians()),
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ];
                    let posed: Vec<_> = plate
                        .positions
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            plate.joint_indices[i]
                                .iter()
                                .zip(plate.joint_weights[i])
                                .map(|(joint, weight)| {
                                    rotations[*joint as usize] * Vec3::from_array(*p) * weight
                                })
                                .sum::<Vec3>()
                        })
                        .collect();
                    for face in plate.indices.as_chunks::<3>().0 {
                        for (a, b) in [
                            (face[0] as usize, face[1] as usize),
                            (face[1] as usize, face[2] as usize),
                        ] {
                            let before = Vec3::from_array(plate.positions[a])
                                .distance(Vec3::from_array(plate.positions[b]));
                            assert!((posed[a].distance(posed[b]) - before).abs() < 1e-6);
                        }
                    }
                    for (p, expected) in posed.iter().zip(&plate.positions) {
                        assert!(p.distance(rotations[0] * Vec3::from_array(*expected)) < 1e-6);
                    }
                }
                plate.joint_indices = original.joint_indices.clone();
                plate.joint_weights = original.joint_weights.clone();
                assert_eq!(
                    plate, original,
                    "attachment changed the authored mesh or UVs"
                );
            }
        }
    }

    #[test]
    fn attachment_retains_short_and_long_authored_plate_coverage() {
        let names = ["l_uparm", "l_uparm_twist4_proc", "l_lowarm"].map(str::to_owned);
        for length in [600, 1050].map(Permille) {
            let mut plate = armor(LimbPlate::UpperArm, length);
            let original = plate.clone();
            attach(LimbPlate::UpperArm, "left", &names, &mut plate).unwrap();
            assert_eq!(plate.positions, original.positions);
            assert_eq!(plate.indices, original.indices);
            assert_eq!(plate.plate_edges, original.plate_edges);
            assert_eq!(plate.normals, original.normals);
            assert_eq!(plate.texcoords, original.texcoords);
        }
    }

    #[test]
    fn missing_major_arm_anchor_is_rejected() {
        let mut plate = armor(LimbPlate::UpperArm, Permille(900));
        let names = ["l_uparm_twist4_proc", "l_lowarm"].map(str::to_owned);
        assert!(attach(LimbPlate::UpperArm, "left", &names, &mut plate).is_err());
    }
}

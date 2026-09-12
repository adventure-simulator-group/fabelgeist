//! Joint cops and their distal metal courses follow the anatomical lower limb.
use adventuresim_armor_model::{ArmorComponentRole, GeneratedArmor};
use adventuresim_character_creator::armor_frames::Side;
use anyhow::{Context, Result};

pub(super) enum CopJoint {
    Elbow,
    Knee,
}

impl CopJoint {
    fn anchor(self, side: Side) -> &'static str {
        match (self, side) {
            (Self::Elbow, Side::Left) => "l_lowarm",
            (Self::Elbow, Side::Right) => "r_lowarm",
            (Self::Knee, Side::Left) => "l_lowleg",
            (Self::Knee, Side::Right) => "r_lowleg",
        }
    }
}

pub(super) fn attach(
    joint: CopJoint,
    placement: &str,
    names: &[String],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let name = joint.anchor(Side::from_placement(placement)?);
    let anchor = names
        .iter()
        .position(|joint| joint == name)
        .context("missing rigid joint cop attachment")? as u32;
    anyhow::ensure!(
        armor
            .components
            .iter()
            .any(|c| c.role == ArmorComponentRole::Plate),
        "joint cop has no main plate component"
    );
    for component in &armor.components {
        if matches!(
            component.role,
            ArmorComponentRole::Plate | ArmorComponentRole::JointExtension
        ) {
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

    fn generated(design: &LimbArmorDesign) -> GeneratedArmor {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.065, 0.055, 0.050],
        };
        let mesh = generate_limb_armor(design, &frame).unwrap();
        let count = mesh.positions.len();
        GeneratedArmor {
            plate_edges: mesh.plate_edges(),
            components: mesh.components.clone(),
            design_hash: [0; 32],
            surface_domain: "test".into(),
            normals: mesh.normals().unwrap(),
            positions: mesh.positions.clone(),
            texcoords: vec![[0.2, 0.7]; count],
            indices: mesh.indices,
            joint_indices: vec![[0; 8]; count],
            joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; count],
            morphs: vec![ArmorMorph {
                name: "identity".into(),
                direct_positions: mesh.positions,
                position_deltas: vec![[0.0, 0.001, 0.0]; count],
                normal_deltas: vec![[0.0; 3]; count],
            }],
        }
    }

    #[test]
    fn generated_joint_plates_share_their_anatomical_attachment_and_keep_separate_meshes() {
        let names =
            ["skin_twist", "l_lowarm", "r_lowarm", "l_lowleg", "r_lowleg"].map(str::to_owned);
        for construction in [
            JointCupConstruction::Wrapped,
            JointCupConstruction::RaisedCop,
        ] {
            for knee in [false, true] {
                let mut d = JointCupDesign {
                    construction,
                    ..JointCupDesign::couter()
                };
                if construction == JointCupConstruction::Wrapped {
                    d.distal_extension = Some(JointExtension::default());
                }
                let design = if knee {
                    LimbArmorDesign::Poleyn(d)
                } else {
                    LimbArmorDesign::Couter(d)
                };
                for (side, offset) in [("left", 0), ("right", 1)] {
                    let mut armor = generated(&design);
                    if construction == JointCupConstruction::Wrapped {
                        assert!(armor.components.iter().any(|component| {
                            component.role == ArmorComponentRole::JointExtension
                        }));
                    }
                    let original = armor.clone();
                    let joint = if knee {
                        CopJoint::Knee
                    } else {
                        CopJoint::Elbow
                    };
                    attach(joint, side, &names, &mut armor).unwrap();
                    let expected = if knee { 3 } else { 1 } + offset;
                    let mut joint_plate_vertices = 0;
                    for component in &armor.components {
                        for vertex in component.vertices.clone() {
                            if matches!(
                                component.role,
                                ArmorComponentRole::Plate | ArmorComponentRole::JointExtension
                            ) {
                                joint_plate_vertices += 1;
                                assert_eq!(armor.joint_indices[vertex][0], expected);
                                assert_eq!(
                                    armor.joint_weights[vertex],
                                    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
                                );
                            } else {
                                assert_eq!(
                                    armor.joint_indices[vertex],
                                    original.joint_indices[vertex]
                                );
                                assert_eq!(
                                    armor.joint_weights[vertex],
                                    original.joint_weights[vertex]
                                );
                            }
                        }
                    }
                    assert!(
                        joint_plate_vertices > 0,
                        "generator omitted the attachment component"
                    );
                    armor.joint_indices = original.joint_indices.clone();
                    armor.joint_weights = original.joint_weights.clone();
                    assert_eq!(
                        armor, original,
                        "attachment changed geometry, UVs or morphs"
                    );
                }
            }
        }
    }

    #[test]
    fn attachment_rejects_missing_joint_or_missing_main_component() {
        let mut armor = generated(&LimbArmorDesign::Couter(JointCupDesign::couter()));
        assert!(attach(CopJoint::Elbow, "left", &[], &mut armor).is_err());
        armor.components.clear();
        assert!(attach(CopJoint::Elbow, "left", &["l_lowarm".into()], &mut armor).is_err());
    }
}

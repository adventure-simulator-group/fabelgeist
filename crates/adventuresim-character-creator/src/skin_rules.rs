//! Rigid plates follow one anatomical joint instead of the nearest skin.
//!
//! Solid helmets move with the head alone, so jaw and neck deformation cannot
//! bend them. Joint cops and their distal courses follow the lower limb, and
//! rerebraces, spaulders and cuisses the upper limb, so limb motion and twist
//! weights cannot shear their plates. Body proportions refit every plate before
//! attachment. Each lame of a fauld hangs from the pelvis and
//! each tasset from its own hip; a besagew hangs from the chest in front of
//! the shoulder. Arming caps, mail coifs and textiles keep the body's flexible
//! skin.

use anyhow::{Context, Result};
use fabelgeist_armor::{
    ArmorComponentRole, GarmentArmorKind, GeneratedArmor, HelmetDesign, LimbArmorDesign,
};
use fabelgeist_rig::{RigJointLookupError, RigJointName, RigJointOrdinal, RigJointPart};
use std::ops::Range;

use crate::armor_frames::Side;
use crate::armor_recipes::ParametricDesign;

/// Give `design`'s rigid plates their single anatomical owner. `joints` are
/// the wearer's global joint states, one per name.
pub fn attach(
    design: &ParametricDesign,
    placement: &str,
    joint_names: &[RigJointName],
    joints: &[[f32; 8]],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let (joint, roles): (_, &[_]) = match design {
        ParametricDesign::WaistAssembly(_) => return waist(joint_names, joints, armor),
        ParametricDesign::Garment(garment)
            if matches!(
                garment.kind,
                GarmentArmorKind::Fauld | GarmentArmorKind::Tassets
            ) =>
        {
            return waist(joint_names, joints, armor);
        }
        ParametricDesign::Garment(garment) if garment.kind == GarmentArmorKind::Gorget => {
            // A gorget is seated on the cuirass. Its formed bib and collar
            // sheets share that rigid support; nearby clavicle and neck skin
            // must not fold metal or move the bib through its lower plate.
            (RigJointName::C_SPINE3, &[ArmorComponentRole::Plate])
        }
        ParametricDesign::Helmet(HelmetDesign::ArmingCap(_) | HelmetDesign::MailCoif(_)) => {
            return Ok(());
        }
        ParametricDesign::Helmet(_) => {
            // Every helmet plate, visors and bevors included, is rigid.
            let head = joint_index(joint_names, &RigJointName::C_HEAD)?;
            fill(armor, 0..armor.positions.len(), head);
            return Ok(());
        }
        ParametricDesign::Limb(limb) => match limb {
            // Keep the fitted saddle and courses coherent until an equipment
            // rig supplies independent plate transforms.
            LimbArmorDesign::Pauldron(_) => (
                limb_joint(placement, RigJointPart::Uparm)?,
                &[
                    ArmorComponentRole::Plate,
                    ArmorComponentRole::JointExtension,
                ],
            ),
            LimbArmorDesign::Couter(_) => (
                limb_joint(placement, RigJointPart::Lowarm)?,
                &[
                    ArmorComponentRole::Plate,
                    ArmorComponentRole::JointExtension,
                ],
            ),
            LimbArmorDesign::Poleyn(_) => (
                limb_joint(placement, RigJointPart::Lowleg)?,
                &[
                    ArmorComponentRole::Plate,
                    ArmorComponentRole::JointExtension,
                ],
            ),
            LimbArmorDesign::Rerebrace(_) | LimbArmorDesign::Spaulder(_) => (
                limb_joint(placement, RigJointPart::Uparm)?,
                &[ArmorComponentRole::Plate],
            ),
            LimbArmorDesign::Cuisse(_) => (
                limb_joint(placement, RigJointPart::Upleg)?,
                &[ArmorComponentRole::Plate],
            ),
            _ => return Ok(()),
        },
        _ => return Ok(()),
    };
    let anchor = joint_index(joint_names, &joint)?;
    let besagews = armor
        .components
        .iter()
        .filter(|component| component.role == ArmorComponentRole::Besagew)
        .map(|component| component.vertices.clone())
        .collect::<Vec<_>>();
    if !besagews.is_empty() {
        let chest = joint_index(joint_names, &RigJointName::C_SPINE3)?;
        for vertices in besagews {
            fill(armor, vertices, chest);
        }
    }
    // A piece without components is one plate.
    if armor.components.is_empty() {
        fill(armor, 0..armor.positions.len(), anchor);
        return Ok(());
    }
    let plates = armor
        .components
        .iter()
        .filter(|component| roles.contains(&component.role))
        .map(|component| component.vertices.clone())
        .collect::<Vec<_>>();
    for vertices in plates {
        fill(armor, vertices, anchor);
    }
    Ok(())
}

/// Each separate sheet of a fauld follows the pelvis; each tasset, the hip
/// on its side of the pelvis.
fn waist(
    joint_names: &[RigJointName],
    joints: &[[f32; 8]],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let pelvis = joint_index(joint_names, &RigJointName::ROOT)?;
    let hips = [
        joint_index(joint_names, &RigJointName::L_UPLEG)?,
        joint_index(joint_names, &RigJointName::R_UPLEG)?,
    ];
    let center = joints
        .get(usize::from(pelvis))
        .context("missing pelvic joint state")?[0];
    let tassets = armor
        .components
        .iter()
        .filter(|component| component.role == ArmorComponentRole::Tassets)
        .map(|component| component.vertices.clone())
        .collect::<Vec<_>>();
    for sheet in sheets(&armor.indices, armor.positions.len()) {
        let anchor = if tassets.iter().any(|range| range.contains(&sheet[0])) {
            let middle =
                sheet.iter().map(|&i| armor.positions[i][0]).sum::<f32>() / sheet.len() as f32;
            // The wearer's left lies toward +x.
            hips[usize::from(middle < center)]
        } else {
            pelvis
        };
        for vertex in sheet {
            fill(armor, vertex..vertex + 1, anchor);
        }
    }
    Ok(())
}

/// The vertices of each connected sheet of triangles.
fn sheets(indices: &[u32], vertex_count: usize) -> Vec<Vec<usize>> {
    fn root(parents: &mut [usize], mut vertex: usize) -> usize {
        while parents[vertex] != vertex {
            parents[vertex] = parents[parents[vertex]];
            vertex = parents[vertex];
        }
        vertex
    }
    let mut parents = (0..vertex_count).collect::<Vec<_>>();
    for triangle in indices.as_chunks::<3>().0 {
        let a = root(&mut parents, triangle[0] as usize);
        for &corner in &triangle[1..] {
            let b = root(&mut parents, corner as usize);
            parents[b] = a;
        }
    }
    let mut sheets = std::collections::BTreeMap::<usize, Vec<usize>>::new();
    for vertex in 0..vertex_count {
        let sheet = root(&mut parents, vertex);
        sheets.entry(sheet).or_default().push(vertex);
    }
    sheets.into_values().collect()
}

fn limb_joint(placement: &str, joint: RigJointPart) -> Result<RigJointName> {
    Ok(Side::from_placement(placement)?.joint(joint))
}

fn joint_index(
    joint_names: &[RigJointName],
    name: &RigJointName,
) -> std::result::Result<RigJointOrdinal, RigJointLookupError> {
    name.require_in(joint_names)
}

fn fill(armor: &mut GeneratedArmor, vertices: Range<usize>, joint: RigJointOrdinal) {
    armor.joint_indices[vertices.clone()].fill([usize::from(joint) as u32; 8]);
    armor.joint_weights[vertices].fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
}

/// The cuirass follows the chest as one rigid assembly. Separate horizontal
/// courses retain their constructed overlaps. Independent course motion needs
/// armor pivots and lap constraints; body-spine joints are not those pivots.
pub(crate) fn breastplate(joint_names: &[RigJointName], armor: &mut GeneratedArmor) -> Result<()> {
    let chest = joint_index(joint_names, &RigJointName::C_SPINE3)?;
    fill(armor, 0..armor.positions.len(), chest);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_armor::{ArmorComponent, BurgonetDesign, CoifDesign, JointCupDesign};

    fn names() -> Vec<RigJointName> {
        [
            RigJointName::ROOT,
            RigJointName::C_HEAD,
            RigJointName::L_UPARM,
            RigJointName::R_LOWARM,
            RigJointName::L_LOWLEG,
            RigJointName::R_UPLEG,
            RigJointName::C_SPINE3,
        ]
        .to_vec()
    }

    fn armor(components: Vec<ArmorComponent>) -> GeneratedArmor {
        GeneratedArmor {
            design_hash: [0; 32],
            surface_domain: "test".into(),
            positions: vec![[0.0; 3]; 4],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            texcoords: vec![[0.0; 2]; 4],
            joint_indices: vec![[0, 1, 2, 3, 4, 5, 0, 0]; 4],
            joint_weights: vec![[0.2, 0.2, 0.2, 0.2, 0.1, 0.1, 0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 1, 3, 2],
            faces: Vec::new(),
            trim: None,
            grids: Vec::new(),
            morphs: Vec::new(),
            components,
        }
    }

    fn component(role: ArmorComponentRole, vertices: Range<usize>) -> ArmorComponent {
        ArmorComponent {
            role,
            vertices,
            indices: 0..0,
            hinge: None,
            mount: None,
            material: None,
        }
    }

    const RIGID: [f32; 8] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    #[test]
    fn gorget_sheets_share_the_cuirass_support_and_spare_flexible_components() {
        let mut gorget = armor(vec![
            component(ArmorComponentRole::Plate, 0..1),
            component(ArmorComponentRole::Plate, 1..3),
            component(ArmorComponentRole::LeatherStraps, 3..4),
        ]);
        let flexible = (gorget.joint_indices[3], gorget.joint_weights[3]);
        let design = ParametricDesign::Garment(fabelgeist_armor::GarmentArmorDesign::new(
            GarmentArmorKind::Gorget,
        ));
        attach(&design, "worn", &names(), &[], &mut gorget).unwrap();
        let mut cuirass = armor(Vec::new());
        breastplate(&names(), &mut cuirass).unwrap();
        assert!(
            gorget.joint_indices[..3]
                .iter()
                .all(|i| *i == cuirass.joint_indices[0])
        );
        assert_eq!(gorget.joint_weights[..3], [RIGID; 3]);
        assert_eq!((gorget.joint_indices[3], gorget.joint_weights[3]), flexible);
    }

    #[test]
    fn solid_helmets_follow_the_head_and_cloth_headwear_keeps_its_skin() {
        let mut burgonet = armor(Vec::new());
        let helmet = ParametricDesign::Helmet(HelmetDesign::Burgonet(BurgonetDesign::default()));
        attach(&helmet, "worn", &names(), &[], &mut burgonet).unwrap();
        assert!(burgonet.joint_indices.iter().all(|j| *j == [1; 8]));
        assert!(burgonet.joint_weights.iter().all(|w| *w == RIGID));
        let mut coif = armor(Vec::new());
        let unchanged = coif.joint_weights.clone();
        let design = ParametricDesign::Helmet(HelmetDesign::MailCoif(CoifDesign::default()));
        attach(&design, "worn", &names(), &[], &mut coif).unwrap();
        assert_eq!(coif.joint_weights, unchanged);
    }

    #[test]
    fn cops_follow_the_lower_limb_on_their_side_and_spare_other_components() {
        let mut couter = armor(vec![
            component(ArmorComponentRole::Plate, 0..2),
            component(ArmorComponentRole::LeatherStraps, 2..4),
        ]);
        let design = ParametricDesign::Limb(LimbArmorDesign::Couter(JointCupDesign::couter()));
        attach(&design, "right", &names(), &[], &mut couter).unwrap();
        assert_eq!(couter.joint_indices[..2], [[3; 8]; 2]);
        assert_ne!(couter.joint_weights[2], RIGID);
        let mut poleyn = armor(Vec::new());
        let design = ParametricDesign::Limb(LimbArmorDesign::Poleyn(JointCupDesign::poleyn()));
        attach(&design, "left", &names(), &[], &mut poleyn).unwrap();
        assert!(poleyn.joint_indices.iter().all(|j| *j == [4; 8]));
    }

    #[test]
    fn a_spaulder_follows_the_arm_and_its_besagew_the_chest() {
        let mut spaulder = armor(vec![
            component(ArmorComponentRole::Plate, 0..2),
            component(ArmorComponentRole::Besagew, 2..4),
        ]);
        let design = ParametricDesign::Limb(LimbArmorDesign::Spaulder(Default::default()));
        attach(&design, "left", &names(), &[], &mut spaulder).unwrap();
        assert_eq!(spaulder.joint_indices, [[2; 8], [2; 8], [6; 8], [6; 8]]);
        assert!(spaulder.joint_weights.iter().all(|w| *w == RIGID));
    }

    #[test]
    fn a_missing_anchor_is_an_error() {
        let mut poleyn = armor(Vec::new());
        let design = ParametricDesign::Limb(LimbArmorDesign::Poleyn(JointCupDesign::poleyn()));
        assert!(attach(&design, "right", &names(), &[], &mut poleyn).is_err());
    }

    #[test]
    fn fauld_lames_hang_from_the_pelvis_and_tassets_from_their_hip() {
        // Three separate sheets: a fauld lame, then a left and a right tasset.
        let mut waist = GeneratedArmor {
            positions: vec![
                [-0.1, 1.0, 0.0],
                [0.1, 1.0, 0.0],
                [0.0, 1.1, 0.0],
                [0.1, 0.9, 0.0],
                [0.2, 0.9, 0.0],
                [0.15, 0.8, 0.0],
                [-0.1, 0.9, 0.0],
                [-0.2, 0.9, 0.0],
                [-0.15, 0.8, 0.0],
            ],
            joint_indices: vec![[0; 8]; 9],
            joint_weights: vec![[0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 9],
            indices: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            components: vec![
                component(ArmorComponentRole::Fauld, 0..3),
                component(ArmorComponentRole::Tassets, 3..9),
            ],
            ..armor(Vec::new())
        };
        let names = [
            RigJointName::ROOT,
            RigJointName::L_UPLEG,
            RigJointName::R_UPLEG,
        ];
        let joints = [[0.0; 8]; 3];
        let design = ParametricDesign::WaistAssembly(fabelgeist_armor::WaistArmorDesign {
            fauld: fabelgeist_armor::GarmentArmorDesign::new(GarmentArmorKind::Fauld),
            tassets: fabelgeist_armor::GarmentArmorDesign::new(GarmentArmorKind::Tassets),
        });
        attach(&design, "worn", &names, &joints, &mut waist).unwrap();
        assert_eq!(waist.joint_indices[..3], [[0; 8]; 3]);
        assert_eq!(waist.joint_indices[3..6], [[1; 8]; 3]);
        assert_eq!(waist.joint_indices[6..], [[2; 8]; 3]);
        assert!(waist.joint_weights.iter().all(|w| *w == RIGID));
    }
}

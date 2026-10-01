//! Rigid plates follow one anatomical joint instead of the nearest skin.
//!
//! Solid helmets move with the head alone, so jaw and neck deformation cannot
//! bend them. Joint cops and their distal courses follow the lower limb, and
//! rerebraces, spaulders and cuisses the upper limb, so limb motion and twist
//! weights cannot shear their plates. Body proportions still refit every plate
//! through its morph targets. Each lame of a fauld hangs from the pelvis and
//! each tasset from its own hip; a besagew hangs from the chest in front of
//! the shoulder. Arming caps, mail coifs and textiles keep the body's flexible
//! skin.

use anyhow::{Context, Result};
use fabelgeist_armor::{
    ArmorComponentRole, GarmentArmorKind, GeneratedArmor, HelmetDesign, LimbArmorDesign,
};
use std::ops::Range;

use crate::armor_frames::Side;
use crate::armor_recipes::ParametricDesign;

/// Give `design`'s rigid plates their single anatomical owner. `joints` are
/// the wearer's global joint states, one per name.
pub fn attach(
    design: &ParametricDesign,
    placement: &str,
    joint_names: &[String],
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
        ParametricDesign::Helmet(HelmetDesign::ArmingCap(_) | HelmetDesign::MailCoif(_)) => {
            return Ok(());
        }
        ParametricDesign::Helmet(_) => {
            // Every helmet plate, visors and bevors included, is rigid.
            let head = joint_index(joint_names, "c_head")?;
            fill(armor, 0..armor.positions.len(), head);
            return Ok(());
        }
        ParametricDesign::Limb(limb) => match limb {
            LimbArmorDesign::Pauldron(_) => return pauldron(placement, joint_names, armor),
            LimbArmorDesign::Couter(_) => (
                limb_joint(placement, "lowarm")?,
                &[
                    ArmorComponentRole::Plate,
                    ArmorComponentRole::JointExtension,
                ],
            ),
            LimbArmorDesign::Poleyn(_) => (
                limb_joint(placement, "lowleg")?,
                &[
                    ArmorComponentRole::Plate,
                    ArmorComponentRole::JointExtension,
                ],
            ),
            LimbArmorDesign::Rerebrace(_) | LimbArmorDesign::Spaulder(_) => (
                limb_joint(placement, "uparm")?,
                &[ArmorComponentRole::Plate],
            ),
            LimbArmorDesign::Cuisse(_) => (
                limb_joint(placement, "upleg")?,
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
        let chest = joint_index(joint_names, "c_spine3")?;
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

/// The broad saddle hangs from the chest; distal lames follow the upper arm.
/// Each remains rigid. This ownership alone is not an overlap constraint.
fn pauldron(placement: &str, names: &[String], armor: &mut GeneratedArmor) -> Result<()> {
    let cap = joint_index(names, "c_spine3")?;
    let arm = joint_index(names, &limb_joint(placement, "uparm")?)?;
    let parts = armor
        .components
        .iter()
        .map(|part| (part.role, part.vertices.clone()))
        .collect::<Vec<_>>();
    for (role, vertices) in parts {
        fill(
            armor,
            vertices,
            if role == ArmorComponentRole::JointExtension {
                arm
            } else {
                cap
            },
        );
    }
    Ok(())
}

/// Each separate sheet of a fauld follows the pelvis; each tasset, the hip
/// on its side of the pelvis.
fn waist(joint_names: &[String], joints: &[[f32; 8]], armor: &mut GeneratedArmor) -> Result<()> {
    let pelvis = joint_index(joint_names, "root")?;
    let hips = [
        joint_index(joint_names, "l_upleg")?,
        joint_index(joint_names, "r_upleg")?,
    ];
    let center = joints
        .get(pelvis as usize)
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

fn limb_joint(placement: &str, joint: &str) -> Result<String> {
    Ok(format!(
        "{}_{joint}",
        Side::from_placement(placement)?.prefix()
    ))
}

fn joint_index(joint_names: &[String], name: &str) -> Result<u32> {
    Ok(joint_names
        .iter()
        .position(|joint| joint == name)
        .with_context(|| format!("missing rigid plate attachment joint {name}"))? as u32)
}

fn fill(armor: &mut GeneratedArmor, vertices: Range<usize>, joint: u32) {
    armor.joint_indices[vertices.clone()].fill([joint; 8]);
    armor.joint_weights[vertices].fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
}

/// The cuirass follows the chest as one rigid assembly. Separate horizontal
/// courses retain their constructed overlaps. Independent course motion needs
/// armor pivots and lap constraints; body-spine joints are not those pivots.
pub(crate) fn breastplate(joint_names: &[String], armor: &mut GeneratedArmor) -> Result<()> {
    let chest = joint_index(joint_names, "c_spine3")?;
    fill(armor, 0..armor.positions.len(), chest);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_armor::{ArmorComponent, BurgonetDesign, CoifDesign, JointCupDesign};

    fn names() -> Vec<String> {
        [
            "root", "c_head", "l_uparm", "r_lowarm", "l_lowleg", "r_upleg", "c_spine3",
        ]
        .map(String::from)
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
            material: None,
        }
    }

    const RIGID: [f32; 8] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

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
        let names = ["root", "l_upleg", "r_upleg"].map(String::from);
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

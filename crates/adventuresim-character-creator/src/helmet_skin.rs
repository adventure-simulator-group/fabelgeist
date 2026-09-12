//! Solid helmet plates follow the head, independently of jaw and neck skin.
use adventuresim_armor_model::{GeneratedArmor, HelmetDesign};
use anyhow::{Context, Result};

pub(super) fn attach(
    design: &HelmetDesign,
    joints: &[String],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    if matches!(
        design,
        HelmetDesign::ArmingCap(_) | HelmetDesign::MailCoif(_)
    ) {
        return Ok(());
    }
    let head = joints
        .iter()
        .position(|name| name == "c_head")
        .context("missing rigid helmet attachment joint")? as u32;
    armor.joint_indices.fill([head; 8]);
    armor
        .joint_weights
        .fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::*;

    fn armor(design: HelmetDesign) -> GeneratedArmor {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.085, 0.115, 0.105],
        };
        let mesh = generate_helmet(&design, &frame).unwrap();
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
            joint_indices: vec![[0, 2, 0, 0, 0, 0, 0, 0]; count],
            joint_weights: vec![[0.4, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; count],
            morphs: vec![ArmorMorph {
                name: "identity".into(),
                direct_positions: mesh.positions,
                position_deltas: vec![[0.0, 0.001, 0.0]; count],
                normal_deltas: vec![[0.0; 3]; count],
            }],
        }
    }

    #[test]
    fn solid_helmet_parts_share_head_skin_without_losing_hinges_or_morphs() {
        let names = ["c_jaw".into(), "c_head".into(), "c_neck_twist1_proc".into()];
        for kind in [
            HelmetKind::Morion,
            HelmetKind::KettleHat,
            HelmetKind::Barbute,
            HelmetKind::Burgonet,
            HelmetKind::Sallet,
            HelmetKind::VisoredSallet,
            HelmetKind::CloseHelmet,
        ] {
            let mut design = HelmetDesign::catalog(kind);
            if let HelmetDesign::Burgonet(burgonet) = &mut design {
                burgonet.buffe = Some(BuffeDesign::default());
            }
            let mut generated = armor(design);
            let original = generated.clone();
            attach(&design, &names, &mut generated).unwrap();
            assert!(
                generated
                    .joint_indices
                    .iter()
                    .all(|indices| indices[0] == 1)
            );
            assert!(
                generated
                    .joint_weights
                    .iter()
                    .all(|weights| weights[0] == 1.0 && weights[1..].iter().all(|w| *w == 0.0))
            );
            generated.joint_indices = original.joint_indices.clone();
            generated.joint_weights = original.joint_weights.clone();
            assert_eq!(
                generated, original,
                "attachment changed geometry, UVs, component/hinge ranges or morph correspondence"
            );
        }
    }

    #[test]
    fn flexible_head_coverings_keep_body_skin_and_solid_helmets_require_head_anchor() {
        for kind in [HelmetKind::ArmingCap, HelmetKind::MailCoif] {
            let design = HelmetDesign::catalog(kind);
            let mut generated = armor(design);
            let original = generated.clone();
            attach(&design, &[], &mut generated).unwrap();
            assert_eq!(generated, original);
        }
        let design = HelmetDesign::catalog(HelmetKind::Burgonet);
        let mut generated = armor(design);
        assert!(attach(&design, &["c_jaw".into()], &mut generated).is_err());
    }
}

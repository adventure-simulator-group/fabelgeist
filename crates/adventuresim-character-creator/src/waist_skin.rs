//! Plate attachments use the pelvis and primary limb joints, not skin twists.
use super::*;
use adventuresim_armor_model::ArmorComponentRole;

/// Pelvic skin transitions to the thigh over the upper 20 cm of the leg.
const PELVIS_ATTACHMENT_TRANSITION_M: f32 = 0.20;

pub(super) fn attach(
    model: &BodyModel,
    body: &GeneratedCharacter,
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let names = &model.mhr.character.skeleton.names;
    let joint = |name| {
        names
            .iter()
            .position(|n| n == name)
            .context("missing waist attachment joint")
    };
    let root = joint("root")?;
    let spine = joint("c_spine1")?;
    let hips = [joint("l_upleg")?, joint("r_upleg")?];
    let knees = [joint("l_lowleg")?, joint("r_lowleg")?];
    let center = body.global_joint_states[root][0];
    for component in &armor.components {
        let points = &armor.positions[component.vertices.clone()];
        let lo = points.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        let hi = points
            .iter()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        for index in component.vertices.clone() {
            let p = armor.positions[index];
            let side = usize::from(p[0] < center);
            let hip = hips[side];
            let knee = knees[side];
            let span = (body.global_joint_states[hip][0] - center).abs();
            let length = body.global_joint_states[hip][1] - body.global_joint_states[knee][1];
            anyhow::ensure!(
                span > 0.0 && length > 0.0 && hi > lo,
                "invalid waist anchors"
            );
            let lateral = ((p[0] - center).abs() / span).min(1.0);
            let (bones, weights) = match component.role {
                ArmorComponentRole::Fauld => {
                    let descent = ((hi - p[1]) / (hi - lo)).clamp(0.0, 1.0);
                    (
                        [spine, root, hip],
                        [1.0 - descent, descent * (1.0 - lateral), descent * lateral],
                    )
                }
                ArmorComponentRole::Tassets => {
                    let descent =
                        ((body.global_joint_states[hip][1] - p[1]) / length).clamp(0.0, 1.0);
                    let pelvic_blend = ((body.global_joint_states[hip][1]
                        + PELVIS_ATTACHMENT_TRANSITION_M * 0.5
                        - p[1])
                        / PELVIS_ATTACHMENT_TRANSITION_M)
                        .clamp(0.0, 1.0);
                    let lateral = lateral * pelvic_blend;
                    (
                        [root, hip, knee],
                        [1.0 - lateral, lateral * (1.0 - descent), lateral * descent],
                    )
                }
                _ => anyhow::bail!("unexpected waist plate component"),
            };
            armor.joint_indices[index] = [0; 8];
            armor.joint_weights[index] = [0.0; 8];
            for i in 0..3 {
                armor.joint_indices[index][i] = bones[i] as u32;
                armor.joint_weights[index][i] = weights[i];
            }
        }
    }
    Ok(())
}

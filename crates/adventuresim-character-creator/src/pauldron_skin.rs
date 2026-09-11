//! Blend the torso-facing wings into the shoulder attachment across its crown.
use super::*;
use adventuresim_character_creator::armor_frames::Side;

const CROWN_BLEND_START: f32 = 0.90;
const CROWN_BLEND_WIDTH: f32 = 0.25;

pub(super) fn attach(
    model: &BodyModel,
    body: &GeneratedCharacter,
    placement: &str,
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let arm_name = match Side::from_placement(placement)? {
        Side::Left => "l_uparm",
        Side::Right => "r_uparm",
    };
    let names = &model.mhr.character.skeleton.names;
    let joint = |name| {
        names
            .iter()
            .position(|n| n == name)
            .context("missing pauldron attachment joint")
    };
    let arm = joint(arm_name)?;
    let chest = joint("c_spine3")?;
    let center = body.global_joint_states[chest][0];
    let span = (body.global_joint_states[arm][0] - center).abs();
    anyhow::ensure!(span > 0.0, "shoulder must lie lateral to the chest anchor");
    for (i, point) in armor.positions.iter().enumerate() {
        let t = (((point[0] - center).abs() / span - CROWN_BLEND_START) / CROWN_BLEND_WIDTH)
            .clamp(0.0, 1.0);
        let shoulder = t * t * (3.0 - 2.0 * t);
        armor.joint_indices[i] = [arm as u32, chest as u32, 0, 0, 0, 0, 0, 0];
        armor.joint_weights[i] = [shoulder, 1.0 - shoulder, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    }
    Ok(())
}

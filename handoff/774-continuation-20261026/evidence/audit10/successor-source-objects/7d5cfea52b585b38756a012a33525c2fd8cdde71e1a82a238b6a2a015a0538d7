//! Supported-foot displacement from current captured native bone positions.

use super::*;
use adventuresim_tactical_core::animation::{AngularMagnitudeDegrees, NonnegativeLength};

pub(super) fn supported_foot_slip(
    metadata: &ScenarioMetadata<'_>,
    before: &FrameSample,
    after: &FrameSample,
) -> AnimationValueResult<Observation<NonnegativeLength>> {
    let mut maximum_slip: Option<NonnegativeLength> = None;
    let body_turn = AngularMagnitudeDegrees::from_degrees(Quat::from_array(before.body_rotation_xyzw)
        .angle_between(Quat::from_array(after.body_rotation_xyzw))
        .to_degrees())?;
    for (foot, support) in [
        (
            BoneRole::FootLeft,
            before
                .left_support_weight
                .fraction()
                .min(after.left_support_weight.fraction()),
        ),
        (
            BoneRole::FootRight,
            before
                .right_support_weight
                .fraction()
                .min(after.right_support_weight.fraction()),
        ),
    ] {
        // A body turn deliberately slides an old world plant onto
        // its new anatomical side corridor. Treat that bounded
        // adaptation separately from skating under a stable body.
        let procedural_plant_active =
            metadata.kind() == ScenarioKind::Terrain || (before.guard_action && after.guard_action);
        if !procedural_plant_active || support < 0.9 || body_turn.degrees() > 0.5 {
            continue;
        }
        let Some((a, b)) = before.bones.get(foot).zip(after.bones.get(foot)) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: foot }));
        };
        let distance = (Vec3::from_array(b.position) - Vec3::from_array(a.position)).xz().length();
        let distance = NonnegativeLength::from_arithmetic(distance)?;
        maximum_slip = Some(maximum_slip.map_or(distance, |maximum| maximum.max(distance)));
    }
    Ok(maximum_slip.map_or(Observation::NotApplicable, Observation::Measured))
}

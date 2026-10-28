//! Final standing leg constraints and rendered knee/foot yaw measurement.

use super::*;

#[derive(Clone, Copy)]
struct KneeYawLeg {
    upper: BoneRole,
    lower: BoneRole,
    foot: BoneRole,
    side: LeadFoot,
}

impl KneeYawLeg {
    const LEFT: Self = Self {
        upper: BoneRole::ThighLeft,
        lower: BoneRole::ShinLeft,
        foot: BoneRole::FootLeft,
        side: LeadFoot::Left,
    };
    const RIGHT: Self = Self {
        upper: BoneRole::ThighRight,
        lower: BoneRole::ShinRight,
        foot: BoneRole::FootRight,
        side: LeadFoot::Right,
    };

    fn constrain(
        self,
        rig: &HumanoidRig,
        frame: ObservedRigFrame,
        parents: &Query<&ChildOf>,
        transforms: &mut ParamSet<(TransformHelper, Query<&mut Transform>)>,
    ) -> AnimationValueResult<Option<AngularMagnitudeDegrees>> {
        let (Some(&upper), Some(&lower), Some(&foot)) = (
            rig.get(&self.upper),
            rig.get(&self.lower),
            rig.get(&self.foot),
        ) else {
            return Ok(None);
        };
        let Some((upper_snapshot, lower_snapshot, foot_snapshot)) =
            snapshot_chain(upper, lower, foot, parents, &transforms.p0())
        else {
            return Ok(None);
        };
        let hip = upper_snapshot.global.translation();
        let knee = lower_snapshot.global.translation();
        let target = foot_snapshot.global.translation();
        let segments = match LimbSegments::measure(hip, knee, target) {
            Ok(segments) => segments,
            Err(error) => {
                warn!(%error, "IK segment measurement rejected");
                return Ok(None);
            }
        };
        let leg_direction = (target - hip).normalize_or_zero();
        let side = anatomical_side(frame.rotation(), frame.origin(), hip, self.side);
        let canonical = pole_to_world(frame.rotation(), canonical_knee_pole(side));
        let current_bend = (knee - hip)
            .reject_from_normalized(leg_direction)
            .try_normalize()
            .unwrap_or(canonical);
        let pole = constrain_rendered_leg_pole(
            rig,
            self.side,
            hip,
            target,
            target,
            current_bend,
            parents,
            &transforms.p0(),
        );
        let Some(solution) = solve_two_bone_with_reach(
            TwoBoneChain::new(hip, knee, target, segments.upper, segments.lower, pole),
            target,
            maximum_reach(segments.upper, segments.lower),
        ) else {
            return Ok(None);
        };
        apply_two_bone_solution(upper, lower, foot, solution, parents, transforms);
        self.rendered_offset(rig, [upper, lower, foot], parents, &transforms.p0())
    }

    fn rendered_offset(
        self,
        rig: &HumanoidRig,
        [upper, lower, foot]: [Entity; 3],
        parents: &Query<&ChildOf>,
        transforms: &TransformHelper,
    ) -> AnimationValueResult<Option<AngularMagnitudeDegrees>> {
        if let Some((final_upper, final_lower, final_foot)) =
            snapshot_chain(upper, lower, foot, parents, transforms)
            && let Some(end_direction) =
                (final_foot.global.translation() - final_upper.global.translation()).try_normalize()
            && let Some(bend) = (final_lower.global.translation()
                - final_upper.global.translation())
            .reject_from_normalized(end_direction)
            .xz()
            .try_normalize()
            && let Some(facing) = rendered_foot_facing(
                rig,
                self.side,
                final_foot.global.translation(),
                parents,
                transforms,
            )
            .and_then(|facing| facing.xz().try_normalize())
        {
            return AngularMagnitudeDegrees::from_degrees(bend.angle_to(facing).abs().to_degrees())
                .map(Some);
        }
        Ok(None)
    }
}

/// Apply the standing foot-facing constraint after the earlier pose owners.
pub(in crate::animation) fn enforce_anatomical_knee_yaw(
    owners: Query<&PresentedSkeleton>,
    rigs: Query<(Entity, &HumanoidRig)>,
    parents: Query<&ChildOf>,
    mut states: Query<&mut LegIkState>,
    mut transforms: ParamSet<(TransformHelper, Query<&mut Transform>)>,
) {
    for (owner, rig) in &rigs {
        let Ok(skeleton) = owners.get(owner) else {
            continue;
        };
        // Downed and posture-transition poses retain their authored knee planes.
        if !anatomical_knee_yaw_posture_is_valid(skeleton) {
            continue;
        }
        let mut final_offsets = [AngularMagnitudeDegrees::ZERO; 2];
        let frame = match ObservedRigFrame::resolve(rig, &transforms.p0()) {
            Ok(Some(frame)) => frame,
            Ok(None) => continue,
            Err(error) => {
                warn!(%error, ?owner, "Rejected knee-yaw rig frame");
                continue;
            }
        };
        for (leg_index, leg) in [KneeYawLeg::LEFT, KneeYawLeg::RIGHT]
            .into_iter()
            .enumerate()
        {
            match leg.constrain(rig, frame, &parents, &mut transforms) {
                Ok(Some(offset)) => final_offsets[leg_index] = offset,
                Ok(None) => {}
                Err(error) => {
                    warn!(%error, "knee-foot yaw offset rejected");
                    continue;
                }
            }
        }
        if let Ok(mut state) = states.get_mut(owner) {
            state.0.left_knee_foot_yaw_offset_degrees = final_offsets[0];
            state.0.right_knee_foot_yaw_offset_degrees = final_offsets[1];
        }
    }
}

pub(in crate::animation::procedural) fn anatomical_knee_yaw_posture_is_valid(
    skeleton: &SkeletonState,
) -> bool {
    !skeleton.body().is_downed() && !skeleton.is_posture_transitioning() && !skeleton.is_quickstep()
}

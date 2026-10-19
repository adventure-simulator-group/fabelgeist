//! Bounded procedural dive rotation in the authored pelvis reference frame.

use super::*;

#[derive(derive_more::Debug, Clone, Copy)]
#[debug("DiveInfluence({:?})", self.0.fraction())]
struct DiveInfluence(adventuresim_tactical_core::animation::UnitInterval);

impl DiveInfluence {
    fn from_factor(factor: f32) -> AnimationValueResult<Self> {
        if !factor.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::DiveInfluence,
            });
        }
        adventuresim_tactical_core::animation::UnitInterval::from_fraction(factor.clamp(0.0, 1.0))
            .map(Self)
            .map_err(|error| error.for_quantity(AnimationQuantity::DiveInfluence))
    }
    fn fraction(self) -> f32 {
        self.0.fraction()
    }
}

/// Tilts the pelvis and therefore its complete descendant hierarchy toward
/// dive travel. Dive clips are upper-body-only; their root/pelvis/leg tracks
/// are masked out before this pass. The directional load supplies the lower
/// launch pose and blends to the authored ground contact only after impact.
pub(in crate::animation) fn apply_procedural_dive_lower_body(
    owners: Query<&PresentedSkeleton>,
    mut bones: Query<(&HumanoidBone, &AuthoredBindTransform, &mut Transform)>,
) {
    for (bone, bind, mut transform) in &mut bones {
        if bone.role != BoneRole::Pelvis {
            continue;
        }
        let Ok(skeleton) = owners.get(bone.owner) else {
            continue;
        };
        let Some(transition) = skeleton.posture_transition() else {
            continue;
        };
        let PostureTransitionKind::DiveToDowned { direction, .. } = transition.kind() else {
            continue;
        };
        let phase = transition.phase().clamp(0.0, 1.0);
        let amount = if phase <= DIVE_GROUND_CONTACT_PHASE {
            smoothstep(0.0, DIVE_GROUND_CONTACT_PHASE, phase)
        } else {
            1.0 - smoothstep(DIVE_GROUND_CONTACT_PHASE, 1.0, phase)
        };
        let amount = match DiveInfluence::from_factor(amount) {
            Ok(amount) => amount,
            Err(error) => {
                warn!(%error, "Rejected dive pelvis contribution");
                continue;
            }
        };
        transform.rotation = procedural_dive_pelvis_rotation(
            transform.rotation,
            bind.local.rotation,
            direction,
            amount,
        );
    }
}

fn dive_pelvis_lean(direction: DiveDirection, amount: DiveInfluence) -> Quat {
    let angle = procedural_tuning().dive_pelvis_lean_degrees.radians() * amount.fraction();
    match direction {
        DiveDirection::Forward => Quat::from_rotation_x(angle),
        DiveDirection::Backward => Quat::from_rotation_x(-angle),
        DiveDirection::Left => Quat::from_rotation_z(-angle),
        DiveDirection::Right => Quat::from_rotation_z(angle),
    }
}

fn procedural_dive_pelvis_rotation(
    authored: Quat,
    bind: Quat,
    direction: DiveDirection,
    amount: DiveInfluence,
) -> Quat {
    let forward_facing = authored.slerp(bind, amount.fraction()).normalize();
    (dive_pelvis_lean(direction, amount) * forward_facing).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dive_pelvis_lean_tips_up_axis_exactly_toward_travel() {
        let forward = dive_pelvis_lean(
            DiveDirection::Forward,
            DiveInfluence::from_factor(1.0).unwrap(),
        ) * Vec3::Y;
        let backward = dive_pelvis_lean(
            DiveDirection::Backward,
            DiveInfluence::from_factor(1.0).unwrap(),
        ) * Vec3::Y;
        let left = dive_pelvis_lean(
            DiveDirection::Left,
            DiveInfluence::from_factor(1.0).unwrap(),
        ) * Vec3::Y;
        let right = dive_pelvis_lean(
            DiveDirection::Right,
            DiveInfluence::from_factor(1.0).unwrap(),
        ) * Vec3::Y;
        assert!(forward.z > 0.64 && forward.y > 0.76);
        assert!(backward.z < -0.64 && backward.y > 0.76);
        assert!(left.x > 0.64 && left.y > 0.76);
        assert!(right.x < -0.64 && right.y > 0.76);
    }

    #[test]
    fn airborne_dive_pelvis_faces_forward_independently_of_guard_rotation() {
        let bind = Quat::from_rotation_y(0.17);
        let guard = Quat::from_euler(EulerRot::YXZ, -0.8, 0.1, -0.2);
        for direction in [
            DiveDirection::Forward,
            DiveDirection::Backward,
            DiveDirection::Left,
            DiveDirection::Right,
        ] {
            let actual = procedural_dive_pelvis_rotation(
                guard,
                bind,
                direction,
                DiveInfluence::from_factor(1.0).unwrap(),
            );
            let expected = (dive_pelvis_lean(direction, DiveInfluence::from_factor(1.0).unwrap())
                * bind)
                .normalize();
            assert!(actual.angle_between(expected) < 0.0001);
        }
    }

    #[test]
    fn dive_pelvis_override_releases_exactly_to_authored_contact() {
        let contact = Quat::from_euler(EulerRot::YXZ, 0.3, -1.2, 0.4);
        let actual = procedural_dive_pelvis_rotation(
            contact,
            Quat::IDENTITY,
            DiveDirection::Backward,
            DiveInfluence::from_factor(0.0).unwrap(),
        );
        assert!(actual.angle_between(contact) < 0.0001);
    }
}

//! Bouncy bones: additive rotational hit springs, client-side.
//!
//! Impacts kick a per-bone damped rotational spring instead of a ragdoll:
//! each reactive bone carries a world-space rotation-vector offset plus
//! angular velocity, and a [`BoneKick`] converts a world-space impulse into
//! spin on the struck bone (`bone_direction × impulse`: a shove at the tip
//! rotates the bone the way the shove points). From there the reaction
//! travels the authored chain as a wave: every frame a deflected bone torques
//! its parent, so the hand snaps first and the shoulder answers a few frames
//! later, cloth-like. Purely cosmetic: nothing syncs and nothing touches
//! physics.
//!
//! The springs advance with the closed-form damped-oscillator solution
//! (amplitude and phase, not stepped Euler), so they are unconditionally
//! stable at any render delta and tuned in honest units: envelope half-life
//! and wobble frequency.
//!
//! Deltas are applied during a fresh parent-before-child walk of the rig
//! with world rotations accumulated on the way down; the cached
//! `GlobalTransform` still holds last frame's deltas and would misdirect the
//! applied axis.

use adventuresim_core::body::BodyPart;
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::{
    HumanoidBone, HumanoidRig, pose_buffer::PoseBufferRig, procedural::BoneRole,
    ragdoll::Ragdolling,
};

fn bounce_tuning() -> BouncyBonesConfig {
    runtime_animation_config().bouncy_bones
}

/// Knock a bone; the chain coupling ripples it toward the pelvis over the
/// next frames. `impulse` is a world-space shove at the bone tip in metres
/// per second of tip velocity to impart.
#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct BoneKick {
    /// The rig owner (the entity holding [`HumanoidRig`] and [`BouncyBones`]).
    pub(crate) owner: Entity,
    pub(crate) role: BoneRole,
    pub(crate) impulse: Vec3,
}

/// The reactive bone a server-reported body-part hit lands on.
pub(crate) fn impact_bone_role(part: BodyPart) -> BoneRole {
    match part {
        BodyPart::Head => BoneRole::Head,
        BodyPart::Chest => BoneRole::Chest,
        BodyPart::Stomach => BoneRole::StomachTwo,
        BodyPart::LeftArm => BoneRole::ForearmLeft,
        BodyPart::RightArm => BoneRole::ForearmRight,
        BodyPart::LeftLeg => BoneRole::ShinLeft,
        BodyPart::RightLeg => BoneRole::ShinRight,
    }
}

struct ReactiveBone {
    role: BoneRole,
    /// Index into [`REACTIVE_BONES`] the reaction propagates to next.
    parent: Option<usize>,
    /// Role whose position marks this bone's tip; `None` extends the line
    /// from the chain parent through this bone.
    tip: Option<BoneRole>,
    /// Inverse-mass feel: how much spin one unit of impulse buys. Hands
    /// flap, the pelvis barely nods.
    response: f32,
}

const fn bone(
    role: BoneRole,
    parent: Option<usize>,
    tip: Option<BoneRole>,
    response: f32,
) -> ReactiveBone {
    ReactiveBone {
        role,
        parent,
        tip,
        response,
    }
}

/// The reactive skeleton. Parent links are the coupling chain (anatomy, not
/// necessarily the exact transform hierarchy). Children inherit their
/// parent's wobble through ordinary transform propagation; this chain only
/// carries reactions upstream. Feet stay out: the terrain contact solver
/// owns them.
const REACTIVE_BONES: [ReactiveBone; 19] = [
    bone(BoneRole::Pelvis, None, Some(BoneRole::StomachOne), 0.15),
    bone(
        BoneRole::StomachOne,
        Some(0),
        Some(BoneRole::StomachTwo),
        0.2,
    ),
    bone(
        BoneRole::StomachTwo,
        Some(1),
        Some(BoneRole::StomachThree),
        0.25,
    ),
    bone(BoneRole::StomachThree, Some(2), Some(BoneRole::Chest), 0.3),
    bone(BoneRole::Chest, Some(3), Some(BoneRole::NeckOne), 0.35),
    bone(BoneRole::NeckOne, Some(4), Some(BoneRole::Head), 0.6),
    bone(BoneRole::Head, Some(5), None, 0.8),
    bone(
        BoneRole::ClavicleLeft,
        Some(4),
        Some(BoneRole::UpperArmLeft),
        0.4,
    ),
    bone(
        BoneRole::UpperArmLeft,
        Some(7),
        Some(BoneRole::ForearmLeft),
        0.6,
    ),
    bone(
        BoneRole::ForearmLeft,
        Some(8),
        Some(BoneRole::HandLeft),
        0.9,
    ),
    bone(BoneRole::HandLeft, Some(9), None, 1.3),
    bone(
        BoneRole::ClavicleRight,
        Some(4),
        Some(BoneRole::UpperArmRight),
        0.4,
    ),
    bone(
        BoneRole::UpperArmRight,
        Some(11),
        Some(BoneRole::ForearmRight),
        0.6,
    ),
    bone(
        BoneRole::ForearmRight,
        Some(12),
        Some(BoneRole::HandRight),
        0.9,
    ),
    bone(BoneRole::HandRight, Some(13), None, 1.3),
    bone(BoneRole::ThighLeft, Some(0), Some(BoneRole::ShinLeft), 0.4),
    bone(BoneRole::ShinLeft, Some(15), Some(BoneRole::FootLeft), 0.6),
    bone(
        BoneRole::ThighRight,
        Some(0),
        Some(BoneRole::ShinRight),
        0.4,
    ),
    bone(
        BoneRole::ShinRight,
        Some(17),
        Some(BoneRole::FootRight),
        0.6,
    ),
];

const BONE_COUNT: usize = REACTIVE_BONES.len();

fn reactive_bone(index: usize) -> &'static ReactiveBone {
    &REACTIVE_BONES[index]
}

fn bone_index(role: BoneRole) -> Option<usize> {
    (0..BONE_COUNT).find(|&index| reactive_bone(index).role == role)
}

/// World-space rotation-vector offset (direction is the axis, length the
/// angle in radians) plus its angular velocity. World-space so kicks land
/// without knowing the animated local frame; a fast-turning character drags
/// a dying wobble a few degrees off-axis for a fraction of a second.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
struct Spring {
    offset: Vec3,
    velocity: Vec3,
}

impl Spring {
    fn at_rest(&self) -> bool {
        self.offset == Vec3::ZERO && self.velocity == Vec3::ZERO
    }
}

/// Lives on the rig owner next to [`HumanoidRig`].
#[derive(Component, Default, Debug)]
pub(crate) struct BouncyBones {
    springs: [Spring; BONE_COUNT],
}

impl BouncyBones {
    fn active_springs(&self) -> u32 {
        self.springs
            .iter()
            .filter(|spring| !spring.at_rest())
            .count() as u32
    }

    fn maximum_deflection_radians(&self) -> f32 {
        self.springs
            .iter()
            .map(|spring| spring.offset.length())
            .fold(0.0, f32::max)
    }
}

/// Per-frame summary for capture tools.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub(crate) struct BouncyBonesTelemetry {
    pub(crate) active_springs: u32,
    pub(crate) maximum_deflection_radians: f32,
}

/// Exact damped-oscillator step, solved per axis: the equation is linear and
/// isotropic, so the scalar solution applies to the whole vector. The
/// damping comes from the envelope half-life and the frequency from the
/// tuned wobble rate; the damped frequency is clamped so an overdamped
/// request degrades to near-critical instead of producing a NaN.
fn spring_step(spring: &mut Spring, halflife: f32, frequency: f32, delta_seconds: f32) {
    let decay = std::f32::consts::LN_2 / halflife.max(1e-3);
    let omega = std::f32::consts::TAU * frequency.max(0.01);
    let damped = (omega * omega - decay * decay).max(1e-4).sqrt();
    let envelope = (-decay * delta_seconds).exp();
    let (sin, cos) = (damped * delta_seconds).sin_cos();
    let b = (spring.velocity + spring.offset * decay) / damped;
    let offset = (spring.offset * cos + b * sin) * envelope;
    let velocity = (spring.velocity * cos - (spring.offset * damped + b * decay) * sin) * envelope;
    // Snap the tail to true zero so at-rest rigs skip the hierarchy walk.
    if offset.length_squared() < 1e-8 && velocity.length_squared() < 1e-8 {
        *spring = Spring::default();
    } else {
        *spring = Spring { offset, velocity };
    }
}

/// The world-space direction a reactive bone points, from its tip role or,
/// for a chain end, continued from its chain parent.
fn bone_direction(
    index: usize,
    rig: &HumanoidRig,
    globals: &Query<&GlobalTransform>,
) -> Option<Vec3> {
    let definition = reactive_bone(index);
    let position = |role: BoneRole| {
        rig.get(&role)
            .and_then(|entity| globals.get(*entity).ok())
            .map(GlobalTransform::translation)
    };
    let origin = position(definition.role)?;
    let direction = match definition.tip {
        Some(tip) => position(tip)? - origin,
        None => origin - position(reactive_bone(definition.parent?).role)?,
    };
    direction.try_normalize()
}

pub(super) fn kick_bones(
    mut kicks: MessageReader<BoneKick>,
    mut rigs: Query<(&HumanoidRig, &mut BouncyBones)>,
    globals: Query<&GlobalTransform>,
) {
    let tuning = bounce_tuning();
    for kick in kicks.read() {
        let Ok((rig, mut bones)) = rigs.get_mut(kick.owner) else {
            continue;
        };
        let Some(start) = bone_index(kick.role) else {
            continue;
        };
        let mut index = start;
        let mut gain = tuning.strength;
        loop {
            if let Some(direction) = bone_direction(index, rig, &globals) {
                // `direction × impulse` is the angular kick that swings the
                // tip the way the shove goes.
                let spin = direction.cross(kick.impulse);
                bones.springs[index].velocity += spin * (reactive_bone(index).response * gain);
            }
            gain *= tuning.transmission;
            match reactive_bone(index).parent {
                Some(parent) if gain > 0.02 => index = parent,
                _ => break,
            }
        }
    }
}

/// World rotation of `entity` from the live `Transform` chain.
fn live_world_rotation(
    entity: Entity,
    parents: &Query<&ChildOf>,
    transforms: &Query<&mut Transform>,
) -> Quat {
    let mut rotation = Quat::IDENTITY;
    let mut current = Some(entity);
    while let Some(entity) = current {
        if let Ok(transform) = transforms.get(entity) {
            rotation = transform.rotation * rotation;
        }
        current = parents.get(entity).ok().map(ChildOf::parent);
    }
    rotation
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query pairs each rig owner with its spring state and pose ownership"
)]
pub(super) fn apply_bouncy_bones(
    time: Res<Time>,
    mut telemetry: ResMut<BouncyBonesTelemetry>,
    mut rigs: Query<(
        Entity,
        &HumanoidRig,
        &mut BouncyBones,
        Option<&PoseBufferRig>,
        Has<Ragdolling>,
    )>,
    bones: Query<&HumanoidBone>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    mut transforms: Query<&mut Transform>,
) {
    *telemetry = BouncyBonesTelemetry::default();
    let delta_seconds = time.delta_secs();
    if delta_seconds <= 0.0 {
        return;
    }
    let tuning = bounce_tuning();
    for (owner, rig, mut springs, pose_rig, ragdolling) in &mut rigs {
        // Wave coupling: a deflected bone torques its chain parent, scaled by
        // the parent's own response, so the kick takes several frames to
        // walk the chain one link at a time instead of the whole arm
        // starting at once. One-way up; the return trip rides transform
        // inheritance.
        let deflections: [Vec3; BONE_COUNT] = std::array::from_fn(|i| springs.springs[i].offset);
        for (index, deflection) in deflections.iter().enumerate() {
            if let Some(parent) = reactive_bone(index).parent
                && *deflection != Vec3::ZERO
            {
                springs.springs[parent].velocity += *deflection
                    * (tuning.coupling_per_second_squared
                        * reactive_bone(parent).response
                        * delta_seconds);
            }
        }
        let mut active = false;
        for spring in &mut springs.springs {
            if spring.at_rest() {
                continue;
            }
            spring_step(
                spring,
                tuning.halflife_seconds,
                tuning.frequency_hz,
                delta_seconds,
            );
            spring.offset = spring.offset.clamp_length_max(tuning.maximum_angle_radians);
            active = true;
        }
        telemetry.active_springs += springs.active_springs();
        telemetry.maximum_deflection_radians = telemetry
            .maximum_deflection_radians
            .max(springs.maximum_deflection_radians());
        // Springs keep decaying while ragdolled or culled so a rig that gets
        // back up does not replay a stale wobble; only the transform walk is
        // skipped, because those bones belong to physics or hold a frozen
        // pose.
        if !active || ragdolling || pose_rig.is_some_and(PoseBufferRig::is_frozen) {
            continue;
        }
        let Some(rig_scene) = rig.rig_scene() else {
            continue;
        };
        let base = parents
            .get(rig_scene)
            .map(|parent| live_world_rotation(parent.parent(), &parents, &transforms))
            .unwrap_or(Quat::IDENTITY);
        let mut stack: Vec<(Entity, Quat)> = vec![(rig_scene, base)];
        while let Some((entity, parent_rotation)) = stack.pop() {
            let local_rotation = transforms
                .get(entity)
                .map(|transform| transform.rotation)
                .unwrap_or(Quat::IDENTITY);
            let mut world_rotation = parent_rotation * local_rotation;
            if let Ok(bone) = bones.get(entity)
                && bone.owner == owner
                && let Some(index) = bone_index(bone.role)
            {
                let offset = springs.springs[index].offset;
                if offset.length_squared() > 1e-8
                    && let Ok(mut transform) = transforms.get_mut(entity)
                {
                    let local_delta = world_rotation.inverse() * offset;
                    transform.rotation *= Quat::from_scaled_axis(local_delta);
                    world_rotation = parent_rotation * transform.rotation;
                }
            }
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter().map(|kid| (kid, world_rotation)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reactive_chain_indices_point_at_earlier_bones_and_end_at_the_pelvis() {
        for index in 0..BONE_COUNT {
            let definition = reactive_bone(index);
            match definition.parent {
                Some(parent) => assert!(parent < index, "{:?}", definition.role),
                None => assert_eq!(definition.role, BoneRole::Pelvis),
            }
        }
        assert_eq!(bone_index(BoneRole::ShinRight), Some(BONE_COUNT - 1));
        assert_eq!(bone_index(BoneRole::FootLeft), None);
    }

    #[test]
    fn every_hit_body_part_lands_on_a_reactive_bone() {
        for part in [
            BodyPart::Head,
            BodyPart::Chest,
            BodyPart::Stomach,
            BodyPart::LeftArm,
            BodyPart::RightArm,
            BodyPart::LeftLeg,
            BodyPart::RightLeg,
        ] {
            assert!(bone_index(impact_bone_role(part)).is_some(), "{part:?}");
        }
    }

    #[test]
    fn spring_decays_toward_rest_and_snaps_to_exact_zero() {
        let mut spring = Spring {
            offset: Vec3::X * 0.5,
            velocity: Vec3::ZERO,
        };
        let mut peak_after_half_cycle = 0.0_f32;
        for step in 0..600 {
            spring_step(&mut spring, 0.18, 3.0, 1.0 / 120.0);
            if step > 20 {
                peak_after_half_cycle = peak_after_half_cycle.max(spring.offset.length());
            }
        }
        assert!(peak_after_half_cycle < 0.5);
        assert!(spring.at_rest());
    }

    #[test]
    fn spring_step_is_stable_for_large_render_deltas() {
        let mut spring = Spring {
            offset: Vec3::Y * 0.8,
            velocity: Vec3::Z * 40.0,
        };
        spring_step(&mut spring, 0.18, 3.0, 0.5);
        assert!(spring.offset.is_finite());
        assert!(spring.offset.length() < 0.8);
        spring_step(&mut spring, 0.001, 0.0, 2.0);
        assert!(spring.at_rest());
    }
}

//! The articulated ragdoll a character's proportioned bind pose defines.
//!
//! Conventions, shared with the entry seed and the pose readback: every body
//! rests at identity rotation in owner space at its driving bone's bind
//! position, so `bone = body * bind_rotation`; joint anchors and axes are
//! owner-space bind data; and the runtime multiplies every length by the
//! owner's live scale.

use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::super::procedural::BoneRole;

/// Index of the pelvis body: the kinematic root that rides the
/// authoritative controller.
pub(super) const PELVIS_BODY: usize = 0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CapsuleShape {
    pub(super) radius: f32,
    /// Cylinder length between the hemispherical caps.
    pub(super) length: f32,
}

#[derive(Debug, Clone)]
pub(super) struct RagdollBody {
    pub(super) role: BoneRole,
    /// Owner-space bind transform of the driving bone; the body rests here
    /// with identity rotation.
    pub(super) bind: Transform,
    pub(super) shape: CapsuleShape,
    /// Collider transform relative to the body, owner-space units.
    pub(super) collider_local: Transform,
    pub(super) kinematic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum RagdollJointKind {
    Spherical {
        twist_axis: Vec3,
        swing: f32,
        twist: f32,
    },
    Hinge {
        axis: Vec3,
        limits: (f32, f32),
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RagdollJoint {
    pub(super) parent: usize,
    pub(super) child: usize,
    /// Owner-space anchor at bind; body-local anchors are relative to each
    /// body's bind position.
    pub(super) anchor: Vec3,
    pub(super) kind: RagdollJointKind,
}

#[derive(Debug, Clone)]
pub(super) struct RagdollDefinition {
    pub(super) bodies: Vec<RagdollBody>,
    pub(super) joints: Vec<RagdollJoint>,
}

#[derive(Clone, Copy)]
enum CapsuleSelector {
    Pelvis,
    Chest,
    Head,
    Thigh,
    Shin,
    Foot,
    UpperArm,
    Forearm,
    Hand,
}

impl CapsuleSelector {
    fn select(self, capsules: &RagdollCapsulesConfig) -> RagdollCapsuleConfig {
        match self {
            Self::Pelvis => capsules.pelvis,
            Self::Chest => capsules.chest,
            Self::Head => capsules.head,
            Self::Thigh => capsules.thigh,
            Self::Shin => capsules.shin,
            Self::Foot => capsules.foot,
            Self::UpperArm => capsules.upper_arm,
            Self::Forearm => capsules.forearm,
            Self::Hand => capsules.hand,
        }
    }
}

/// A direction read from two bind positions, with a fallback for rigs that
/// lack the far joint.
#[derive(Clone, Copy)]
struct BindAxis {
    from: BoneRole,
    to: BoneRole,
    fallback: Vec3,
}

struct BodySpec {
    role: BoneRole,
    capsule: CapsuleSelector,
    /// The capsule runs from the bone along this axis.
    axis: BindAxis,
    kinematic: bool,
}

enum AnchorAt {
    Child,
    Midpoint,
    /// A joint between parent and child bodies, falling back to the child.
    Role(BoneRole),
}

enum JointSpecKind {
    Spherical {
        twist: BindAxis,
        swing_limit: f32,
        twist_limit: f32,
    },
    Hinge {
        bone: BindAxis,
        flex: FlexDirection,
        limits: (f32, f32),
    },
}

/// Which way a hinged limb folds, in character terms.
#[derive(Clone, Copy)]
enum FlexDirection {
    Forward,
    Backward,
}

struct JointSpec {
    parent: BoneRole,
    child: BoneRole,
    anchor: AnchorAt,
    kind: JointSpecKind,
}

const fn axis(from: BoneRole, to: BoneRole, fallback: Vec3) -> BindAxis {
    BindAxis { from, to, fallback }
}

const fn body(role: BoneRole, capsule: CapsuleSelector, axis: BindAxis) -> BodySpec {
    BodySpec {
        role,
        capsule,
        axis,
        kinematic: false,
    }
}

const fn spherical(
    parent: BoneRole,
    child: BoneRole,
    anchor: AnchorAt,
    twist: BindAxis,
    swing_limit: f32,
    twist_limit: f32,
) -> JointSpec {
    JointSpec {
        parent,
        child,
        anchor,
        kind: JointSpecKind::Spherical {
            twist,
            swing_limit,
            twist_limit,
        },
    }
}

const fn hinge(
    parent: BoneRole,
    child: BoneRole,
    bone: BindAxis,
    flex: FlexDirection,
    limits: (f32, f32),
) -> JointSpec {
    JointSpec {
        parent,
        child,
        anchor: AnchorAt::Child,
        kind: JointSpecKind::Hinge { bone, flex, limits },
    }
}

use BoneRole::*;
use CapsuleSelector as Capsule;

const BODY_SPECS: [BodySpec; 15] = [
    BodySpec {
        kinematic: true,
        ..body(Pelvis, Capsule::Pelvis, axis(Pelvis, Chest, Vec3::Y))
    },
    body(Chest, Capsule::Chest, axis(Chest, Head, Vec3::Y)),
    body(Head, Capsule::Head, axis(Chest, Head, Vec3::Y)),
    body(
        ThighLeft,
        Capsule::Thigh,
        axis(ThighLeft, ShinLeft, Vec3::NEG_Y),
    ),
    body(
        ShinLeft,
        Capsule::Shin,
        axis(ShinLeft, FootLeft, Vec3::NEG_Y),
    ),
    body(
        FootLeft,
        Capsule::Foot,
        axis(FootLeft, ToeLeft, Vec3::NEG_Z),
    ),
    body(
        ThighRight,
        Capsule::Thigh,
        axis(ThighRight, ShinRight, Vec3::NEG_Y),
    ),
    body(
        ShinRight,
        Capsule::Shin,
        axis(ShinRight, FootRight, Vec3::NEG_Y),
    ),
    body(
        FootRight,
        Capsule::Foot,
        axis(FootRight, ToeRight, Vec3::NEG_Z),
    ),
    body(
        UpperArmLeft,
        Capsule::UpperArm,
        axis(UpperArmLeft, ForearmLeft, Vec3::NEG_Y),
    ),
    body(
        ForearmLeft,
        Capsule::Forearm,
        axis(ForearmLeft, HandLeft, Vec3::NEG_Y),
    ),
    body(
        HandLeft,
        Capsule::Hand,
        axis(ForearmLeft, HandLeft, Vec3::NEG_Y),
    ),
    body(
        UpperArmRight,
        Capsule::UpperArm,
        axis(UpperArmRight, ForearmRight, Vec3::NEG_Y),
    ),
    body(
        ForearmRight,
        Capsule::Forearm,
        axis(ForearmRight, HandRight, Vec3::NEG_Y),
    ),
    body(
        HandRight,
        Capsule::Hand,
        axis(ForearmRight, HandRight, Vec3::NEG_Y),
    ),
];

const JOINT_SPECS: [JointSpec; 14] = [
    spherical(
        Pelvis,
        Chest,
        AnchorAt::Midpoint,
        axis(Chest, Head, Vec3::Y),
        0.5,
        0.4,
    ),
    spherical(
        Chest,
        Head,
        AnchorAt::Role(NeckOne),
        axis(Chest, Head, Vec3::Y),
        0.6,
        0.8,
    ),
    spherical(
        Pelvis,
        ThighLeft,
        AnchorAt::Child,
        axis(ThighLeft, ShinLeft, Vec3::NEG_Y),
        0.9,
        0.4,
    ),
    hinge(
        ThighLeft,
        ShinLeft,
        axis(ShinLeft, FootLeft, Vec3::NEG_Y),
        FlexDirection::Backward,
        (0.0, 2.3),
    ),
    hinge(
        ShinLeft,
        FootLeft,
        axis(ShinLeft, FootLeft, Vec3::NEG_Y),
        FlexDirection::Backward,
        (-0.5, 0.5),
    ),
    spherical(
        Pelvis,
        ThighRight,
        AnchorAt::Child,
        axis(ThighRight, ShinRight, Vec3::NEG_Y),
        0.9,
        0.4,
    ),
    hinge(
        ThighRight,
        ShinRight,
        axis(ShinRight, FootRight, Vec3::NEG_Y),
        FlexDirection::Backward,
        (0.0, 2.3),
    ),
    hinge(
        ShinRight,
        FootRight,
        axis(ShinRight, FootRight, Vec3::NEG_Y),
        FlexDirection::Backward,
        (-0.5, 0.5),
    ),
    spherical(
        Chest,
        UpperArmLeft,
        AnchorAt::Child,
        axis(UpperArmLeft, ForearmLeft, Vec3::NEG_Y),
        1.5,
        0.6,
    ),
    hinge(
        UpperArmLeft,
        ForearmLeft,
        axis(ForearmLeft, HandLeft, Vec3::NEG_Y),
        FlexDirection::Forward,
        (0.0, 2.5),
    ),
    spherical(
        ForearmLeft,
        HandLeft,
        AnchorAt::Child,
        axis(ForearmLeft, HandLeft, Vec3::NEG_Y),
        0.6,
        0.4,
    ),
    spherical(
        Chest,
        UpperArmRight,
        AnchorAt::Child,
        axis(UpperArmRight, ForearmRight, Vec3::NEG_Y),
        1.5,
        0.6,
    ),
    hinge(
        UpperArmRight,
        ForearmRight,
        axis(ForearmRight, HandRight, Vec3::NEG_Y),
        FlexDirection::Forward,
        (0.0, 2.5),
    ),
    spherical(
        ForearmRight,
        HandRight,
        AnchorAt::Child,
        axis(ForearmRight, HandRight, Vec3::NEG_Y),
        0.6,
        0.4,
    ),
];

/// Owner-space character forward, read from the feet so the definition does
/// not assume which way the authored rig faces; the controller convention
/// (`-Z`) is the fallback.
fn character_forward(bind: &impl Fn(BoneRole) -> Option<Transform>) -> Vec3 {
    let measured = [(FootLeft, ToeLeft), (FootRight, ToeRight)]
        .into_iter()
        .filter_map(|(foot, toe)| {
            let toe = bind(toe)?.translation;
            let foot = bind(foot)?.translation;
            Vec3::new(toe.x - foot.x, 0.0, toe.z - foot.z).try_normalize()
        })
        .sum::<Vec3>();
    measured.try_normalize().unwrap_or(Vec3::NEG_Z)
}

impl BindAxis {
    fn resolve(self, bind: &impl Fn(BoneRole) -> Option<Transform>) -> Vec3 {
        bind(self.from)
            .zip(bind(self.to))
            .and_then(|(from, to)| (to.translation - from.translation).try_normalize())
            .unwrap_or(self.fallback)
    }
}

impl RagdollDefinition {
    /// Build the ragdoll from a character's owner-space bind transforms.
    /// `None` when a driving bone is missing from the rig.
    pub(super) fn from_bind_pose(
        bind: impl Fn(BoneRole) -> Option<Transform>,
        capsules: &RagdollCapsulesConfig,
    ) -> Option<Self> {
        let forward = character_forward(&bind);
        let bodies = BODY_SPECS
            .iter()
            .map(|spec| {
                let bind_transform = bind(spec.role)?;
                let capsule = spec.capsule.select(capsules);
                let direction = spec.axis.resolve(&bind);
                Some(RagdollBody {
                    role: spec.role,
                    bind: bind_transform,
                    shape: CapsuleShape {
                        radius: capsule.radius_metres,
                        length: capsule.length_metres,
                    },
                    collider_local: Transform::from_translation(
                        direction * (capsule.length_metres * 0.5),
                    )
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, direction)),
                    kinematic: spec.kinematic,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let body_index = |role: BoneRole| bodies.iter().position(|body| body.role == role);
        let joints = JOINT_SPECS
            .iter()
            .map(|spec| {
                let parent = body_index(spec.parent)?;
                let child = body_index(spec.child)?;
                let child_position = bodies[child].bind.translation;
                let anchor = match spec.anchor {
                    AnchorAt::Child => child_position,
                    AnchorAt::Midpoint => (bodies[parent].bind.translation + child_position) * 0.5,
                    AnchorAt::Role(role) => bind(role).map_or(child_position, |t| t.translation),
                };
                let kind = match spec.kind {
                    JointSpecKind::Spherical {
                        twist,
                        swing_limit,
                        twist_limit,
                    } => RagdollJointKind::Spherical {
                        twist_axis: twist.resolve(&bind),
                        swing: swing_limit,
                        twist: twist_limit,
                    },
                    JointSpecKind::Hinge { bone, flex, limits } => RagdollJointKind::Hinge {
                        axis: hinge_axis(bone.resolve(&bind), flex, forward),
                        limits,
                    },
                };
                Some(RagdollJoint {
                    parent,
                    child,
                    anchor,
                    kind,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self { bodies, joints })
    }

    pub(super) fn body_of(&self, role: BoneRole) -> Option<usize> {
        self.bodies.iter().position(|body| body.role == role)
    }
}

/// The hinge axis about which a positive angle folds `bone_direction`
/// toward the flex direction: rotating `v` about `v × f` moves `v` toward
/// `f`. A limb already lying along its flex direction falls back to the
/// character's lateral axis.
fn hinge_axis(bone_direction: Vec3, flex: FlexDirection, forward: Vec3) -> Vec3 {
    let flex_direction = match flex {
        FlexDirection::Forward => forward,
        FlexDirection::Backward => -forward,
    };
    bone_direction
        .cross(flex_direction)
        .try_normalize()
        .unwrap_or_else(|| forward.cross(Vec3::Y).normalize_or_zero())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upright_bind(role: BoneRole) -> Option<Transform> {
        let position = match role {
            Pelvis => Vec3::new(0.0, 1.0, 0.0),
            Chest => Vec3::new(0.0, 1.35, 0.0),
            NeckOne => Vec3::new(0.0, 1.55, 0.0),
            Head => Vec3::new(0.0, 1.6, 0.0),
            ThighLeft => Vec3::new(-0.1, 0.95, 0.0),
            ShinLeft => Vec3::new(-0.1, 0.5, 0.0),
            FootLeft => Vec3::new(-0.1, 0.05, 0.0),
            ToeLeft => Vec3::new(-0.1, 0.02, -0.15),
            ThighRight => Vec3::new(0.1, 0.95, 0.0),
            ShinRight => Vec3::new(0.1, 0.5, 0.0),
            FootRight => Vec3::new(0.1, 0.05, 0.0),
            ToeRight => Vec3::new(0.1, 0.02, -0.15),
            UpperArmLeft => Vec3::new(-0.2, 1.4, 0.0),
            ForearmLeft => Vec3::new(-0.5, 1.4, 0.0),
            HandLeft => Vec3::new(-0.75, 1.4, 0.0),
            UpperArmRight => Vec3::new(0.2, 1.4, 0.0),
            ForearmRight => Vec3::new(0.5, 1.4, 0.0),
            HandRight => Vec3::new(0.75, 1.4, 0.0),
            _ => return None,
        };
        Some(Transform::from_translation(position))
    }

    fn capsules() -> RagdollCapsulesConfig {
        runtime_animation_config().ragdoll.capsules
    }

    #[test]
    fn definition_roots_at_a_kinematic_pelvis_and_lists_joints_parent_first() {
        let definition = RagdollDefinition::from_bind_pose(upright_bind, &capsules()).unwrap();
        assert_eq!(definition.bodies.len(), 15);
        assert_eq!(definition.joints.len(), 14);
        assert!(definition.bodies[PELVIS_BODY].kinematic);
        assert_eq!(definition.bodies[PELVIS_BODY].role, Pelvis);
        let mut placed = vec![false; definition.bodies.len()];
        placed[PELVIS_BODY] = true;
        for joint in &definition.joints {
            assert!(
                placed[joint.parent],
                "{:?}",
                definition.bodies[joint.child].role
            );
            placed[joint.child] = true;
        }
        assert!(placed.iter().all(|placed| *placed));
    }

    #[test]
    fn knee_and_elbow_hinges_fold_the_limb_the_anatomical_way() {
        let definition = RagdollDefinition::from_bind_pose(upright_bind, &capsules()).unwrap();
        let forward = character_forward(&upright_bind);
        assert!(forward.dot(Vec3::NEG_Z) > 0.99);
        let fold = |axis: Vec3, limb: Vec3| axis.cross(limb);
        let knee = definition
            .joints
            .iter()
            .find(|joint| definition.bodies[joint.child].role == ShinLeft)
            .unwrap();
        let RagdollJointKind::Hinge { axis, limits } = knee.kind else {
            panic!("knee must be a hinge");
        };
        assert!(
            fold(axis, Vec3::NEG_Y).dot(Vec3::Z) > 0.99,
            "knee folds the foot backward"
        );
        assert_eq!(limits.0, 0.0);
        let elbow = definition
            .joints
            .iter()
            .find(|joint| definition.bodies[joint.child].role == ForearmRight)
            .unwrap();
        let RagdollJointKind::Hinge { axis, .. } = elbow.kind else {
            panic!("elbow must be a hinge");
        };
        assert!(
            fold(axis, Vec3::X).dot(Vec3::NEG_Z) > 0.99,
            "elbow folds the hand forward"
        );
    }

    #[test]
    fn capsules_run_from_the_bone_along_the_limb() {
        let definition = RagdollDefinition::from_bind_pose(upright_bind, &capsules()).unwrap();
        let thigh = &definition.bodies[definition.body_of(ThighLeft).unwrap()];
        assert!(thigh.collider_local.translation.y < 0.0);
        assert!((thigh.collider_local.rotation * Vec3::Y).dot(Vec3::NEG_Y) > 0.99);
        assert_eq!(thigh.shape.radius, capsules().thigh.radius_metres);
    }

    #[test]
    fn a_rig_missing_a_driving_bone_has_no_ragdoll() {
        let missing_head = |role: BoneRole| (role != Head).then(|| upright_bind(role)).flatten();
        assert!(RagdollDefinition::from_bind_pose(missing_head, &capsules()).is_none());
    }
}

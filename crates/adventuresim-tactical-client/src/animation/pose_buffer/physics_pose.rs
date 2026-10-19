//! The pose buffer's boundary toward physics-driven poses.
//!
//! A ragdoll reads bodies back into `previous`/`next` so the same buffer
//! feeds rendering, the procedural passes, and, on exit, the ordinary
//! inertialized transition out of whatever tangle physics ended in.

use super::*;
use bevy::math::Affine3A;

impl PoseBufferRig {
    pub(in crate::animation) fn joint_count(&self) -> usize {
        self.definition.joints.len()
    }

    pub(in crate::animation) fn joint_role(&self, joint: RigJointOrdinal) -> Option<BoneRole> {
        self.definition.joints[joint].role
    }

    pub(in crate::animation) fn joint_parent(
        &self,
        joint: RigJointOrdinal,
    ) -> Option<RigJointOrdinal> {
        self.definition.joints[joint].parent
    }

    pub(in crate::animation) fn joint_entity(&self, joint: RigJointOrdinal) -> Option<Entity> {
        self.entities[joint]
    }

    pub(in crate::animation) fn previous_pose(&self, joint: RigJointOrdinal) -> LocalPose {
        self.previous[joint]
    }

    pub(in crate::animation) fn upcoming_pose(&self, joint: RigJointOrdinal) -> LocalPose {
        self.next[joint]
    }

    /// The pose the renderer shows this frame, inertial offset included.
    pub(in crate::animation) fn displayed_pose(&self, joint: RigJointOrdinal) -> LocalPose {
        let pose =
            self.previous[joint].interpolate(self.next[joint], self.interpolation_alpha.fraction());
        if self.inertial_state == InertialState::Settled {
            pose
        } else {
            self.offsets[joint].peek(pose)
        }
    }

    pub(in crate::animation) fn is_frozen(&self) -> bool {
        self.visibility == PoseVisibility::Frozen
    }

    pub(in crate::animation) fn is_physics_owned(&self) -> bool {
        self.ownership == PoseOwnership::Physics
    }

    /// Hand the buffer to physics or back to authored sampling. Returning it forces the next update to capture a
    /// transition out of the physics pose, even if the plan is unchanged.
    pub(in crate::animation) fn set_ownership(&mut self, ownership: PoseOwnership) {
        if self.ownership == PoseOwnership::Physics && ownership == PoseOwnership::Authored {
            self.plan = None;
        }
        self.ownership = ownership;
    }

    /// Physics owns this joint: both buffered samples become `pose`, and any
    /// in-flight transition on it is dropped so the bone lands exactly on
    /// its body.
    pub(in crate::animation) fn pin_joint(&mut self, joint: RigJointOrdinal, pose: LocalPose) {
        self.previous[joint] = pose;
        self.next[joint] = pose;
        self.offsets[joint] = JointInertialOffset::default();
        self.target_linear_velocities[joint] = Vec3::ZERO;
        self.target_angular_velocities[joint] = Vec3::ZERO;
    }

    /// An undriven ancestor of a driven joint: hold exactly `next`, with no
    /// interpolation or offset displacing it, so the driven descendants'
    /// derived locals compose back onto their bodies.
    pub(in crate::animation) fn hold_joint(&mut self, joint: RigJointOrdinal) {
        let held = self.next[joint];
        self.pin_joint(joint, held);
    }

    /// Seconds between the two buffered samples, for finite-difference
    /// velocities.
    pub(in crate::animation) fn sample_interval(
        &self,
    ) -> adventuresim_tactical_core::animation::PositiveAnimationDuration {
        pose_sample_interval()
    }
}

impl LocalPose {
    pub(in crate::animation) fn affine(self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

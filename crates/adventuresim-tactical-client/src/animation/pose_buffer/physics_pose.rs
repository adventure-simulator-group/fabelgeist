//! The pose buffer's boundary toward physics-driven poses.
//!
//! A ragdoll reads bodies back into `previous`/`next` so the same buffer
//! feeds rendering, the procedural passes, and, on exit, the ordinary
//! inertialized transition out of whatever tangle physics ended in.

use bevy::math::Affine3A;

use super::*;

impl PoseBufferRig {
    pub(in crate::animation) fn joint_count(&self) -> usize {
        self.definition.joints.len()
    }

    pub(in crate::animation) fn joint_name(&self, joint: usize) -> Option<&str> {
        self.definition.joints[joint].name.as_deref()
    }

    pub(in crate::animation) fn joint_parent(&self, joint: usize) -> Option<usize> {
        self.definition.joints[joint].parent
    }

    pub(in crate::animation) fn joint_entity(&self, joint: usize) -> Option<Entity> {
        self.entities[joint]
    }

    pub(in crate::animation) fn previous_pose(&self, joint: usize) -> LocalPose {
        self.previous[joint]
    }

    pub(in crate::animation) fn upcoming_pose(&self, joint: usize) -> LocalPose {
        self.next[joint]
    }

    /// The pose the renderer shows this frame, inertial offset included.
    pub(in crate::animation) fn displayed_pose(&self, joint: usize) -> LocalPose {
        let pose = self.previous[joint].interpolate(self.next[joint], self.interpolation_alpha);
        if self.settled {
            pose
        } else {
            self.offsets[joint].peek(pose)
        }
    }

    pub(in crate::animation) fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub(in crate::animation) fn is_physics_owned(&self) -> bool {
        self.physics_owned
    }

    /// Hand the buffer to physics (`true`) or back to authored sampling
    /// (`false`). Handing it back forces the next update to capture a
    /// transition out of the physics pose, even if the plan is unchanged.
    pub(in crate::animation) fn set_physics_owned(&mut self, owned: bool) {
        if self.physics_owned && !owned {
            self.plan = None;
        }
        self.physics_owned = owned;
    }

    /// Physics owns this joint: both buffered samples become `pose`, and any
    /// in-flight transition on it is dropped so the bone lands exactly on
    /// its body.
    pub(in crate::animation) fn pin_joint(&mut self, joint: usize, pose: LocalPose) {
        self.previous[joint] = pose;
        self.next[joint] = pose;
        self.offsets[joint] = JointInertialOffset::default();
        self.target_linear_velocities[joint] = Vec3::ZERO;
        self.target_angular_velocities[joint] = Vec3::ZERO;
    }

    /// An undriven ancestor of a driven joint: hold exactly `next`, with no
    /// interpolation or offset displacing it, so the driven descendants'
    /// derived locals compose back onto their bodies.
    pub(in crate::animation) fn hold_joint(&mut self, joint: usize) {
        let held = self.next[joint];
        self.pin_joint(joint, held);
    }

    /// Seconds between the two buffered samples, for finite-difference
    /// velocities.
    pub(in crate::animation) fn sample_interval_seconds(&self) -> f32 {
        pose_sample_seconds()
    }
}

impl LocalPose {
    pub(in crate::animation) fn affine(self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

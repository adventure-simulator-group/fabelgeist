//! Interruption-safe pose inertialization (Bollo, GDC 2018, "Inertialization:
//! High-Performance Animation Transitions in Gears of War").
//!
//! A plan change never crossfades clip against clip. The difference between
//! the displayed pose and the new target folds into a per-joint offset, and
//! that offset decays to zero over a fixed blend with a quintic that lands on
//! zero position, velocity, and acceleration. An interruption captures the
//! still-decaying offset together with its velocity, so back-to-back
//! transitions stay continuous.
//!
//! Each channel is a scalar magnitude along a direction frozen at capture; a
//! rotation offset is an angle about the axis of the offset quaternion. Two
//! overshoot guards from the talk apply: an initial velocity that carries the
//! offset away from zero is dropped, and the blend is shortened when the
//! offset is already closing fast enough to swing through zero.

use bevy::prelude::*;

use super::LocalPose;

/// Neighbourhood-checked slerp: `q` and `-q` are the same rotation, and
/// interpolating across the antipode is the classic one-frame bone teleport.
pub(super) fn hemisphere_slerp(first: Quat, mut second: Quat, alpha: f32) -> Quat {
    if first.dot(second) < 0.0 {
        second = -second;
    }
    first.slerp(second, alpha.clamp(0.0, 1.0)).normalize()
}

/// Keep `w >= 0` so logarithms and differences take the short way round.
pub(super) fn shortest_rotation(rotation: Quat) -> Quat {
    if rotation.w < 0.0 {
        -rotation
    } else {
        rotation
    }
}

pub(super) fn quaternion_exp(value: Vec3) -> Quat {
    let half_angle = value.length();
    if half_angle < 1e-8 {
        Quat::from_xyzw(value.x, value.y, value.z, 1.0).normalize()
    } else {
        let scale = half_angle.sin() / half_angle;
        Quat::from_xyzw(
            scale * value.x,
            scale * value.y,
            scale * value.z,
            half_angle.cos(),
        )
    }
}

pub(super) fn quaternion_log(rotation: Quat) -> Vec3 {
    let vector = Vec3::new(rotation.x, rotation.y, rotation.z);
    let length = vector.length();
    if length < 1e-8 {
        vector
    } else {
        rotation.w.clamp(-1.0, 1.0).acos() * vector / length
    }
}

pub(super) fn scaled_angle_axis(rotation: Quat) -> Vec3 {
    2.0 * quaternion_log(rotation)
}

pub(super) fn quaternion_from_scaled_angle_axis(value: Vec3) -> Quat {
    quaternion_exp(value / 2.0)
}

/// Angular velocity in scaled-angle-axis form (rad/s) between two rotations
/// `delta_seconds` apart. The hemisphere fix is mandatory: a near-antipodal
/// pair would otherwise read as a spin of ~2π per step.
pub(super) fn quaternion_angular_velocity(next: Quat, current: Quat, delta_seconds: f32) -> Vec3 {
    scaled_angle_axis(shortest_rotation(next * current.inverse())) / delta_seconds.max(1e-5)
}

pub(super) fn local_pose_velocity(
    previous: LocalPose,
    current: LocalPose,
    delta_seconds: f32,
) -> (Vec3, Vec3) {
    (
        (current.translation - previous.translation) / delta_seconds.max(1.0e-5),
        quaternion_angular_velocity(current.rotation, previous.rotation, delta_seconds),
    )
}

/// One inertialized channel: the magnitude of an offset along `direction`
/// as a quintic in time. The default is a finished (zero) offset.
#[derive(Clone, Copy, Debug, Default)]
struct QuinticChannel {
    direction: Vec3,
    x0: f32,
    v0: f32,
    a0: f32,
    a: f32,
    b: f32,
    c: f32,
    duration: f32,
}

impl QuinticChannel {
    /// `offset` is the offset vector at capture, `velocity` its rate of
    /// change, and `blend_seconds` the requested decay (shortened by the
    /// overshoot guard when the offset is already closing quickly).
    fn new(offset: Vec3, velocity: Vec3, blend_seconds: f32) -> Self {
        let magnitude = offset.length();
        // A pure velocity discontinuity still needs a direction to decay
        // along.
        let direction = if magnitude > 1e-6 {
            offset / magnitude
        } else {
            velocity.normalize_or_zero()
        };
        let x0 = magnitude;
        let mut v0 = velocity.dot(direction);
        let mut duration = blend_seconds;
        if x0 * v0 > 0.0 {
            v0 = 0.0;
        } else if v0 < 0.0 && x0 > 0.0 {
            duration = duration.min(-5.0 * x0 / v0);
        }
        let duration = duration.max(1e-3);
        let (t2, t3) = (duration * duration, duration * duration * duration);
        let (t4, t5) = (t3 * duration, t3 * t2);
        let a0 = (-8.0 * v0 * duration - 20.0 * x0) / t2;
        let a = -(a0 * t2 + 6.0 * v0 * duration + 12.0 * x0) / (2.0 * t5);
        let b = (3.0 * a0 * t2 + 16.0 * v0 * duration + 30.0 * x0) / (2.0 * t4);
        let c = -(3.0 * a0 * t2 + 12.0 * v0 * duration + 20.0 * x0) / (2.0 * t3);
        Self {
            direction,
            x0,
            v0,
            a0,
            a,
            b,
            c,
            duration,
        }
    }

    fn offset(&self, elapsed: f32) -> Vec3 {
        if elapsed >= self.duration {
            return Vec3::ZERO;
        }
        let t = elapsed;
        let x = ((((self.a * t + self.b) * t + self.c) * t + self.a0 / 2.0) * t + self.v0) * t
            + self.x0;
        self.direction * x
    }

    fn velocity(&self, elapsed: f32) -> Vec3 {
        if elapsed >= self.duration {
            return Vec3::ZERO;
        }
        let t = elapsed;
        let v =
            (((5.0 * self.a * t + 4.0 * self.b) * t + 3.0 * self.c) * t + self.a0) * t + self.v0;
        self.direction * v
    }
}

/// Per-joint inertialization state. The displayed pose is
/// `rotation_offset * input.rotation` and `input.translation + offset`; the
/// offset is what decays to identity and zero.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct JointInertialOffset {
    translation: QuinticChannel,
    rotation: QuinticChannel,
    elapsed: f32,
}

impl JointInertialOffset {
    /// Call at the moment of a plan change with the pose and velocity that
    /// were displayed this frame (offset included, which is what makes an
    /// interruption seamless) and the new target with its velocity.
    pub(super) fn capture(
        &mut self,
        displayed: LocalPose,
        displayed_linear_velocity: Vec3,
        displayed_angular_velocity: Vec3,
        target: LocalPose,
        target_linear_velocity: Vec3,
        target_angular_velocity: Vec3,
        blend_seconds: f32,
    ) {
        self.translation = QuinticChannel::new(
            displayed.translation - target.translation,
            displayed_linear_velocity - target_linear_velocity,
            blend_seconds,
        );
        let rotation_offset = shortest_rotation(displayed.rotation * target.rotation.inverse());
        self.rotation = QuinticChannel::new(
            scaled_angle_axis(rotation_offset),
            displayed_angular_velocity - target_angular_velocity,
            blend_seconds,
        );
        self.elapsed = 0.0;
    }

    /// Both channels have reached zero; the joint sits on the raw target.
    pub(super) fn done(&self) -> bool {
        self.elapsed >= self.translation.duration.max(self.rotation.duration)
    }

    pub(super) fn translation_velocity(&self) -> Vec3 {
        self.translation.velocity(self.elapsed)
    }

    pub(super) fn angular_velocity(&self) -> Vec3 {
        self.rotation.velocity(self.elapsed)
    }

    fn rotation_offset(&self) -> Quat {
        quaternion_from_scaled_angle_axis(self.rotation.offset(self.elapsed))
    }

    /// Advance the decay and return the displayed pose.
    pub(super) fn update(&mut self, input: LocalPose, delta_seconds: f32) -> LocalPose {
        self.elapsed += delta_seconds.max(0.0);
        self.peek(input)
    }

    /// Apply the current offset without advancing it.
    pub(super) fn peek(&self, input: LocalPose) -> LocalPose {
        if self.done() {
            return input;
        }
        LocalPose {
            translation: input.translation + self.translation.offset(self.elapsed),
            // Round trips through exp/log accumulate drift; keep it unit.
            rotation: (self.rotation_offset() * input.rotation).normalize(),
            scale: input.scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(translation: Vec3, rotation: Quat) -> LocalPose {
        LocalPose {
            translation,
            rotation,
            scale: Vec3::ONE,
        }
    }

    #[test]
    fn quintic_starts_on_the_displayed_pose_and_lands_exactly_on_the_target() {
        let displayed = pose(Vec3::new(1.0, 0.0, 0.0), Quat::from_rotation_y(0.4));
        let target = pose(Vec3::new(-1.0, 0.5, 0.0), Quat::from_rotation_x(-0.7));
        let mut offset = JointInertialOffset::default();
        offset.capture(
            displayed,
            Vec3::ZERO,
            Vec3::ZERO,
            target,
            Vec3::ZERO,
            Vec3::ZERO,
            0.2,
        );

        let start = offset.peek(target);
        assert!(start.translation.distance(displayed.translation) < 1.0e-5);
        assert!(start.rotation.angle_between(displayed.rotation) < 1.0e-5);

        let mut last = start;
        for _ in 0..20 {
            last = offset.update(target, 0.01);
        }
        assert!(offset.done());
        assert_eq!(last.translation, target.translation);
        assert!(last.rotation.angle_between(target.rotation) < 1.0e-6);
    }

    #[test]
    fn decay_finishes_with_zero_velocity() {
        let displayed = pose(Vec3::X, Quat::from_rotation_z(0.6));
        let target = pose(Vec3::ZERO, Quat::IDENTITY);
        let mut offset = JointInertialOffset::default();
        offset.capture(
            displayed,
            Vec3::ZERO,
            Vec3::ZERO,
            target,
            Vec3::ZERO,
            Vec3::ZERO,
            0.3,
        );
        let step = 0.001;
        let mut previous = offset.peek(target);
        let mut final_speed = f32::INFINITY;
        while !offset.done() {
            let current = offset.update(target, step);
            final_speed = current.translation.distance(previous.translation) / step;
            previous = current;
        }
        assert!(final_speed < 0.05, "{final_speed}");
        assert_eq!(offset.translation_velocity(), Vec3::ZERO);
        assert_eq!(offset.angular_velocity(), Vec3::ZERO);
    }

    #[test]
    fn closing_velocity_is_preserved_and_diverging_velocity_is_dropped() {
        let displayed = pose(Vec3::X * 0.2, Quat::IDENTITY);
        let target = pose(Vec3::NEG_X, Quat::IDENTITY);
        let mut closing = JointInertialOffset::default();
        closing.capture(
            displayed,
            Vec3::new(-0.5, 0.0, 0.0),
            Vec3::ZERO,
            target,
            Vec3::ZERO,
            Vec3::ZERO,
            0.25,
        );
        assert!(
            closing
                .translation_velocity()
                .distance(Vec3::new(-0.5, 0.0, 0.0))
                < 1.0e-5
        );

        let mut diverging = JointInertialOffset::default();
        diverging.capture(
            displayed,
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::ZERO,
            target,
            Vec3::ZERO,
            Vec3::ZERO,
            0.25,
        );
        assert_eq!(diverging.translation_velocity(), Vec3::ZERO);
    }

    #[test]
    fn fast_closing_offsets_finish_early_instead_of_overshooting() {
        let displayed = pose(Vec3::X * 0.1, Quat::IDENTITY);
        let target = pose(Vec3::ZERO, Quat::IDENTITY);
        let mut offset = JointInertialOffset::default();
        offset.capture(
            displayed,
            Vec3::new(-5.0, 0.0, 0.0),
            Vec3::ZERO,
            target,
            Vec3::ZERO,
            Vec3::ZERO,
            1.0,
        );
        let mut elapsed = 0.0;
        while !offset.done() {
            let current = offset.update(target, 0.005);
            elapsed += 0.005;
            assert!(current.translation.x >= -1.0e-4, "overshot at {elapsed}");
        }
        assert!(elapsed < 0.2, "{elapsed}");
    }

    #[test]
    fn chained_interruptions_preserve_the_displayed_pose() {
        let first = pose(Vec3::new(1.0, 0.0, 0.0), Quat::from_rotation_y(0.4));
        let second = pose(Vec3::new(-1.0, 0.0, 0.0), Quat::from_rotation_y(-0.7));
        let third = pose(Vec3::new(0.0, 1.0, 0.0), Quat::from_rotation_x(0.8));
        let mut offset = JointInertialOffset::default();
        offset.capture(
            first,
            Vec3::ZERO,
            Vec3::ZERO,
            second,
            Vec3::ZERO,
            Vec3::ZERO,
            0.2,
        );
        let displayed = offset.update(second, 0.025);
        offset.capture(
            displayed,
            offset.translation_velocity(),
            offset.angular_velocity(),
            third,
            Vec3::ZERO,
            Vec3::ZERO,
            0.2,
        );
        let after_interrupt = offset.peek(third);
        assert!(displayed.translation.distance(after_interrupt.translation) < 1.0e-4);
        assert!(displayed.rotation.angle_between(after_interrupt.rotation) < 1.0e-4);
    }

    #[test]
    fn quaternion_antipodes_interpolate_without_a_teleport() {
        let rotation = Quat::from_rotation_y(1.2);
        let halfway = hemisphere_slerp(rotation, -rotation, 0.5);
        assert!(rotation.angle_between(halfway) < 0.0001);
    }

    #[test]
    fn angular_velocity_uses_the_short_quaternion_hemisphere() {
        let rotation = Quat::from_rotation_x(0.4);
        let velocity = quaternion_angular_velocity(-rotation, rotation, 1.0 / 30.0);
        assert!(velocity.length() < 0.0001);
    }
}

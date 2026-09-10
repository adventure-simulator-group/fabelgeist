//! Analytic camera follow damping.

use super::CameraProfile;
use bevy::prelude::*;

pub(super) fn damp_focus(
    current: Vec3,
    target: Vec3,
    velocity: &mut Vec3,
    profile: CameraProfile,
    dt: f32,
) -> Vec3 {
    let mut horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    let horizontal = critical_damp_vec3(
        Vec3::new(current.x, 0.0, current.z),
        Vec3::new(target.x, 0.0, target.z),
        &mut horizontal_velocity,
        profile.horizontal_follow_time,
        dt,
    );
    let mut vertical_velocity = velocity.y;
    let y = critical_damp_scalar(
        current.y,
        target.y,
        &mut vertical_velocity,
        profile.vertical_follow_time,
        dt,
    );
    *velocity = Vec3::new(
        horizontal_velocity.x,
        vertical_velocity,
        horizontal_velocity.z,
    );
    Vec3::new(horizontal.x, y, horizontal.z)
}

pub(super) fn critical_damp_vec3(
    current: Vec3,
    target: Vec3,
    velocity: &mut Vec3,
    smooth_time: f32,
    dt: f32,
) -> Vec3 {
    let omega = 2.0 / smooth_time.max(0.0001);
    let displacement = current - target;
    let exponential = (-omega * dt).exp();
    let temporary = (*velocity + displacement * omega) * dt;
    *velocity = (*velocity - temporary * omega) * exponential;
    target + (displacement + temporary) * exponential
}

pub(super) fn critical_damp_scalar(
    current: f32,
    target: f32,
    velocity: &mut f32,
    smooth_time: f32,
    dt: f32,
) -> f32 {
    let omega = 2.0 / smooth_time.max(0.0001);
    let displacement = current - target;
    let exponential = (-omega * dt).exp();
    let temporary = (*velocity + displacement * omega) * dt;
    *velocity = (*velocity - temporary * omega) * exponential;
    target + (displacement + temporary) * exponential
}

pub(super) fn camera_view_metrics(projection: &Projection) -> (f32, f32) {
    match projection {
        Projection::Perspective(perspective) => (
            perspective.aspect_ratio.max(0.1),
            (perspective.fov * 0.5).tan(),
        ),
        _ => (16.0 / 9.0, 40.0_f32.to_radians().tan()),
    }
}

pub(super) fn blend_profile(a: CameraProfile, b: CameraProfile, t: f32) -> CameraProfile {
    CameraProfile {
        distance: a.distance.lerp(b.distance, t),
        shoulder_offset: a.shoulder_offset.lerp(b.shoulder_offset, t),
        focus_height: a.focus_height.lerp(b.focus_height, t),
        horizontal_follow_time: a.horizontal_follow_time.lerp(b.horizontal_follow_time, t),
        vertical_follow_time: a.vertical_follow_time.lerp(b.vertical_follow_time, t),
        maximum_follow_error: a.maximum_follow_error.lerp(b.maximum_follow_error, t),
        sweet_spot: a.sweet_spot.lerp(b.sweet_spot, t),
    }
}

pub(super) fn sweet_spot_target(
    anchor: Vec3,
    focus: Vec3,
    rotation: Quat,
    distance: f32,
    sweet_spot: Vec2,
    aspect: f32,
    tan_half_fov: f32,
) -> (Vec3, Vec2) {
    let right = rotation * Vec3::X;
    let up = rotation * Vec3::Y;
    let error = anchor - focus;
    let half_height = distance * tan_half_fov;
    let allowed_x = half_height * aspect * sweet_spot.x;
    let allowed_y = half_height * sweet_spot.y;
    let x = error.dot(right);
    let y = error.dot(up);
    let retained_x = x.clamp(-allowed_x, allowed_x);
    let retained_y = y.clamp(-allowed_y, allowed_y);
    let target = anchor - right * retained_x - up * retained_y;
    let screen_error = Vec2::new(
        if half_height > 0.0 {
            x / (half_height * aspect)
        } else {
            0.0
        },
        if half_height > 0.0 {
            y / half_height
        } else {
            0.0
        },
    );
    (target, screen_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::CameraRigConfig;

    #[test]
    fn critical_damping_is_nearly_render_rate_independent() {
        let simulate = |steps: usize| {
            let mut value = Vec3::ZERO;
            let mut velocity = Vec3::ZERO;
            for _ in 0..steps {
                value = critical_damp_vec3(
                    value,
                    Vec3::new(2.0, 1.0, -3.0),
                    &mut velocity,
                    0.25,
                    1.0 / steps as f32,
                );
            }
            value
        };
        assert!(simulate(30).abs_diff_eq(simulate(144), 0.0001));
    }

    #[test]
    fn sweet_spot_absorbs_small_motion_and_bounds_large_motion() {
        let profile = CameraRigConfig::default().lowered;
        let focus = Vec3::ZERO;
        let (small, _) = sweet_spot_target(
            Vec3::new(0.05, 0.02, 0.0),
            focus,
            Quat::IDENTITY,
            profile.distance,
            profile.sweet_spot,
            16.0 / 9.0,
            40.0_f32.to_radians().tan(),
        );
        assert!(small.abs_diff_eq(focus, 0.0001));
        let (large, _) = sweet_spot_target(
            Vec3::new(2.0, 0.0, 0.0),
            focus,
            Quat::IDENTITY,
            profile.distance,
            profile.sweet_spot,
            16.0 / 9.0,
            40.0_f32.to_radians().tan(),
        );
        assert!(large.x > 1.0 && large.x < 2.0);
    }
}

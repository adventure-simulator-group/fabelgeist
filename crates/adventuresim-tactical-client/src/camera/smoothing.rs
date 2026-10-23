//! Analytic camera follow damping.

use super::values::{CameraResult, ScreenError, ScreenWindow};
use super::{CameraProfile, ResponseShapingTime};
use adventuresim_building_generator::spatial_geometry::{Position, RigidRotation};
use adventuresim_tactical_core::{
    animation::{AnimationElapsed, UnitInterval},
    scene_coordinates::{Scene, SceneVelocity},
};
use bevy::prelude::*;
use fabelgeist_geometry::SignedLength;
use fabelgeist_math::units::Ratio;

#[derive(Clone, Copy)]
pub(super) struct FocusSample {
    pub(super) position: Position<Scene>,
    pub(super) velocity: SceneVelocity,
}

pub(super) struct SweetSpotSample {
    pub(super) target: Position<Scene>,
    pub(super) error: ScreenError,
}

pub(super) struct CameraViewMetrics {
    pub(super) aspect: Ratio,
    pub(super) tan_half_vertical_fov: Ratio,
}

pub(super) fn damp_focus(
    current: Position<Scene>,
    target: Position<Scene>,
    velocity: SceneVelocity,
    profile: CameraProfile,
    elapsed: AnimationElapsed,
) -> CameraResult<FocusSample> {
    // The analytic filters consume scene metres and scene metres per second.
    // Keep their original horizontal/vertical operation order, then admit the
    // complete replacement before publishing either position or velocity.
    let current = current.metres();
    let target = target.metres();
    let velocity = velocity.metres_per_second();
    let mut horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    let horizontal = critical_damp_vec3(
        Vec3::new(current.x, 0.0, current.z),
        Vec3::new(target.x, 0.0, target.z),
        &mut horizontal_velocity,
        profile.horizontal_follow_time,
        elapsed,
    );
    let mut vertical_velocity = velocity.y;
    let y = critical_damp_scalar(
        current.y,
        target.y,
        &mut vertical_velocity,
        profile.vertical_follow_time,
        elapsed,
    );
    Ok(FocusSample {
        position: Position::from_metres(Vec3::new(horizontal.x, y, horizontal.z))?,
        velocity: SceneVelocity::from_metres_per_second(Vec3::new(
            horizontal_velocity.x,
            vertical_velocity,
            horizontal_velocity.z,
        ))?,
    })
}

pub(super) fn critical_damp_vec3(
    current: Vec3,
    target: Vec3,
    velocity: &mut Vec3,
    smooth_time: ResponseShapingTime,
    elapsed: AnimationElapsed,
) -> Vec3 {
    let dt = elapsed.seconds();
    let omega = 2.0 / smooth_time.seconds().max(0.0001);
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
    smooth_time: ResponseShapingTime,
    elapsed: AnimationElapsed,
) -> f32 {
    let dt = elapsed.seconds();
    let omega = 2.0 / smooth_time.seconds().max(0.0001);
    let displacement = current - target;
    let exponential = (-omega * dt).exp();
    let temporary = (*velocity + displacement * omega) * dt;
    *velocity = (*velocity - temporary * omega) * exponential;
    target + (displacement + temporary) * exponential
}

pub(super) fn camera_view_metrics(projection: &Projection) -> CameraViewMetrics {
    match projection {
        Projection::Perspective(perspective) => CameraViewMetrics {
            aspect: Ratio::from_ratio(perspective.aspect_ratio.max(0.1)),
            tan_half_vertical_fov: Ratio::from_ratio((perspective.fov * 0.5).tan()),
        },
        _ => CameraViewMetrics {
            aspect: Ratio::from_ratio(16.0 / 9.0),
            tan_half_vertical_fov: Ratio::from_ratio(40.0_f32.to_radians().tan()),
        },
    }
}

pub(super) fn blend_profile(
    a: CameraProfile,
    b: CameraProfile,
    blend: UnitInterval,
) -> CameraResult<CameraProfile> {
    let t = blend.fraction();
    Ok(CameraProfile {
        distance: SignedLength::from_metres(a.distance.metres().lerp(b.distance.metres(), t))?,
        shoulder_offset: SignedLength::from_metres(
            a.shoulder_offset
                .metres()
                .lerp(b.shoulder_offset.metres(), t),
        )?,
        focus_height: SignedLength::from_metres(
            a.focus_height.metres().lerp(b.focus_height.metres(), t),
        )?,
        horizontal_follow_time: a
            .horizontal_follow_time
            .checked_lerp(b.horizontal_follow_time, blend)?,
        vertical_follow_time: a
            .vertical_follow_time
            .checked_lerp(b.vertical_follow_time, blend)?,
        maximum_follow_error: SignedLength::from_metres(
            a.maximum_follow_error
                .metres()
                .lerp(b.maximum_follow_error.metres(), t),
        )?,
        sweet_spot: ScreenWindow::from_ndc_half_extents(
            a.sweet_spot
                .ndc_half_extents()
                .lerp(b.sweet_spot.ndc_half_extents(), t),
        )?,
    })
}

pub(super) fn sweet_spot_target(
    anchor: Position<Scene>,
    focus: Position<Scene>,
    rotation: RigidRotation,
    distance: SignedLength,
    sweet_spot: ScreenWindow,
    view: CameraViewMetrics,
) -> CameraResult<SweetSpotSample> {
    // Project scene metres into the render camera basis; NDC is dimensionless.
    let anchor = anchor.metres();
    let focus = focus.metres();
    let rotation = rotation.quaternion();
    let distance = distance.metres();
    let sweet_spot = sweet_spot.ndc_half_extents();
    let right = rotation * Vec3::X;
    let up = rotation * Vec3::Y;
    let error = anchor - focus;
    let aspect = view.aspect.ratio();
    let half_height = distance * view.tan_half_vertical_fov.ratio();
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
    Ok(SweetSpotSample {
        target: Position::from_metres(target)?,
        error: ScreenError::from_ndc(screen_error)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::CameraRigConfig;

    #[test]
    fn reversing_a_mode_blend_continues_from_the_displayed_state() {
        let mut velocity = 0.0;
        let raised = critical_damp_scalar(
            0.0,
            1.0,
            &mut velocity,
            ResponseShapingTime::from_seconds(0.18).unwrap(),
            AnimationElapsed::from_seconds(0.08).unwrap(),
        );
        let reversed = critical_damp_scalar(
            raised,
            0.0,
            &mut velocity,
            ResponseShapingTime::from_seconds(0.18).unwrap(),
            AnimationElapsed::from_seconds(0.01).unwrap(),
        );
        assert!((reversed - raised).abs() < 0.1);
        assert!(reversed > 0.0 && reversed < 1.0);
    }

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
                    ResponseShapingTime::from_seconds(0.25).unwrap(),
                    AnimationElapsed::from_seconds(1.0 / steps as f32).unwrap(),
                );
            }
            value
        };
        assert!(simulate(30).abs_diff_eq(simulate(144), 0.0001));
    }

    #[test]
    fn sweet_spot_absorbs_small_motion_and_bounds_large_motion() {
        let profile = CameraRigConfig::default().lowered;
        let focus = Position::ORIGIN;
        let small = sweet_spot_target(
            Position::from_metres(Vec3::new(0.05, 0.02, 0.0)).unwrap(),
            focus,
            RigidRotation::IDENTITY,
            profile.distance,
            profile.sweet_spot,
            CameraViewMetrics {
                aspect: Ratio::from_ratio(16.0 / 9.0),
                tan_half_vertical_fov: Ratio::from_ratio(40.0_f32.to_radians().tan()),
            },
        )
        .unwrap();
        assert!(small.target.metres().abs_diff_eq(focus.metres(), 0.0001));
        let large = sweet_spot_target(
            Position::from_metres(Vec3::new(2.0, 0.0, 0.0)).unwrap(),
            focus,
            RigidRotation::IDENTITY,
            profile.distance,
            profile.sweet_spot,
            CameraViewMetrics {
                aspect: Ratio::from_ratio(16.0 / 9.0),
                tan_half_vertical_fov: Ratio::from_ratio(40.0_f32.to_radians().tan()),
            },
        )
        .unwrap();
        assert!(large.target.metres().x > 1.0 && large.target.metres().x < 2.0);
    }
}

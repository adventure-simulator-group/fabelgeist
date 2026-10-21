//! Body-heading observations retain their applicability and angular units.

use super::*;
use adventuresim_tactical_core::animation::AngularMagnitudeDegrees;

impl ScenarioFacingMetrics {
    pub(in crate::animation_viewer::report) fn measure(frames: &[&FrameSample]) -> AnimationValueResult<Self> {
        Ok(Self {
            maximum_facing_motion_error_degrees: maximum_facing_error(frames)?,
            maximum_facing_tracking_excess_degrees: maximum_facing_tracking_excess(frames)?,
            maximum_guard_facing_error_degrees: maximum_guard_facing_error(frames)?,
            final_facing_motion_error_degrees: final_facing_error(frames)?,
            maximum_dive_axis_motion_error_degrees: maximum_dive_axis_motion_error(frames)?,
        })
    }
}

pub(in crate::animation_viewer::report) fn maximum_facing_error(frames: &[&FrameSample]) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let maximum = frames
        .iter()
        .filter(|frame| {
            frame.speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map(|frame| {
            Vec3::from_array(frame.body_forward_direction)
                .angle_between(Vec3::from_array(frame.desired_body_forward_direction))
                .to_degrees()
        })
        .map(AngularMagnitudeDegrees::from_degrees)
        .collect::<AnimationValueResult<Vec<_>>>()?;
    if maximum.is_empty() { return Ok(Observation::NotApplicable); }
    let maximum = maximum.into_iter().fold(AngularMagnitudeDegrees::ZERO, AngularMagnitudeDegrees::max);
    Ok(Observation::Measured(maximum))
}

pub(in crate::animation_viewer::report) fn maximum_facing_tracking_excess(frames: &[&FrameSample]) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let maximum = frames
        .windows(2)
        .filter(|pair| {
            pair[1].speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map(|pair| {
            let before = Vec3::from_array(pair[0].body_forward_direction);
            let actual = Vec3::from_array(pair[1].body_forward_direction);
            let desired = Vec3::from_array(pair[1].desired_body_forward_direction);
            let elapsed = pair[1].time_seconds.saturating_sub(pair[0].time_seconds);
            let permitted_residual = body_facing_turn().tracking_residual(before, desired, elapsed);
            (actual.angle_between(desired) - permitted_residual)
                .abs()
                .to_degrees()
        })
        .map(AngularMagnitudeDegrees::from_degrees)
        .collect::<AnimationValueResult<Vec<_>>>()?;
    if maximum.is_empty() { return Ok(Observation::NotApplicable); }
    let maximum = maximum.into_iter().fold(AngularMagnitudeDegrees::ZERO, AngularMagnitudeDegrees::max);
    Ok(Observation::Measured(maximum))
}

pub(in crate::animation_viewer::report) fn maximum_guard_facing_error(frames: &[&FrameSample]) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let maximum = frames
        .iter()
        .filter(|frame| {
            frame.guard_action
                && frame.speed_metres_per_second
                    > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map(|frame| {
            Vec3::from_array(frame.body_forward_direction)
                .angle_between(Vec3::from_array(frame.desired_body_forward_direction))
                .to_degrees()
        })
        .map(AngularMagnitudeDegrees::from_degrees)
        .collect::<AnimationValueResult<Vec<_>>>()?;
    if maximum.is_empty() { return Ok(Observation::NotApplicable); }
    let maximum = maximum.into_iter().fold(AngularMagnitudeDegrees::ZERO, AngularMagnitudeDegrees::max);
    Ok(Observation::Measured(maximum))
}

pub(in crate::animation_viewer::report) fn final_facing_error(frames: &[&FrameSample]) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let measurement = frames
        .iter()
        .rev()
        .find(|frame| {
            frame.speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map(|frame| {
            Vec3::from_array(frame.body_forward_direction)
                .angle_between(Vec3::from_array(frame.desired_body_forward_direction))
                .to_degrees()
        });
    let Some(measurement) = measurement else {
        return Ok(Observation::NotApplicable);
    };
    Ok(Observation::Measured(AngularMagnitudeDegrees::from_degrees(measurement)?))
}

pub(in crate::animation_viewer::report) fn maximum_dive_axis_motion_error(
    frames: &[&FrameSample],
) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let mut applicable = false;
    let mut maximum: Option<AngularMagnitudeDegrees> = None;
    for frame in frames.iter().filter(|frame| frame.scenario.has(ScenarioFeature::Dive)) {
        applicable = true;
        let Some(head) = frame.bones.get(BoneRole::Head) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: BoneRole::Head }));
        };
        let Some(pelvis) = frame.bones.get(BoneRole::Pelvis) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: BoneRole::Pelvis }));
        };
        let head = Vec3::from_array(head.position);
        let pelvis = Vec3::from_array(pelvis.position);
        let axis = Vec3::new(head.x - pelvis.x, 0.0, head.z - pelvis.z);
        // Upright frames have no stable horizontal heading yet.
        if axis.length() < 0.25 {
            continue;
        }
        let axis = axis.normalize();
        let Some(travel) = Vec3::from_array(frame.world_travel_direction).try_normalize() else {
            return Ok(Observation::Unavailable(UnavailableReason::NoMotionSamples));
        };
        let value = AngularMagnitudeDegrees::from_degrees(axis.angle_between(travel).to_degrees())?;
        maximum = Some(maximum.map_or(value, |maximum| maximum.max(value)));
    }
    if !applicable {
        return Ok(Observation::NotApplicable);
    }
    let Some(maximum) = maximum else {
        return Ok(Observation::Unavailable(UnavailableReason::NoStableDiveAxis));
    };
    Ok(Observation::Measured(maximum))
}

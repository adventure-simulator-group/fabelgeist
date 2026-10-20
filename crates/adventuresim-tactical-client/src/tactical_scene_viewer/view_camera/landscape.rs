//! Landscape proof cameras consume admitted geographic observations.

use super::*;

pub(super) fn vista_valley(state: &SceneCaptureState, half: f32) -> CaptureCameraResult<(Vec3, Vec3, Vec3)> {
    let vista = state.vista_geometry.ok_or(CaptureCameraError::MissingVista)?;
    let direction = (vista.minimum().point.metres() - state.obstacle_focus.xz())
        .try_normalize()
        .unwrap_or(-Vec2::X);
    let safe_height = state
        .terrain
        .maximum_height_metres
        .max(vista.maximum().elevation)
        .metres()
        + 18.0;
    let mut position = state.obstacle_focus
        - Vec3::new(direction.x, 0.0, direction.y) * (half * 0.62)
        + Vec3::Y * 8.0;
    position.y = position.y.max(safe_height);
    let mut target = state.obstacle_focus
        + Vec3::new(direction.x, 0.0, direction.y) * (half * 5.0)
        + Vec3::Y * 2.0;
    target.y = target.y.clamp(position.y - 80.0, position.y - 8.0);
    Ok((position, target, Vec3::Y))
}


pub(super) fn horizon(state: &SceneCaptureState, half: f32) -> (Vec3, Vec3, Vec3) {
    let mut position = state.ground_eye_position + Vec3::Y * 12.0;
    let peak = state.vista_geometry.map(|vista| vista.maximum());
    let highest = peak.map_or(state.terrain.maximum_height_metres, |peak| {
        state.terrain.maximum_height_metres.max(peak.elevation)
    });
    position.y = position.y.max(highest.metres() + 12.0);
    let heading = peak.map_or(state.ground_eye_target.xz() - position.xz(), |peak| {
        peak.point.metres() - position.xz()
    });
    let direction = heading
        .try_normalize()
        .unwrap_or(Vec2::X);
    let target =
        position + Vec3::new(direction.x, -0.08, direction.y) * (half * 10.0).max(200.0);
    (position, target, Vec3::Y)
}

pub(super) fn vista_peak(state: &SceneCaptureState, half: f32) -> CaptureCameraResult<(Vec3, Vec3, Vec3)> {
    let vista = state.vista_geometry.ok_or(CaptureCameraError::MissingVista)?;
    let direction = (vista.maximum().point.metres() - state.obstacle_focus.xz())
        .try_normalize()
        .unwrap_or(Vec2::X);
    let safe_height = state
        .terrain
        .maximum_height_metres
        .max(vista.maximum().elevation)
        .metres()
        + 18.0;
    let mut position = state.obstacle_focus
        - Vec3::new(direction.x, 0.0, direction.y) * (half * 0.62)
        + Vec3::Y * 8.0;
    position.y = position.y.max(safe_height);
    let mut target = state.obstacle_focus
        + Vec3::new(direction.x, 0.0, direction.y) * (half * 5.0)
        + Vec3::Y * 2.0;
    target.y = target.y.clamp(position.y - 80.0, position.y - 8.0);
    Ok((position, target, Vec3::Y))
}

pub(super) fn terrain_grazing(state: &SceneCaptureState) -> (Vec3, Vec3, Vec3) {
    let target = state.obstacle_focus - Vec3::Y * 1.42;
    // Keep this plate close enough to resolve centimetre-scale ground
    // geometry. The former seven-metre view was useful as landscape
    // context but could not distinguish a 25 cm detail grid from the
    // authoritative two-metre surface it refines.
    (
        target + Vec3::new(-1.65, 0.28, 1.45),
        target + Vec3::new(0.35, 0.01, -0.25),
        Vec3::Y,
    )
}

pub(super) fn city_oblique(state: &SceneCaptureState, half: f32, azimuth_degrees: f32) -> (Vec3, Vec3, Vec3) {
    let frame_half = half.max(state.city_half_extent_metres + 30.0);
    let azimuth = azimuth_degrees.to_radians();
    (
        state.obstacle_focus
            + Vec3::new(azimuth.sin(), 0.0, azimuth.cos()) * (frame_half * 1.6)
            + Vec3::Y * (frame_half * 0.85),
        state.obstacle_focus + Vec3::Y * 4.0,
        Vec3::Y,
    )
}

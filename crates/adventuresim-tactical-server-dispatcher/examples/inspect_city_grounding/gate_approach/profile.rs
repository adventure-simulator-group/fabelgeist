//! Longitudinal trial route with a level platform covering the gate swing bound.
use super::*;
use bevy::math::Vec3Swizzles;

pub(super) fn inspect(
    compound: &CityCompound,
    route: &CityAccessSegment,
    street_height: f32,
    gate_height: f32,
    court_height: f32,
    maximum_grade: f32,
) -> Result<Value, adventuresim_building_generator::DoorError> {
    let delta = route.end_metres() - route.start_metres();
    let length = delta.length();
    let direction = delta / length;
    let gate_distance =
        (compound.boundary.gate.centre_metres.metres() - route.start_metres()).dot(direction);
    let door = compound.boundary.gate.door(compound.id)?;
    let hinge_distance = (door.hinge_centre.metres().xz() - route.start_metres()).dot(direction);
    let radius = door.horizontal_sweep_radius_metres()?;
    let platform_end = hinge_distance + radius + route.half_width_metres();
    let court_begin = length - route.half_width_metres();
    let run = court_begin - platform_end;
    let rise = (gate_height - court_height).abs();
    let permitted = run.max(0.0) * maximum_grade;
    let points = [
        (0.0, street_height),
        (route.half_width_metres(), street_height),
        (gate_distance - route.half_width_metres(), gate_height),
        (platform_end, gate_height),
        (court_begin, court_height),
        (length, court_height),
    ];
    Ok(json!({
        "hinge_projection_distance_m":hinge_distance,
        "leaf_sweep_radius_m":radius,
        "gate_platform_end_distance_m":platform_end,
        "court_landing_begin_distance_m":court_begin,
        "gate_to_court_effective_run_m":run,
        "required_rise_m":rise,"permitted_rise_m":permitted,
        "shortfall_m":(rise-permitted).max(0.0),
        "grade":if run > 0.0 {Some(rise/run)} else {None},
        "passes_endpoint_grade":run > 0.0 && rise <= permitted,
        "points":points.map(|(distance,height)|json!({"distance_m":distance,
            "point":route.start_metres()+direction*distance,"elevation_m":height})),
        "verification_scope":"A longitudinal route profile with landings and a conservative gate-swing radius. Lateral support, retaining faces, terrain transitions, actor traversal and complete gate-sweep collision are unaccepted.",
    }))
}

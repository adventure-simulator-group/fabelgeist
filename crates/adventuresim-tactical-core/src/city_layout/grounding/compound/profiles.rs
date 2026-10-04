use super::*;

pub(super) fn court(
    property: &CityCompound,
    levels: &mut CompoundSupportLevels,
    front: &CityAccessSegment,
    rear: &CityAccessSegment,
    limits: SupportLimits,
    treatment: CourtTreatment,
) -> Result<(SupportProfile, Vec<CourtStair>), SupportDiagnostic> {
    let local = |point| {
        property
            .plot
            .orientation
            .world_to_local(point - property.plot.centre_metres)
    };
    let court_z = local(property.court.centre_metres).y;
    let floor = levels.front.elevation.metres();
    let mut stairs = Vec::new();
    match treatment {
        CourtTreatment::Level => {
            levels.court = SupportElevation(floor);
            levels.rear.elevation = SupportElevation(floor);
        }
        CourtTreatment::Terraced(bounds) => {
            for (route, member) in [(front, levels.front), (rear, levels.rear)] {
                if let Some(stair) = CourtStair::compile(
                    property,
                    route,
                    member.elevation,
                    levels.court,
                    bounds,
                    limits,
                )? {
                    stairs.push(stair);
                }
            }
        }
    }
    let front_bearing_end = levels
        .front
        .contact
        .corners()
        .into_iter()
        .map(|point| local(point).y)
        .fold(f32::NEG_INFINITY, f32::max);
    let rear_bearing_begin = levels
        .rear
        .contact
        .corners()
        .into_iter()
        .map(|point| local(point).y)
        .fold(f32::INFINITY, f32::min);
    let minimum = (court_z - property.court.dimensions_metres.y * 0.5).max(front_bearing_end);
    let maximum = (court_z + property.court.dimensions_metres.y * 0.5).min(rear_bearing_begin);
    if minimum >= maximum {
        return Err(SupportDiagnostic::new(
            property,
            SupportConstraint::Bearing,
            SupportBoundary::CourtLanding,
            property.court.centre_metres,
            minimum - maximum,
            0.0,
        ));
    }
    let half_depth = property.plot.dimensions_metres.y * 0.5;
    // Vertical terrace boundaries are retaining faces, not walkable ramps.
    // Only the explicitly reserved stair flights cross these boundaries.
    let points = [
        (-half_depth, floor),
        (minimum, floor),
        (minimum, levels.court.metres()),
        (maximum, levels.court.metres()),
        (maximum, levels.rear.elevation.metres()),
        (half_depth, levels.rear.elevation.metres()),
    ]
    .map(|(distance_metres, height_metres)| ProfilePoint {
        distance_metres,
        height_metres,
    });
    Ok((
        SupportProfile {
            points: points.to_vec(),
        },
        stairs,
    ))
}

pub(super) fn passage(
    property: &CityCompound,
    levels: CompoundSupportLevels,
    route: CityAccessSegment,
    limits: SupportLimits,
) -> Result<SupportProfile, SupportDiagnostic> {
    let delta = route.end_metres - route.start_metres;
    let length = delta.length();
    let direction = delta / length;
    let gate = (property.boundary.gate.centre_metres - route.start_metres).dot(direction);
    let door = property.boundary.gate.door(property.id);
    let hinge = (door.hinge_centre.xz() - route.start_metres).dot(direction);
    let platform_end = hinge + door.horizontal_sweep_radius_metres() + route.half_width_metres;
    let points = [
        (0.0, levels.street.metres()),
        (route.half_width_metres, levels.street.metres()),
        (gate - route.half_width_metres, levels.gate.metres()),
        (platform_end, levels.gate.metres()),
        (length - route.half_width_metres, levels.court.metres()),
        (length, levels.court.metres()),
    ]
    .map(|(distance_metres, height_metres)| ProfilePoint {
        distance_metres,
        height_metres,
    });
    SupportProfile::checked(
        property,
        points.to_vec(),
        limits,
        SupportBoundary::GateLanding,
        |distance| route.start_metres + direction * distance,
    )
}

use super::*;

pub(super) struct CourtSupportConstruction {
    pub profile: SupportProfile,
    pub stairs: Vec<CourtStair>,
}

pub(super) fn court(
    property: &CityCompound,
    levels: &mut CompoundSupportLevels,
    front: &CityAccessSegment,
    rear: &CityAccessSegment,
    limits: SupportLimits,
    treatment: CourtTreatment,
) -> Result<CourtSupportConstruction, SupportDiagnostic> {
    let local = |point| {
        property
            .plot
            .orientation()
            .world_to_local(point - property.plot.centre_metres())
    };
    let court_z = local(property.court.centre_metres()).y;
    let floor = levels.front.elevation;
    let mut stairs = Vec::new();
    match treatment {
        CourtTreatment::Level => {
            levels.court = floor;
            levels.rear.elevation = floor;
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
    let minimum = (court_z - property.court.dimensions_metres().y * 0.5).max(front_bearing_end);
    let maximum = (court_z + property.court.dimensions_metres().y * 0.5).min(rear_bearing_begin);
    if minimum >= maximum {
        return Err(SupportDiagnostic::new(
            property,
            SupportConstraint::Bearing,
            SupportBoundary::CourtLanding,
            property.court.centre_metres(),
            minimum - maximum,
            0.0,
        ));
    }
    let half_depth = property.plot.dimensions_metres().y * 0.5;
    // Vertical terrace boundaries are retaining faces, not walkable ramps.
    // Only the explicitly reserved stair flights cross these boundaries.
    let points = [
        ProfilePoint::at_metres(-half_depth, floor),
        ProfilePoint::at_metres(minimum, floor),
        ProfilePoint::at_metres(minimum, levels.court),
        ProfilePoint::at_metres(maximum, levels.court),
        ProfilePoint::at_metres(maximum, levels.rear.elevation),
        ProfilePoint::at_metres(half_depth, levels.rear.elevation),
    ];
    Ok(CourtSupportConstruction {
        profile: court_sequence(property, points.into_iter().collect())?,
        stairs,
    })
}

pub(super) fn passage(
    property: &CityCompound,
    levels: CompoundSupportLevels,
    route: CityAccessSegment,
    limits: SupportLimits,
) -> Result<SupportProfile, SupportDiagnostic> {
    let delta = route.end_metres() - route.start_metres();
    let length = delta.length();
    let direction = delta / length;
    let gate =
        (property.boundary.gate.centre_metres.metres() - route.start_metres()).dot(direction);
    let door = property
        .boundary
        .gate
        .door(property.id)
        .map_err(|cause| SupportDiagnostic::gate_construction(property, cause))?;
    let hinge = (door.hinge_centre.metres().xz() - route.start_metres()).dot(direction);
    let platform_end = hinge
        + door
            .horizontal_sweep_radius_metres()
            .map_err(|cause| SupportDiagnostic::gate_construction(property, cause))?
        + route.half_width_metres();
    let points = [
        ProfilePoint::at_metres(0.0, levels.street),
        ProfilePoint::at_metres(route.half_width_metres(), levels.street),
        ProfilePoint::at_metres(gate - route.half_width_metres(), levels.gate),
        ProfilePoint::at_metres(platform_end, levels.gate),
        ProfilePoint::at_metres(length - route.half_width_metres(), levels.court),
        ProfilePoint::at_metres(length, levels.court),
    ];
    SupportProfile::checked(
        property,
        points
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                SupportDiagnostic::new(
                    property,
                    SupportConstraint::Reservation,
                    SupportBoundary::GateLanding,
                    route.start_metres(),
                    1.0,
                    0.0,
                )
            })?,
        limits,
        SupportBoundary::GateLanding,
        |distance| route.start_metres() + direction * distance.metres(),
    )
}

fn court_sequence(
    property: &CityCompound,
    points: Vec<Option<ProfilePoint>>,
) -> Result<SupportProfile, SupportDiagnostic> {
    SupportProfile::stepped(
        points
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                SupportDiagnostic::new(
                    property,
                    SupportConstraint::Reservation,
                    SupportBoundary::CourtLanding,
                    property.court.centre_metres(),
                    1.0,
                    0.0,
                )
            })?,
    )
    .map_err(|error| {
        SupportDiagnostic::new(
            property,
            SupportConstraint::Reservation,
            SupportBoundary::CourtLanding,
            property.plot.centre_metres()
                + property
                    .plot
                    .orientation()
                    .local_to_world(Vec2::Y * error.coordinate().metres()),
            1.0,
            0.0,
        )
    })
}

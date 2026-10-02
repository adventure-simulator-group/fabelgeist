use super::*;

const ROUTE_JOIN_TOLERANCE_METRES: f32 = 0.02;

pub(in crate::city_layout::compiled) fn validate_access(
    compound: &CityCompound,
    front: &TacticalBuildingPlacement,
    front_recipe: &Recipe,
    rear: &TacticalBuildingPlacement,
    rear_recipe: &Recipe,
    streets: &[CityStreetPatch],
) -> Result<(), CityCompileError> {
    let error = |issue| CityCompileError::Compound {
        property: compound.id,
        issue,
    };
    let front_door = front_recipe
        .door_point(front, Vec2::Y)
        .ok_or(error(CompoundIssue::MissingCourtDoor))?;
    let rear_door = rear_recipe
        .door_point(rear, -Vec2::Y)
        .ok_or(error(CompoundIssue::MissingRangeDoor))?;
    let endpoints = |route: &CityAccessSegment| [route.start_metres, route.end_metres];
    let near = |a: Vec2, b: Vec2| a.distance(b) <= ROUTE_JOIN_TOLERANCE_METRES;
    let mut connected = BTreeSet::new();
    for (i, route) in compound.access.iter().enumerate() {
        if endpoints(route)
            .iter()
            .any(|p| streets.iter().any(|street| street.contains(*p)))
        {
            connected.insert(i);
        }
    }
    loop {
        let before = connected.len();
        for (i, route) in compound.access.iter().enumerate() {
            if connected.iter().any(|&j| {
                endpoints(&compound.access[j])
                    .iter()
                    .any(|a| endpoints(route).iter().any(|b| near(*a, *b)))
            }) {
                connected.insert(i);
            }
        }
        if before == connected.len() {
            break;
        }
    }
    if connected.len() != compound.access.len() || connected.is_empty() {
        return Err(error(CompoundIssue::StreetDisconnected));
    }
    for (door, issue) in [
        (front_door, CompoundIssue::MissingCourtDoor),
        (rear_door, CompoundIssue::MissingRangeDoor),
    ] {
        if !compound
            .access
            .iter()
            .any(|route| endpoints(route).iter().any(|p| near(door, *p)))
        {
            return Err(error(issue));
        }
    }
    Ok(())
}

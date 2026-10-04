use super::*;
use bevy::math::Vec3Swizzles;
mod profiles;

/// Project the observed gate platform only within its reserved street landing
/// reach. Geographic source and the unchanged street endpoint remain separate.
pub(super) fn street_gate_level(
    property: &CityCompound,
    levels: CompoundSupportLevels,
    limits: SupportLimits,
) -> Result<SupportElevation, SupportDiagnostic> {
    let mut routes = property
        .access
        .iter()
        .filter(|route| route.contains_centreline(property.boundary.gate.centre_metres));
    let route = routes
        .next()
        .filter(|_| routes.next().is_none())
        .ok_or_else(|| {
            SupportDiagnostic::new(
                property,
                SupportConstraint::GateBinding,
                SupportBoundary::GateLanding,
                property.boundary.gate.centre_metres,
                1.0,
                0.0,
            )
        })?;
    let delta = route.end_metres - route.start_metres;
    let station =
        (property.boundary.gate.centre_metres - route.start_metres).dot(delta.normalize_or_zero());
    let run = station - route.half_width_metres * 2.0;
    let reach = run * limits.maximum_grade - limits.contact_tolerance_metres;
    if reach < 0.0 {
        return Err(SupportDiagnostic::new(
            property,
            SupportConstraint::AccessGrade,
            SupportBoundary::GateLanding,
            route.start_metres,
            (levels.gate.metres() - levels.street.metres()).abs(),
            0.0,
        ));
    }
    Ok(SupportElevation(levels.gate.metres().clamp(
        levels.street.metres() - reach,
        levels.street.metres() + reach,
    )))
}

pub(super) fn compile(
    property: &CityCompound,
    mut levels: CompoundSupportLevels,
    limits: SupportLimits,
    treatment: CourtTreatment,
) -> Result<CompoundSupportPlan, SupportDiagnostic> {
    let binding = |constraint, boundary, location| {
        SupportDiagnostic::new(property, constraint, boundary, location, 1.0, 0.0)
    };
    validate_inputs(property, levels)?;
    let route_to = |member: MemberSupport| {
        let mut routes = property
            .access
            .iter()
            .filter(|route| route.ends_at(member.court_threshold_metres));
        let first = routes.next();
        first.filter(|_| routes.next().is_none()).ok_or(binding(
            SupportConstraint::ThresholdBinding,
            SupportBoundary::CourtLanding,
            member.court_threshold_metres,
        ))
    };
    let front_route = route_to(levels.front)?;
    let rear_route = route_to(levels.rear)?;
    let mut gate_routes = property
        .access
        .iter()
        .filter(|route| route.contains_centreline(property.boundary.gate.centre_metres));
    let passage = gate_routes
        .next()
        .filter(|_| gate_routes.next().is_none())
        .copied()
        .ok_or(binding(
            SupportConstraint::GateBinding,
            SupportBoundary::GateLanding,
            property.boundary.gate.centre_metres,
        ))?;
    for route in [front_route, rear_route, &passage] {
        let delta = property
            .plot
            .orientation
            .world_to_local(route.end_metres - route.start_metres);
        if !delta.is_finite()
            || delta.x.abs() > CityAccessSegment::JOIN_TOLERANCE_METRES
            || delta.y.abs() <= route.half_width_metres * 2.0
            || !route.half_width_metres.is_finite()
            || route.half_width_metres <= 0.0
        {
            return Err(binding(
                SupportConstraint::Reservation,
                SupportBoundary::PropertyReservation,
                route.start_metres,
            ));
        }
    }
    let (court_profile, court_stairs) = profiles::court(
        property,
        &mut levels,
        front_route,
        rear_route,
        limits,
        treatment,
    )?;
    let passage_profile = profiles::passage(property, levels, passage, limits)?;
    let split = support_boundary(property, levels.front, limits)?;
    let plan = CompoundSupportPlan {
        property: property.clone(),
        levels,
        limits,
        passage,
        split_frontage_metres: split,
        court_profile,
        passage_profile,
        court_stairs,
        treatment,
        street_entry: None,
    };
    validate_bearings(&plan)?;
    Ok(plan)
}

fn validate_inputs(
    property: &CityCompound,
    levels: CompoundSupportLevels,
) -> Result<(), SupportDiagnostic> {
    let binding = |constraint, boundary, location| {
        SupportDiagnostic::new(property, constraint, boundary, location, 1.0, 0.0)
    };
    if levels.front.building_id != property.front_building_id
        || levels.rear.building_id != property.rear_building_id
    {
        return Err(binding(
            SupportConstraint::MemberBinding,
            SupportBoundary::PropertyReservation,
            property.plot.centre_metres,
        ));
    }
    if !property.plot.is_valid()
        || !property.court.is_valid()
        || property.court.orientation != property.plot.orientation
        || !levels.front.contact.is_valid()
        || !levels.rear.contact.is_valid()
        || levels.front.contact.orientation != property.plot.orientation
        || levels.rear.contact.orientation != property.plot.orientation
    {
        return Err(binding(
            SupportConstraint::Reservation,
            SupportBoundary::PropertyReservation,
            property.plot.centre_metres,
        ));
    }
    Ok(())
}

fn support_boundary(
    property: &CityCompound,
    front: MemberSupport,
    limits: SupportLimits,
) -> Result<f32, SupportDiagnostic> {
    let side = property.boundary.gate.hinge.opposite().sign();
    let local = |point| {
        property
            .plot
            .orientation
            .world_to_local(point - property.plot.centre_metres)
    };
    let post = property.boundary.gate.post(property.boundary.gate.hinge);
    let post_half = post.size_metres.x * 0.5;
    let split = local(post.centre_metres.xz()).x - side * post_half;
    let contact_edge = front
        .contact
        .corners()
        .into_iter()
        .map(|point| local(point).x * side)
        .fold(f32::NEG_INFINITY, f32::max);
    let penetration = contact_edge - split * side;
    if penetration > limits.contact_tolerance_metres {
        return Err(SupportDiagnostic::new(
            property,
            SupportConstraint::Bearing,
            SupportBoundary::FrontBearing,
            post.centre_metres.xz(),
            penetration,
            limits.contact_tolerance_metres,
        ));
    }
    Ok(split)
}

fn validate_bearings(plan: &CompoundSupportPlan) -> Result<(), SupportDiagnostic> {
    let mesh = plan.mesh();
    for (member, boundary) in [
        (plan.levels.front, SupportBoundary::FrontBearing),
        (plan.levels.rear, SupportBoundary::RearBearing),
    ] {
        let corners = member.contact.corners();
        let to_local = |point| {
            plan.property
                .plot
                .orientation
                .world_to_local(point - plan.property.plot.centre_metres)
        };
        let minimum_z = corners
            .into_iter()
            .map(|p| to_local(p).y)
            .fold(f32::INFINITY, f32::min);
        let maximum_z = corners
            .into_iter()
            .map(|p| to_local(p).y)
            .fold(f32::NEG_INFINITY, f32::max);
        let contact_x = to_local(member.contact.centre_metres).x;
        let intersections = plan
            .court_profile
            .points
            .iter()
            .filter(|p| (minimum_z..=maximum_z).contains(&p.distance_metres))
            .map(|p| {
                plan.property.plot.centre_metres
                    + plan
                        .property
                        .plot
                        .orientation
                        .local_to_world(Vec2::new(contact_x, p.distance_metres))
            });
        for point in corners.into_iter().chain(intersections) {
            if !plan.property.plot.contains(point) {
                return Err(SupportDiagnostic::new(
                    &plan.property,
                    SupportConstraint::Reservation,
                    SupportBoundary::PropertyReservation,
                    point,
                    1.0,
                    0.0,
                ));
            }
            // At a retaining edge, both surfaces exist. The bound member's
            // architectural floor selects its intended bearing elevation;
            // averaging the edge or selecting its upper level would be wrong.
            let error = mesh
                .elevations_at(point)
                .iter()
                .map(|height| (height.metres() - member.elevation.metres()).abs())
                .fold(f32::INFINITY, f32::min);
            if error > plan.limits.contact_tolerance_metres {
                return Err(SupportDiagnostic::new(
                    &plan.property,
                    SupportConstraint::Bearing,
                    boundary,
                    point,
                    error,
                    plan.limits.contact_tolerance_metres,
                ));
            }
        }
    }
    Ok(())
}

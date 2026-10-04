//! Admit gateway route payloads without changing geometry or validation order.

use super::route_error::RouteAdmissionError;
use super::*;

pub(super) fn validate_journey_route(
    ctx: &ReducerContext,
    route: &JourneyRoutePlan,
    origin: (f64, f64),
    destination: (f64, f64),
) -> Result<(), RouteAdmissionError> {
    let authority = require_strategic_gateway(ctx)?;
    if authority.terrain_schema != 3
        || authority.terrain_package_digest.as_deref() != Some(route.package_digest.as_str())
    {
        return Err(RouteAdmissionError::TerrainPackageMismatch);
    }
    validate_journey_route_payload(route, origin, destination)
}

pub(super) fn validate_journey_route_payload(
    route: &JourneyRoutePlan,
    origin: (f64, f64),
    destination: (f64, f64),
) -> Result<(), RouteAdmissionError> {
    const MAX_POINTS: usize = 512;
    const MAX_SPANS: usize = 256;
    if !valid_route_digest(&route.package_digest) {
        return Err(RouteAdmissionError::InvalidDigest);
    }
    if route.weather_rules_version != adventuresim_core::weather::WEATHER_RULES_VERSION
        || route
            .weather_interval_start
            .floor_to_interval_minutes(adventuresim_core::weather::WEATHER_INTERVAL_MINUTES)
            != Some(route.weather_interval_start)
        || route.intensity_bps > adventuresim_world_schema::BASIS_POINTS_PER_WHOLE
        || route.ground_moisture_bps > adventuresim_world_schema::BASIS_POINTS_PER_WHOLE
        || route.snow_cover_bps > adventuresim_world_schema::BASIS_POINTS_PER_WHOLE
        || (route.precipitation == Precipitation::Clear && route.intensity_bps != 0)
    {
        return Err(RouteAdmissionError::InvalidWeather);
    }
    if !(2..=MAX_POINTS).contains(&route.points.len())
        || route.spans.is_empty()
        || route.spans.len() > MAX_SPANS
        || route.distance_m == 0
        || route.distance_m > 2_000_000
        || route.minutes == 0
        || route.minutes > 2_000_000
    {
        return Err(RouteAdmissionError::AggregateBounds);
    }
    let Some(coordinates) = route
        .points
        .iter()
        .map(wgs84_route_coordinate)
        .collect::<Option<Vec<_>>>()
    else {
        return Err(RouteAdmissionError::InvalidCoordinate);
    };
    let first = coordinates
        .first()
        .expect("bounded nonempty route")
        .longitude_latitude_degrees();
    let last = coordinates
        .last()
        .expect("bounded nonempty route")
        .longitude_latitude_degrees();
    if straight_line_distance_m(first.0, first.1, origin.0, origin.1, true) > 500
        || straight_line_distance_m(last.0, last.1, destination.0, destination.1, true) > 500
    {
        return Err(RouteAdmissionError::EndpointMismatch);
    }
    let mut physical = 0_u64;
    for pair in coordinates.windows(2) {
        let from = pair[0].longitude_latitude_degrees();
        let to = pair[1].longitude_latitude_degrees();
        let segment = straight_line_distance_m(from.0, from.1, to.0, to.1, true);
        if segment == 0 || segment > 100_000 {
            return Err(RouteAdmissionError::DiscontinuousPath);
        }
        physical = physical
            .checked_add(segment)
            .ok_or(RouteAdmissionError::DistanceOverflow)?;
    }
    let tolerance = route.distance_m / 20 + 250;
    if physical.abs_diff(route.distance_m) > tolerance {
        return Err(RouteAdmissionError::DistanceMismatch);
    }
    let minimum_minutes = route
        .distance_m
        .saturating_mul(MINUTES_PER_HOUR)
        .div_ceil(7_500)
        .max(1);
    if route.minutes < minimum_minutes {
        return Err(RouteAdmissionError::ExcessSpeed);
    }
    let mut cursor = 0_u64;
    for span in &route.spans {
        let weight_sum = u32::from(span.terrain.plains)
            + u32::from(span.terrain.forest)
            + u32::from(span.terrain.hills)
            + u32::from(span.terrain.wetlands)
            + u32::from(span.terrain.urban);
        if weight_sum != 1_000
            || span.terrain.urban != 0
            || span.training_multiplier_permille > 1_000
            || span.check_millirank > 5_000
        {
            return Err(RouteAdmissionError::InvalidSkillMetadata);
        }
        if span.start_minute != cursor || span.duration_minutes == 0 {
            return Err(RouteAdmissionError::DiscontinuousSpans);
        }
        cursor = cursor
            .checked_add(span.duration_minutes)
            .ok_or(RouteAdmissionError::MinutesOverflow)?;
    }
    if cursor != route.minutes {
        return Err(RouteAdmissionError::MinutesMismatch);
    }
    Ok(())
}

pub(super) fn validate_route_departure_weather_interval(
    route: &JourneyRoutePlan,
    departure_minute: StrategicMinute,
) -> Result<(), RouteAdmissionError> {
    super::route_weather::require_departure_weather_interval(
        route.weather_interval_start,
        departure_minute,
    )
}

pub(super) fn validate_return_journey_route(
    ctx: &ReducerContext,
    route: &JourneyRoutePlan,
    origin: (f64, f64),
    destination: (f64, f64),
) -> Result<(), RouteAdmissionError> {
    let leg = route
        .return_route
        .as_ref()
        .ok_or(RouteAdmissionError::ReturnRouteRequired)?;
    validate_journey_route(
        ctx,
        &JourneyRoutePlan {
            package_digest: route.package_digest.clone(),
            weather_rules_version: route.weather_rules_version,
            weather_interval_start: route.weather_interval_start,
            precipitation: route.precipitation,
            intensity_bps: route.intensity_bps,
            ground_moisture_bps: route.ground_moisture_bps,
            snow_cover_bps: route.snow_cover_bps,
            distance_m: leg.distance_m,
            minutes: leg.minutes,
            points: leg.points.clone(),
            spans: leg.spans.clone(),
            return_route: None,
        },
        origin,
        destination,
    )
}

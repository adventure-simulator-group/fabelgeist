//! Redirect a reached camp without advancing its participants.

use super::*;

pub(super) fn redirect_camped_party_to_settlement(
    ctx: &ReducerContext,
    party: &mut Party,
    destination: &Settlement,
    route: Option<JourneyRoutePlan>,
) -> Result<(), TravelError> {
    let mut journey = ctx
        .db
        .party_journey_authority()
        .party_id()
        .find(&party.id)
        .ok_or(TravelError::CampJourneyMissing)?;
    let redirect_departure_minute = living_party_member_ids(ctx, &party.id)
        .into_iter()
        .filter_map(|member_id| {
            ctx.db
                .character_time()
                .character_id()
                .find(u64::from(member_id))
        })
        .map(|time| time.minutes)
        .max()
        .unwrap_or(journey.departure_minute);
    let travel_minutes = if let Some(route) = route.as_ref() {
        validate_route_departure_weather_interval(route, redirect_departure_minute)?;
        let current_route = ctx
            .db
            .party_journey_route_authority()
            .party_id()
            .find(&party.id)
            .ok_or(TravelError::CampRouteMissing)?;
        let origin = route_position_at_minute(&current_route, journey.completed_movement_minutes)
            .ok_or(TravelError::CampPositionUnavailable)?;
        validate_journey_route(
            ctx,
            route,
            origin,
            (destination.coord_x, destination.coord_y),
        )?;
        route.minutes
    } else {
        camp_redirect_minutes(&journey, &destination.id).ok_or(TravelError::NotCampEndpoint)?
    };
    if travel_minutes == 0 {
        return Err(TravelError::AlreadyAtJourneyEndpoint);
    }

    journey.origin = JourneyEndpoint::Camp(party.id.clone());
    journey.destination = JourneyEndpoint::Settlement(JourneySettlementEndpoint {
        id: destination.id.clone(),
        name: destination.name.clone(),
    });
    journey.total_movement_minutes = travel_minutes;
    journey.completed_movement_minutes = 0;
    journey.departure_minute = redirect_departure_minute;
    journey.completed_elapsed_minutes = 0;
    journey.reached_camp_movement_minutes.clear();
    journey.actual_camp_intervals.clear();
    journey.forecast_camp_intervals.clear();
    ctx.db.party_journey_authority().party_id().update(journey);
    if ctx
        .db
        .party_journey_route_authority()
        .party_id()
        .find(&party.id)
        .is_some()
    {
        ctx.db
            .party_journey_route_authority()
            .party_id()
            .delete(&party.id);
    }
    if let Some(route) = route {
        ctx.db
            .party_journey_route_authority()
            .insert(PartyJourneyRoute {
                party_id: party.id.clone(),
                gateway_bucket: 0,
                package_digest: route.package_digest,
                weather_rules_version: route.weather_rules_version,
                weather_interval_start: route.weather_interval_start,
                precipitation: route.precipitation,
                intensity_bps: route.intensity_bps,
                ground_moisture_bps: route.ground_moisture_bps,
                snow_cover_bps: route.snow_cover_bps,
                distance_m: route.distance_m,
                minutes: route.minutes,
                points: route.points,
                spans: route.spans,
                return_route: route.return_route,
            });
    }

    party.current_settlement_id = None;
    party.current_case_site_id = None;
    party.camp_destination = Some(JourneyEndpoint::Settlement(JourneySettlementEndpoint {
        id: destination.id.clone(),
        name: destination.name.clone(),
    }));
    party.camp_remaining_minutes = travel_minutes;
    ctx.db.party_authority().id().update(party.clone());
    refresh_party_journey_forecast(ctx, &party.id)?;
    Ok(())
}

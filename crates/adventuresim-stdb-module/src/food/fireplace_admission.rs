// Owns validation of persisted fireplace station and dish authority.
fn parse_persisted_fireplace_fixture(
    fireplace_fixture_id: &str,
) -> Result<StrategicFixtureId, FireplaceCustodyError> {
    match fireplace_fixture_id
        .parse::<StrategicFixtureId>()
        .map_err(FireplaceCustodyError::Fixture)?
    {
        fixture @ StrategicFixtureId::Fireplace { .. } => Ok(fixture),
        _ => Err(FireplaceCustodyError::NonFireplaceFixture),
    }
}

fn validate_persisted_station_fixture(
    ctx: &ReducerContext,
    station: &FireplaceStation,
) -> Result<StrategicFixtureId, FireplaceCustodyError> {
    let fixture = parse_persisted_fireplace_fixture(&station.fireplace_fixture_id)?;
    let expected_key = match station.instrument_object_id {
        Some(object_id) => vessel_station_key(
            station.character_id,
            &station.fireplace_fixture_id,
            object_id,
        ),
        None => station_key(station.character_id, &station.fireplace_fixture_id),
    };
    if station.key != expected_key {
        return Err(FireplaceCustodyError::StationKeyMismatch);
    }
    if station.instrument_item_id.is_some() != station.instrument_return_custody.is_some() {
        return Err(FireplaceCustodyError::AmbiguousReturnCustody);
    }
    if let Some(custody) = station.instrument_return_custody.as_ref() {
        crate::object_custody::carried_destination(custody, (station.character_id).into())?;
    }
    if let Some(object_id) = station.instrument_object_id {
        let object = ctx
            .db
            .inventory_object()
            .id()
            .find(object_id)
            .ok_or(FireplaceCustodyError::MissingStationObject)?;
        if station.instrument_item_id.as_deref() != Some(object.item_id.as_str()) {
            return Err(FireplaceCustodyError::StationObjectMismatch);
        }
        crate::object_custody::require_object_at_fixture(ctx, &object, &fixture)?;
    }
    Ok(fixture)
}

fn validate_persisted_dish_fixture(
    ctx: &ReducerContext,
    dish: &FireplaceDish,
) -> Result<StrategicFixtureId, FireplaceCustodyError> {
    let fixture = parse_persisted_fireplace_fixture(&dish.fireplace_fixture_id)?;
    let station = ctx
        .db
        .fireplace_station()
        .key()
        .find(dish.station_key.clone())
        .ok_or(FireplaceCustodyError::MissingDishStation)?;
    let station_fixture = validate_persisted_station_fixture(ctx, &station)?;
    if station.character_id != dish.character_id || station_fixture != fixture {
        return Err(FireplaceCustodyError::DishStationMismatch);
    }
    crate::object_custody::carried_destination(&dish.return_custody, (dish.character_id).into())?;
    Ok(fixture)
}

// Owns fireplace fixture validation, vessel custody, cleanup, placement, and retrieval.
fn station_key(character_id: u64, fireplace_fixture_id: &str) -> String {
    format!("{character_id}|{fireplace_fixture_id}")
}

pub(crate) fn require_clear_current_camp_fireplace(
    ctx: &ReducerContext,
    camp_place: &StrategicPlaceId,
) -> Result<(), FireplaceCustodyError> {
    if !matches!(camp_place, StrategicPlaceId::JourneyCamp { .. }) {
        return Err(FireplaceCustodyError::NotJourneyCamp);
    }
    let mut occupied = false;
    for station in ctx.db.fireplace_station().iter() {
        let fixture = validate_persisted_station_fixture(ctx, &station)?;
        occupied |= fixture.place() == camp_place && station.instrument_item_id.is_some();
    }
    for dish in ctx.db.fireplace_dish().iter() {
        let fixture = validate_persisted_dish_fixture(ctx, &dish)?;
        occupied |= fixture.place() == camp_place;
    }
    if occupied {
        Err(FireplaceCustodyError::OccupiedCamp)
    } else {
        Ok(())
    }
}

pub(crate) fn require_members_clear_current_camp_fireplace(
    ctx: &ReducerContext,
    party_id: &str,
    character_ids: &[u64],
) -> Result<(), String> {
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(party_id.to_string())
        .ok_or("Party not found")?;
    if party.camp_destination.is_none() {
        return Ok(());
    }
    let journey = ctx
        .db
        .party_journey_authority()
        .party_id()
        .find(party_id.to_string())
        .ok_or("Journey camp not found")?;
    if !crate::strategic::party_journey_is_current_camp(&party, &journey) {
        return Err("Party has incoherent current journey camp authority".into());
    }
    let place = crate::strategic::current_journey_camp_place(ctx, party_id)?;
    let member_ids = character_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut occupied = false;
    for station in ctx.db.fireplace_station().iter() {
        let fixture = validate_persisted_station_fixture(ctx, &station)?;
        occupied |= member_ids.contains(&station.character_id)
            && fixture.place() == &place
            && station.instrument_item_id.is_some();
    }
    for dish in ctx.db.fireplace_dish().iter() {
        let fixture = validate_persisted_dish_fixture(ctx, &dish)?;
        occupied |= member_ids.contains(&dish.character_id) && fixture.place() == &place;
    }
    if occupied {
        Err("Retrieve this member's dish and remove their cooking instrument before they leave the camp party".into())
    } else {
        Ok(())
    }
}

/// Resolves only the dead character's private station rows. Unretrieved food is
/// abandoned. Tools return to their exact recorded source when it still exists;
/// otherwise they move to the dead character's personal estate inventory. If
/// even that character row is absent, the tool is abandoned with the station.
/// A stale party reference can therefore never lock travel or leak another
/// player's dish.
pub(crate) fn cleanup_fireplace_custody_for_death(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> Result<(), String> {
    enum StationCleanup {
        Delete {
            station_key: String,
        },
        Abandon,
        Return {
            station_key: String,
            item_id: String,
            object_id: Option<u64>,
            destination: OperationalCustody,
        },
    }

    let personal_estate_exists = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .is_some();
    let stations = ctx
        .db
        .fireplace_station()
        .character_id()
        .filter(u64::from(character_id))
        .collect::<Vec<_>>();
    let mut cleanup = Vec::with_capacity(stations.len());

    // Resolve and validate every return before deleting a dish, changing an
    // object row, or removing a station.
    for station in &stations {
        validate_persisted_station_fixture(ctx, station)?;
        let Some(item_id) = station.instrument_item_id.as_deref() else {
            cleanup.push(StationCleanup::Delete {
                station_key: station.key.clone(),
            });
            continue;
        };
        let recorded_destination = crate::object_custody::carried_destination(
            station
                .instrument_return_custody
                .as_ref()
                .ok_or("Fireplace instrument return custody is missing")?,
            (character_id).into(),
        )?;
        let exact_party = match &recorded_destination {
            OperationalCustody::Party(party_id) => Some(party_id),
            _ => None,
        }
        .filter(|party_id| {
            ctx.db
                .party_authority()
                .id()
                .find(party_id.as_str().to_owned())
                .is_some()
        });
        let destination = if let Some(party_id) = exact_party {
            Some(OperationalCustody::party(party_id.as_str()).map_err(|error| error.to_string())?)
        } else if personal_estate_exists {
            Some(
                OperationalCustody::character((character_id).into())
                    .map_err(|error| error.to_string())?,
            )
        } else {
            None
        };
        let Some(destination) = destination else {
            cleanup.push(StationCleanup::Abandon);
            continue;
        };
        if let Some(object_id) = station.instrument_object_id {
            let object = ctx
                .db
                .inventory_object()
                .id()
                .find(object_id)
                .ok_or("Fireplace instrument object is missing")?;
            if object.item_id != item_id {
                return Err("Fireplace instrument conflicts with its physical object".into());
            }
            crate::inventory_container::prevalidate_rehome_subtree(ctx, object_id, &destination)?;
        }
        cleanup.push(StationCleanup::Return {
            station_key: station.key.clone(),
            item_id: item_id.into(),
            object_id: station.instrument_object_id,
            destination,
        });
    }

    for dish in ctx
        .db
        .fireplace_dish()
        .character_id()
        .filter(u64::from(character_id))
        .collect::<Vec<_>>()
    {
        ctx.db
            .fireplace_dish()
            .station_key()
            .delete(dish.station_key);
    }
    for plan in cleanup {
        let StationCleanup::Return {
            station_key,
            item_id,
            object_id,
            destination,
        } = plan
        else {
            if let StationCleanup::Delete { station_key } = plan {
                ctx.db.fireplace_station().key().delete(station_key);
            }
            // Abandoned tools remain installed at their station.
            continue;
        };
        if let Some(object_id) = object_id {
            let row_id = match &destination {
                OperationalCustody::Party(party_id) => {
                    ctx.db
                        .party_inventory_item()
                        .insert(PartyInventoryItem {
                            id: 0,
                            party_id: party_id.as_str().into(),
                            item_id: item_id.clone(),
                            quantity: 1,
                        })
                        .id
                }
                OperationalCustody::Character(character) => {
                    ctx.db
                        .inventory_item()
                        .insert(crate::InventoryItem {
                            id: 0,
                            character_id: character.get(),
                            item_id: item_id.clone(),
                            quantity: 1,
                        })
                        .id
                }
                _ => return Err("Fireplace return destination is not carried inventory".into()),
            };
            let mut object = ctx
                .db
                .inventory_object()
                .id()
                .find(object_id)
                .ok_or("Fireplace instrument object is missing")?;
            object.location =
                crate::inventory_container::carried_location_for_row(&destination, row_id)?;
            ctx.db.inventory_object().id().update(object);
            crate::inventory_container::rehome_subtree(ctx, object_id, &destination)?;
        } else {
            match destination {
                OperationalCustody::Party(party_id) => {
                    crate::strategic::add_to_party_inventory_checked(
                        ctx,
                        party_id.as_str(),
                        &item_id,
                        1,
                    )?;
                }
                OperationalCustody::Character(character) => {
                    ctx.db.inventory_item().insert(crate::InventoryItem {
                        id: 0,
                        character_id: character.get(),
                        item_id,
                        quantity: 1,
                    });
                }
                _ => return Err("Fireplace return destination is not carried inventory".into()),
            }
        }
        ctx.db.fireplace_station().key().delete(station_key);
    }
    Ok(())
}

fn validate_fireplace_fixture(
    ctx: &ReducerContext,
    actor: &crate::Character,
    fireplace_fixture_id: &str,
) -> Result<(), String> {
    let fixture = fireplace_fixture_id
        .parse::<StrategicFixtureId>()
        .map_err(|_| "Invalid canonical fireplace identity")?;
    let StrategicFixtureId::Fireplace { place } = fixture else {
        return Err("Fixture is not a fireplace".into());
    };
    match place {
        StrategicPlaceId::SettlementVenue {
            settlement_id,
            kind,
        } => {
            if actor.current_settlement_id.as_deref() != Some(settlement_id.as_str()) {
                return Err("The character is not at this settlement fireplace".into());
            }
            let settlement = ctx
                .db
                .settlement()
                .id()
                .find(settlement_id.as_str().to_string())
                .ok_or("Settlement not found")?;
            let available = match kind {
                adventuresim_core::strategic_place::SettlementVenueKind::Residences => true,
                adventuresim_core::strategic_place::SettlementVenueKind::Keep => matches!(
                    settlement.category,
                    crate::strategic::SettlementCategory::Town
                        | crate::strategic::SettlementCategory::City
                        | crate::strategic::SettlementCategory::Capital
                ),
                adventuresim_core::strategic_place::SettlementVenueKind::Market => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "merchants",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::Forge => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "weapons",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::Armoury => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "armor",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::Tailor => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "clothing",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::Herbalist => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "herbalist",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::Inn => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "inn",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::Church => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "religion",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::Bookstore => {
                    adventuresim_core::organization::service_npc_location_available(
                        &settlement.economy,
                        "books",
                    )
                }
                adventuresim_core::strategic_place::SettlementVenueKind::PublicSquare => false,
            };
            if !available {
                return Err("This settlement building has no fireplace".into());
            }
            Ok(())
        }
        StrategicPlaceId::ChapterVenue {
            settlement_id,
            organization_id,
            authored_location_id,
        } => {
            if actor.current_settlement_id.as_deref() != Some(settlement_id.as_str()) {
                return Err("The character is not at this settlement fireplace".into());
            }
            let settlement = ctx
                .db
                .settlement()
                .id()
                .find(settlement_id.as_str().to_string())
                .ok_or("Settlement not found")?;
            let available = adventuresim_core::organization::organization_chapter_at(
                settlement_id.as_str(),
                authored_location_id.as_str(),
            )
            .is_some_and(|(organization, chapter)| {
                organization.id == organization_id.as_str()
                    && chapter.location_id == authored_location_id.as_str()
                    && adventuresim_core::organization::chapter_has_standalone_building(
                        organization,
                        chapter,
                        &settlement.economy,
                    )
            });
            if !available {
                return Err("This settlement building has no fireplace".into());
            }
            Ok(())
        }
        StrategicPlaceId::JourneyCamp {
            party_id,
            departure_minute,
            movement_minute,
        } => {
            if actor.party_id.as_deref() != Some(party_id.as_str()) {
                return Err("The character is not in this camp's party".into());
            }
            let current = crate::strategic::current_journey_camp_place(ctx, party_id.as_str())?;
            if current
                != StrategicPlaceId::journey_camp(
                    party_id.as_str(),
                    departure_minute,
                    movement_minute,
                )
                .map_err(|_| "Invalid canonical camp identity")?
            {
                return Err("This is not the party's current journey camp".into());
            }
            Ok(())
        }
        _ => Err("Invalid fireplace place".into()),
    }
}

fn fireplace_station_for(
    ctx: &ReducerContext,
    character_id: u64,
    fireplace_fixture_id: &str,
) -> FireplaceStation {
    let key = station_key(character_id, fireplace_fixture_id);
    ctx.db
        .fireplace_station()
        .key()
        .find(key.clone())
        .unwrap_or(FireplaceStation {
            key,
            character_id,
            fireplace_fixture_id: fireplace_fixture_id.into(),
            instrument_item_id: None,
            instrument_object_id: None,
            instrument_return_custody: None,
        })
}

fn method_for_instrument(item_id: Option<&str>) -> Result<CookingMethod, String> {
    match item_id {
        None => Ok(CookingMethod::Roast),
        Some("cooking_pan") => Ok(CookingMethod::PanFry),
        Some("cooking_pot") => Ok(CookingMethod::Stew),
        Some("portable_oven") => Ok(CookingMethod::Bake),
        _ => Err("That item is not a cooking instrument".into()),
    }
}

fn vessel_station_key(character_id: u64, fireplace_fixture_id: &str, object_id: u64) -> String {
    format!("{character_id}|{fireplace_fixture_id}|container:{object_id}")
}

/// Places one exact vessel and its entire subtree over this exact fireplace.
/// The root carried row is removed, so ordinary inventory/trade views cannot
/// remotely transfer it. Children retain their stable object edges.
#[reducer]
pub fn place_fireplace_container(
    ctx: &ReducerContext,
    character_id: u64,
    fireplace_fixture_id: String,
    inventory_scope: String,
    inventory_item_id: u64,
) -> Result<(), String> {
    crate::strategic::require_strategic_gateway(ctx)
        .map_err(|error: crate::strategic::GatewayAdmissionError| error.to_string())?;
    let actor = crate::character::require_living_character(ctx, (character_id).into())
        .map_err(|error: crate::character::LivingCharacterError| error.to_string())?;
    if actor.has_tactical_server_assignment() {
        return Err("Cooking is unavailable during a tactical encounter".into());
    }
    validate_fireplace_fixture(ctx, &actor, &fireplace_fixture_id)?;
    let inventory_scope = CarriedInventoryScope::try_from(inventory_scope.as_str())
        .map_err(|error| error.to_string())?;
    let mut object = crate::inventory_container::require_object(
        ctx,
        character_id.into(),
        inventory_scope,
        inventory_item_id.into(),
    )
    .map_err(
        |error: crate::inventory_container::InventoryObjectError| -> String { error.to_string() },
    )?;
    if crate::inventory_container::object_is_nested(ctx, object.id) {
        return Err(
            "Remove a vessel from its parent container before placing it over a fire".into(),
        );
    }
    method_for_instrument(Some(&object.item_id))?;
    let source_custody = crate::object_custody::carried_scope_custody(ctx, &actor, inventory_scope)
        .map_err(
            |error: crate::object_custody::ObjectCustodyError| -> String { error.to_string() },
        )?;
    let resolved = crate::object_custody::resolve_object_custody(ctx, &object).map_err(
        |error: crate::object_custody::ObjectCustodyError| -> String { error.to_string() },
    )?;
    if resolved.root != source_custody {
        return Err("Container custody conflicts with the selected inventory".into());
    }
    let persisted_source = crate::object_custody::encode_custody(&source_custody);
    match &object.location {
        InventoryLocation::Personal(location) => {
            ctx.db.inventory_item().id().delete(location.row_id);
        }
        InventoryLocation::Party(location) => {
            ctx.db.party_inventory_item().id().delete(location.row_id);
        }
        _ => return Err("Container is not in carried inventory".into()),
    }
    let key = vessel_station_key(character_id, &fireplace_fixture_id, object.id);
    object.location = InventoryLocation::fireplace(fireplace_fixture_id.clone());
    ctx.db.inventory_object().id().update(object.clone());
    ctx.db.fireplace_station().insert(FireplaceStation {
        key,
        character_id,
        fireplace_fixture_id,
        instrument_item_id: Some(object.item_id),
        instrument_object_id: Some(object.id),
        instrument_return_custody: Some(persisted_source),
    });
    Ok(())
}

#[reducer]
pub fn retrieve_fireplace_container(
    ctx: &ReducerContext,
    character_id: u64,
    fireplace_fixture_id: String,
    container_object_id: u64,
) -> Result<(), String> {
    crate::strategic::require_strategic_gateway(ctx)
        .map_err(|error: crate::strategic::GatewayAdmissionError| error.to_string())?;
    let actor = crate::character::require_living_character(ctx, (character_id).into())
        .map_err(|error: crate::character::LivingCharacterError| error.to_string())?;
    if actor.has_tactical_server_assignment() {
        return Err("Cooking is unavailable during a tactical encounter".into());
    }
    validate_fireplace_fixture(ctx, &actor, &fireplace_fixture_id)?;
    let key = vessel_station_key(character_id, &fireplace_fixture_id, container_object_id);
    let station = ctx
        .db
        .fireplace_station()
        .key()
        .find(key.clone())
        .ok_or("Container is not at this fireplace")?;
    if ctx
        .db
        .fireplace_dish()
        .station_key()
        .find(key.clone())
        .is_some()
    {
        return Err("Retrieve the cooked dish before removing its container".into());
    }
    let item_id = station
        .instrument_item_id
        .clone()
        .ok_or("Fireplace vessel is missing")?;
    let fixture = validate_persisted_station_fixture(ctx, &station)
        .map_err(|error: crate::food::FireplaceCustodyError| -> String { error.to_string() })?;
    let return_custody = station
        .instrument_return_custody
        .as_ref()
        .ok_or("Container return custody is unknown")?;
    let destination =
        crate::object_custody::carried_destination(return_custody, (character_id).into()).map_err(
            |error: crate::object_custody::ObjectCustodyError| -> String { error.to_string() },
        )?;
    crate::inventory_container::prevalidate_rehome_subtree(ctx, container_object_id, &destination)
        .map_err(
            |error: crate::inventory_container::InventoryContainerError| -> String {
                error.to_string()
            },
        )?;
    let inventory_row_id = match &destination {
        OperationalCustody::Character(character) => {
            let row = ctx.db.inventory_item().insert(crate::InventoryItem {
                id: 0,
                character_id: character.get(),
                item_id: item_id.clone(),
                quantity: 1,
            });
            row.id
        }
        OperationalCustody::Party(party_id) => {
            if ctx
                .db
                .party_authority()
                .id()
                .find(party_id.as_str().to_owned())
                .is_none()
            {
                return Err("Original party inventory is unavailable".into());
            }
            let row = ctx
                .db
                .party_inventory_item()
                .insert(crate::strategic::PartyInventoryItem {
                    id: 0,
                    party_id: party_id.as_str().into(),
                    item_id: item_id.clone(),
                    quantity: 1,
                });
            row.id
        }
        _ => return Err("Container return custody is not a carried inventory".into()),
    };
    let mut object = ctx
        .db
        .inventory_object()
        .id()
        .find(container_object_id)
        .ok_or("Container object is missing")?;
    crate::object_custody::require_object_at_fixture(ctx, &object, &fixture).map_err(
        |error: crate::object_custody::ObjectCustodyError| -> String { error.to_string() },
    )?;
    object.location =
        crate::inventory_container::carried_location_for_row(&destination, inventory_row_id)
            .map_err(
                |error: crate::inventory_container::InventoryContainerError| -> String {
                    error.to_string()
                },
            )?;
    ctx.db.inventory_object().id().update(object);
    crate::inventory_container::rehome_subtree(ctx, container_object_id, &destination).map_err(
        |error: crate::inventory_container::InventoryContainerError| -> String {
            error.to_string()
        },
    )?;
    ctx.db.fireplace_station().key().delete(key);
    crate::inventory_container::merge_empty_container(ctx, container_object_id).map_err(
        |error: crate::inventory_container::InventoryContainerError| -> String {
            error.to_string()
        },
    )?;
    Ok(())
}

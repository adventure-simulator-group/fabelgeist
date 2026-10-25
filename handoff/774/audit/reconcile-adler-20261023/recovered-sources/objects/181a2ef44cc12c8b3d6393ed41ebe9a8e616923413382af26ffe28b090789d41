//! Physical occupancy history, separate from legal tenure and family membership.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, SpacetimeType)]
pub enum PropertyOccupancyKind {
    Admitted,
    Removed,
}

#[derive(Clone, Debug)]
#[table(accessor = property_occupancy_transition)]
pub struct PropertyOccupancyTransition {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub character_id: u64,
    #[index(btree)]
    pub property_id: String,
    pub holding_id: Option<String>,
    pub kind: PropertyOccupancyKind,
    pub minute: StrategicMinute,
    pub ordinal: u64,
}

fn fold(mut transitions: Vec<PropertyOccupancyTransition>) -> Option<PropertyOccupancyTransition> {
    transitions.sort_by_key(|transition| (transition.minute, transition.ordinal));
    transitions.into_iter().fold(
        None,
        |current: Option<PropertyOccupancyTransition>, transition| match transition.kind {
            PropertyOccupancyKind::Admitted => Some(transition),
            PropertyOccupancyKind::Removed
                if current.as_ref().is_some_and(|home| {
                    home.property_id == transition.property_id
                        && home.holding_id == transition.holding_id
                }) =>
            {
                None
            }
            PropertyOccupancyKind::Removed => current,
        },
    )
}

pub(super) fn admit(
    ctx: &ReducerContext,
    occupant: ResidenceOccupant,
    minute: StrategicMinute,
) -> Result<(), String> {
    let previous = at(ctx, occupant.character_id, minute);
    if previous.as_ref().is_some_and(|home| {
        home.property_id == occupant.property_id && home.holding_id == occupant.holding_id
    }) {
        return Ok(());
    }
    properties::require_property_room_at(ctx, &occupant, minute)?;
    if let Some(previous) = previous {
        record(
            ctx,
            &ResidenceOccupant {
                character_id: occupant.character_id,
                property_id: previous.property_id,
                holding_id: previous.holding_id,
                admitted_minute: previous.minute,
            },
            minute,
            PropertyOccupancyKind::Removed,
        );
    }
    record(ctx, &occupant, minute, PropertyOccupancyKind::Admitted);
    refresh_current(ctx, occupant.character_id);
    Ok(())
}

/// A child inherits an exact parent's occupied home, including a seeded home
/// without a legal holding. Capacity failure does not prevent the birth.
pub(crate) fn inherit_parent_home_at(
    ctx: &ReducerContext,
    child_id: u64,
    parents: [u64; 2],
    minute: StrategicMinute,
    settlement_id: &str,
) -> Result<(), String> {
    let home = parents
        .into_iter()
        .filter_map(|parent| at(ctx, parent, minute))
        .find(|home| {
            ctx.db
                .settlement_property()
                .id()
                .find(&home.property_id)
                .is_some_and(|property| property.settlement_id == settlement_id)
                && home
                    .holding_id
                    .as_ref()
                    .is_none_or(|id| holding_active_at(ctx, id, minute))
        })
        .ok_or("No occupied parental property at the effective minute")?;
    admit(
        ctx,
        ResidenceOccupant {
            character_id: child_id,
            property_id: home.property_id,
            holding_id: home.holding_id,
            admitted_minute: minute,
        },
        minute,
    )
}

pub(super) fn at(
    ctx: &ReducerContext,
    character_id: u64,
    minute: StrategicMinute,
) -> Option<PropertyOccupancyTransition> {
    fold(
        ctx.db
            .property_occupancy_transition()
            .character_id()
            .filter(character_id)
            .filter(|transition| transition.minute <= minute)
            .collect(),
    )
}

pub(super) fn at_view(
    ctx: &ViewContext,
    character_id: u64,
    minute: StrategicMinute,
) -> Option<PropertyOccupancyTransition> {
    fold(
        ctx.db
            .property_occupancy_transition()
            .character_id()
            .filter(character_id)
            .filter(|transition| transition.minute <= minute)
            .collect(),
    )
}

pub(super) fn record(
    ctx: &ReducerContext,
    occupant: &ResidenceOccupant,
    minute: StrategicMinute,
    kind: PropertyOccupancyKind,
) {
    let ordinal = ctx
        .db
        .property_occupancy_transition()
        .character_id()
        .filter(occupant.character_id)
        .count() as u64;
    ctx.db
        .property_occupancy_transition()
        .insert(PropertyOccupancyTransition {
            id: format!("property-occupancy:{}:{ordinal}", occupant.character_id),
            character_id: occupant.character_id,
            property_id: occupant.property_id.clone(),
            holding_id: occupant.holding_id.clone(),
            kind,
            minute,
            ordinal,
        });
}

/// Rebuild the current pointer from the ledger after a backdated write.
pub(super) fn refresh_current(ctx: &ReducerContext, character_id: u64) {
    if let Some(home) = fold(
        ctx.db
            .property_occupancy_transition()
            .character_id()
            .filter(character_id)
            .collect(),
    ) {
        store_current(
            ctx,
            ResidenceOccupant {
                character_id,
                property_id: home.property_id,
                holding_id: home.holding_id,
                admitted_minute: home.minute,
            },
        );
    } else {
        ctx.db
            .residence_occupant()
            .character_id()
            .delete(character_id);
    }
}

/// A delayed admission persists until the next effective move or scoped removal.
pub(super) fn admission_end(
    occupant: &ResidenceOccupant,
    minute: StrategicMinute,
    mut events: Vec<PropertyOccupancyTransition>,
) -> Option<StrategicMinute> {
    events.sort_by_key(|event| (event.minute, event.ordinal));
    events
        .into_iter()
        .find(|event| {
            event.minute > minute
                && (event.kind == PropertyOccupancyKind::Admitted
                    || (event.property_id == occupant.property_id
                        && event.holding_id == occupant.holding_id))
        })
        .map(|event| event.minute)
}

pub(super) fn store_current(ctx: &ReducerContext, occupant: ResidenceOccupant) {
    if ctx
        .db
        .residence_occupant()
        .character_id()
        .find(occupant.character_id)
        .is_some()
    {
        ctx.db.residence_occupant().character_id().update(occupant);
    } else {
        ctx.db.residence_occupant().insert(occupant);
    }
}

/// Count each character once at the requested frontier, including characters
/// with later moves. No tactical observed position is inferred from this home.
pub(super) fn residents_at(
    ctx: &ReducerContext,
    property_id: &str,
    minute: StrategicMinute,
) -> Vec<u64> {
    let characters = ctx
        .db
        .property_occupancy_transition()
        .property_id()
        .filter(property_id)
        .map(|transition| transition.character_id)
        .collect::<std::collections::BTreeSet<_>>();
    characters
        .into_iter()
        .filter(|character_id| {
            at(ctx, *character_id, minute).is_some_and(|home| home.property_id == property_id)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(
        ordinal: u64,
        minute: u64,
        property: &str,
        kind: PropertyOccupancyKind,
    ) -> PropertyOccupancyTransition {
        PropertyOccupancyTransition {
            id: ordinal.to_string(),
            character_id: 1,
            property_id: property.into(),
            holding_id: None,
            kind,
            minute: StrategicMinute::new(minute),
            ordinal,
        }
    }
    #[test]
    fn same_minute_moves_follow_recorded_order_and_delayed_removal_stays_scoped() {
        let events = vec![
            event(0, 0, "a", PropertyOccupancyKind::Admitted),
            event(1, 0, "a", PropertyOccupancyKind::Removed),
            event(2, 0, "b", PropertyOccupancyKind::Admitted),
            event(3, 0, "a", PropertyOccupancyKind::Removed),
        ];
        assert_eq!(fold(events).unwrap().property_id, "b");
    }
    #[test]
    fn later_written_backdated_admission_does_not_replace_a_future_move() {
        let events = vec![
            event(0, 100, "future", PropertyOccupancyKind::Admitted),
            event(1, 20, "past", PropertyOccupancyKind::Admitted),
        ];
        assert_eq!(fold(events).unwrap().property_id, "future");
    }
}

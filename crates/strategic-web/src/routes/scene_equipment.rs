//! Read-only visible equipment for retained strategic character models.
use super::AppState;
use crate::spacetimedb::SqlQuery;
use crate::{
    session::Session,
    spacetimedb::{
        self, CharacterView, EquipmentAnchorKind, EquipmentOccupancy, InventoryLocation,
        InventoryObject, WeaponHolderInstance, WeaponInstance,
    },
};
use adventuresim_core::equipment_presentation::*;
use adventuresim_stdb_client::{CharacterEquippedItem, InventoryItem};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;

const MAX_CHARACTERS_PER_REQUEST: usize = 256;

#[derive(Deserialize)]
struct Request {
    characters: String,
}

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/scene-equipment", get(equipment))
}

async fn equipment(
    State(state): State<AppState>,
    session: Session,
    Query(request): Query<Request>,
) -> Result<Json<Vec<CharacterEquipmentAppearance>>, StatusCode> {
    let actor = session.character_id_u64().ok_or(StatusCode::UNAUTHORIZED)?;
    let ids = request
        .characters
        .split(',')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if ids.is_empty() || ids.len() > MAX_CHARACTERS_PER_REQUEST {
        return Err(StatusCode::BAD_REQUEST);
    }
    let actor = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            spacetimedb::character_by_id(actor.into()),
        )
        .await
        .map_err(unavailable)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let condition = ids
        .iter()
        .map(|id| format!("id = {id}"))
        .collect::<Vec<_>>()
        .join(" OR ");
    let characters = state
        .db
        .query_sats_into::<adventuresim_stdb_client::Character, CharacterView>(SqlQuery::from(
            format!("SELECT * FROM character WHERE {condition}"),
        ))
        .await
        .map_err(unavailable)?;
    let mut grants = session.character_ids();
    if characters
        .iter()
        .any(|target| !visible(&actor, target, &grants))
        && let Some(party) = actor.party_id.as_deref()
    {
        let requests = state
            .db
            .query_sats::<adventuresim_stdb_client::PartyJoinRequest>(SqlQuery::from(format!(
                "SELECT * FROM party_join_request WHERE party_id = {}",
                spacetimedb::sql_string_literal(party)
            )))
            .await
            .map_err(unavailable)?;
        grants.extend(
            requests
                .into_iter()
                .filter(|request| request.party_id == party)
                .map(|request| request.character_id),
        );
    }
    if ids.iter().any(|id| {
        !characters
            .iter()
            .any(|person| person.id == *id && visible(&actor, person, &grants))
    }) {
        return Err(StatusCode::FORBIDDEN);
    }
    appearances(&state, ids).await
}

async fn appearances(
    state: &AppState,
    ids: Vec<u64>,
) -> Result<Json<Vec<CharacterEquipmentAppearance>>, StatusCode> {
    let condition = ids
        .iter()
        .map(|id| format!("character_id = {id}"))
        .collect::<Vec<_>>()
        .join(" OR ");
    let worn_sql = SqlQuery::from(format!(
        "SELECT * FROM character_equipped_item WHERE {condition}"
    ));
    let inventory_sql = SqlQuery::from(format!("SELECT * FROM inventory_item WHERE {condition}"));
    let occupancy_sql = SqlQuery::from(format!(
        "SELECT * FROM equipment_occupancy WHERE {condition}"
    ));

    let (worn, inventory, occupancies, objects) = tokio::join!(
        state.db.query_sats::<CharacterEquippedItem>(worn_sql),
        state.db.query_sats::<InventoryItem>(inventory_sql),
        state.db.query_sats::<EquipmentOccupancy>(occupancy_sql),
        state
            .db
            .query_sats::<InventoryObject>("SELECT * FROM inventory_object".into()),
    );
    let (mut worn, inventory, mut occupancies, objects) = (
        worn.map_err(unavailable)?,
        inventory.map_err(unavailable)?,
        occupancies.map_err(unavailable)?,
        objects.map_err(unavailable)?,
    );
    worn.sort_by_key(|row| row.inventory_item_id);
    occupancies.sort_by_key(|row| {
        (
            row.inventory_item_id,
            row.requirement_index,
            row.capacity_index,
        )
    });
    let (weapons, holders) = recipes(state, &objects, &worn).await?;
    let mut result = Vec::new();
    for id in ids {
        let mut equipment = Vec::new();
        for node in worn.iter().filter(|node| node.character_id == id) {
            let item = inventory
                .iter()
                .find(|item| item.id == node.inventory_item_id && item.character_id == id)
                .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
            let mut appearance = EquipmentAppearance {
                id: PresentationId(item.id.to_string()),
                item: item.item_id.clone(),
                placement: node.placement_id.clone(),
                occupancies: occupancies
                    .iter()
                    .filter(|row| row.inventory_item_id == item.id && row.character_id == id)
                    .map(occupancy)
                    .collect::<Result<_, _>>()?,
                weapon: None,
                holder: None,
            };
            if let Some(object) = objects.iter().find(|object| matches!(&object.location,
                InventoryLocation::Personal(location) if location.character_id == id && location.row_id == item.id)) {
                appearance.weapon = weapons.iter().find(|row| row.physical_object_id == object.id)
                    .map(|row| generated(row.generator_version, row.design_hash.clone(), row.recipe.clone())).transpose()?;
                appearance.holder = holders.iter().find(|row| row.physical_object_id == object.id)
                    .map(|row| generated(row.generator_version, row.design_hash.clone(), row.recipe.clone())).transpose()?;
            }
            equipment.push(appearance);
        }
        result.push(CharacterEquipmentAppearance {
            id: PresentationId(id.to_string()),
            equipment,
        });
    }
    Ok(Json(result))
}

async fn recipes(
    state: &AppState,
    objects: &[InventoryObject],
    worn: &[CharacterEquippedItem],
) -> Result<(Vec<WeaponInstance>, Vec<WeaponHolderInstance>), StatusCode> {
    let condition = objects.iter().filter(|object| matches!(&object.location,
        InventoryLocation::Personal(location) if worn.iter().any(|item|
            item.character_id == location.character_id && item.inventory_item_id == location.row_id)))
        .map(|object| format!("physical_object_id = {}", object.id)).collect::<Vec<_>>().join(" OR ");
    if condition.is_empty() {
        return Ok((vec![], vec![]));
    }
    let weapon_sql = SqlQuery::from(format!("SELECT * FROM weapon_instance WHERE {condition}"));
    let holder_sql = SqlQuery::from(format!(
        "SELECT * FROM weapon_holder_instance WHERE {condition}"
    ));
    let (weapons, holders) = tokio::join!(
        state.db.query_sats::<WeaponInstance>(weapon_sql),
        state.db.query_sats::<WeaponHolderInstance>(holder_sql)
    );
    Ok((weapons.map_err(unavailable)?, holders.map_err(unavailable)?))
}

fn visible(actor: &CharacterView, target: &CharacterView, grants: &[u64]) -> bool {
    grants.contains(&target.id)
        || actor.id == target.id
        || actor
            .party_id
            .as_ref()
            .is_some_and(|party| target.party_id.as_ref() == Some(party))
        || actor
            .current_settlement_id
            .as_ref()
            .is_some_and(|place| target.current_settlement_id.as_ref() == Some(place))
}

fn unavailable(error: impl std::fmt::Display) -> StatusCode {
    tracing::warn!(%error, "could not load strategic equipment presentation");
    StatusCode::SERVICE_UNAVAILABLE
}

fn generated(
    generator_version: u16,
    hash: Vec<u8>,
    recipe: Vec<u8>,
) -> Result<GeneratedAppearance, StatusCode> {
    Ok(GeneratedAppearance {
        generator_version,
        design_hash: hash
            .try_into()
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?,
        recipe,
    })
}

fn occupancy(row: &EquipmentOccupancy) -> Result<AppearanceOccupancy, StatusCode> {
    let anchor = match row.anchor_kind {
        EquipmentAnchorKind::CharacterLocation => {
            AppearanceAnchor::Character(spacetimedb::core_equipment_location(
                row.location.ok_or(StatusCode::SERVICE_UNAVAILABLE)?,
            ))
        }
        EquipmentAnchorKind::ItemAttachment => AppearanceAnchor::Attachment {
            parent: PresentationId(
                row.parent_inventory_item_id
                    .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
                    .to_string(),
            ),
            point: row
                .attachment_point_id
                .clone()
                .ok_or(StatusCode::SERVICE_UNAVAILABLE)?,
        },
    };
    Ok(AppearanceOccupancy {
        anchor,
        channel: spacetimedb::core_equipment_channel(row.channel),
        order: row.order,
        requirement_index: row.requirement_index,
        capacity_index: row.capacity_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn person(id: u64) -> CharacterView {
        CharacterView {
            id,
            name: "Person".into(),
            xp: 0,
            level: 1,
            current_settlement_id: None,
            current_case_site_id: None,
            party_id: None,
            age_years: 30,
            alive: true,
            temporary: false,
            social_notification_count: 0,
            automatic_social_chat_enabled: false,
        }
    }
    #[test]
    fn equipment_scope_requires_ownership_party_or_same_settlement() {
        let mut actor = person(1);
        let mut target = person(2);
        assert!(
            !visible(&actor, &target, &[]),
            "two absent locations do not confer access"
        );
        assert!(visible(&actor, &target, &[2]));
        actor.current_settlement_id = Some("town".into());
        target.current_settlement_id = Some("town".into());
        assert!(visible(&actor, &target, &[]));
        target.current_settlement_id = Some("elsewhere".into());
        assert!(!visible(&actor, &target, &[]));
        actor.party_id = Some("party".into());
        target.party_id = actor.party_id.clone();
        assert!(visible(&actor, &target, &[]));
    }

    #[test]
    fn malformed_equipment_edges_and_recipe_hashes_fail_closed() {
        let row = EquipmentOccupancy {
            id: "edge".into(),
            character_id: 1,
            inventory_item_id: 2,
            anchor_kind: EquipmentAnchorKind::ItemAttachment,
            location: None,
            parent_inventory_item_id: None,
            attachment_point_id: Some("loop".into()),
            channel: adventuresim_stdb_client::EquipmentChannel::Mount,
            order: 0,
            requirement_index: 0,
            capacity_index: 0,
        };
        assert!(occupancy(&row).is_err());
        assert!(generated(1, vec![0; 31], vec![]).is_err());
    }
}

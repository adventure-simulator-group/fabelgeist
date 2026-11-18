//! Equipment form admission and canonical durable command serialization.

use super::*;
use adventuresim_core::{
    equipment_command::{
        EquipmentChange, EquipmentChanges, EquipmentDestination, EquipmentIntent, ParentSelection,
        ParentSelections,
    },
    equipment_placement::{
        EquipmentAttachmentPointId, EquipmentHand, EquipmentParentRequirementIndex,
    },
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in super::super) struct EquipmentForm {
    inventory_item_id: u64,
    destination: EquipmentDestinationForm,
    conflicts: EquipmentConflictForm,
    placement_index: Option<u16>,
    attachment_targets: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EquipmentAttachmentTargetForm {
    requirement_index: u16,
    parent_inventory_item_id: u64,
    attachment_point_id: String,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum EquipmentDestinationForm {
    Inventory,
    HeldLeft,
    HeldRight,
    Authored,
    AdministerPreparation,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum EquipmentConflictForm {
    RejectOccupied,
    ReplaceOccupied,
}

impl EquipmentForm {
    fn request(self) -> Result<EquipmentIntent, String> {
        if self.destination != EquipmentDestinationForm::Authored
            && (self.placement_index.is_some() || self.attachment_targets.is_some())
        {
            return Err("Only authored placements accept attachment selections".into());
        }
        let destination = match self.destination {
            EquipmentDestinationForm::Inventory => EquipmentDestination::Inventory,
            EquipmentDestinationForm::HeldLeft => EquipmentDestination::Held(EquipmentHand::Left),
            EquipmentDestinationForm::HeldRight => EquipmentDestination::Held(EquipmentHand::Right),
            EquipmentDestinationForm::Authored => {
                let placement_index = self
                    .placement_index
                    .ok_or("Choose a complete placement for this item")?;
                let targets: Vec<EquipmentAttachmentTargetForm> = serde_json::from_str(
                    self.attachment_targets
                        .as_deref()
                        .ok_or("Choose attachment targets")?,
                )
                .map_err(|_| "Invalid attachment target selection")?;
                let targets = targets
                    .into_iter()
                    .map(|target| {
                        Ok(ParentSelection {
                            requirement: EquipmentParentRequirementIndex::new(
                                target.requirement_index,
                            ),
                            parent: InventoryRowId::try_new(target.parent_inventory_item_id)
                                .map_err(|error| error.to_string())?,
                            point: EquipmentAttachmentPointId::try_from(target.attachment_point_id)
                                .map_err(|error| error.to_string())?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let targets =
                    ParentSelections::try_from(targets).map_err(|error| error.to_string())?;
                EquipmentDestination::authored(
                    EquipmentPlacementIndex::new(placement_index),
                    targets,
                )
            }
            EquipmentDestinationForm::AdministerPreparation => {
                return Err("Choose an equipment destination".into());
            }
        };
        Ok(EquipmentIntent {
            changes: EquipmentChanges::Single(EquipmentChange {
                inventory: InventoryRowId::try_new(self.inventory_item_id)
                    .map_err(|error| error.to_string())?,
                destination,
            }),
            conflicts: match self.conflicts {
                EquipmentConflictForm::RejectOccupied => EquipmentConflictPolicy::RejectOccupied,
                EquipmentConflictForm::ReplaceOccupied => EquipmentConflictPolicy::ReplaceOccupied,
            },
        })
    }
}

pub(in super::super) async fn set_equipment(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<EquipmentForm>,
) -> Response {
    let Some(character_id) = session.character_id_u64() else {
        return (StatusCode::UNAUTHORIZED, "Choose a character").into_response();
    };
    let inventory: Option<InventoryItem> = match state
        .db
        .query_one_sats(&format!(
            "SELECT * FROM inventory_item WHERE id = {} AND character_id = {character_id}",
            form.inventory_item_id
        ))
        .await
    {
        Ok(inventory) => inventory,
        Err(error) => {
            tracing::warn!(%error, character_id, "failed to load equipment inventory row");
            return (StatusCode::SERVICE_UNAVAILABLE, "Inventory is unavailable").into_response();
        }
    };
    let Some(inventory) = inventory else {
        return (StatusCode::NOT_FOUND, "Item is not in this inventory").into_response();
    };
    let definition: Option<CatalogItemView> = match state
        .db
        .query_one_sats_into::<DbItem, CatalogItemView>(&db::item_by_id(&inventory.item_id))
        .await
    {
        Ok(definition) => definition,
        Err(error) => {
            tracing::warn!(%error, character_id, "failed to load equipment definition");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Equipment catalog is unavailable",
            )
                .into_response();
        }
    };
    let Some(definition) = definition else {
        return (StatusCode::NOT_FOUND, "Item definition is missing").into_response();
    };
    if form.destination == EquipmentDestinationForm::AdministerPreparation {
        if definition.kind != db::CatalogItemKind::Medication {
            return (StatusCode::BAD_REQUEST, "This item is not a preparation").into_response();
        }
        let administration = match standard_medication_administration(
            character_id,
            form.inventory_item_id,
            &inventory.item_id,
            true,
        ) {
            Ok(administration) => administration,
            Err(error) => return (StatusCode::BAD_REQUEST, error).into_response(),
        };
        if let Err(error) = state
            .db
            .call("administer_preparation", &administration.reducer_args())
            .await
        {
            tracing::warn!(
                %error,
                character_id,
                inventory_item_id = form.inventory_item_id,
                "preparation administration rejected"
            );
            return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
        return (StatusCode::NO_CONTENT, "").into_response();
    }
    let request = match form.request() {
        Ok(request) => request,
        Err(error) => return (StatusCode::BAD_REQUEST, error).into_response(),
    };
    let request = adventuresim_stdb_client::EquipmentChangeSetRequest::from(request);
    let request = match adventuresim_stdb_client::json_reducer_argument(&request) {
        Ok(request) => request,
        Err(error) => {
            tracing::error!(%error, "failed to serialize equipment request");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Equipment is unavailable",
            )
                .into_response();
        }
    };
    if let Err(error) = state
        .db
        .call("change_equipment", &[json!(character_id), request])
        .await
    {
        tracing::warn!(%error, character_id, "failed to update equipment");
        return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
    }
    (StatusCode::NO_CONTENT, "").into_response()
}

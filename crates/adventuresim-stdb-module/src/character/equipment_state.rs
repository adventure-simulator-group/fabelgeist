//! Normalized durable equipment state and resolution of its selected anchors.

use adventuresim_core::{
    equipment_placement::{AuthoredPlacementId, EquipmentHand, EquippedPlacement},
    item_catalog::{EquipmentChannel, OccupancyRequirement, ParentRequirement},
};
use spacetimedb::{SpacetimeType, table};

use crate::item::{Item, PersistedEquipmentPlacement};

/// One equipped inventory row, with generic holding distinct from authored wear.
#[derive(Clone, Debug)]
#[table(
    accessor = character_equipped_item, public,
    index(accessor = character_id, btree(columns = [character_id]))
)]
pub struct CharacterEquippedItem {
    #[primary_key]
    pub inventory_item_id: u64,
    pub character_id: u64,
    pub placement: EquippedPlacement,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq, Eq)]
pub struct EquipmentAttachmentTargetSelection {
    pub requirement_index: u16,
    pub parent_inventory_item_id: u64,
    pub attachment_point_id: String,
}

#[derive(Clone, Debug)]
#[table(
    accessor = equipment_occupancy, public,
    index(accessor = character_id, btree(columns = [character_id])),
    index(accessor = inventory_item_id, btree(columns = [inventory_item_id]))
)]
pub struct EquipmentOccupancy {
    /// Stable key for an item's character requirement or an attachment cell.
    #[primary_key]
    pub id: String,
    pub character_id: u64,
    pub inventory_item_id: u64,
    pub anchor_kind: EquipmentAnchorKind,
    pub location: Option<adventuresim_core::item_catalog::EquipmentLocation>,
    pub parent_inventory_item_id: Option<u64>,
    pub attachment_point_id: Option<String>,
    pub channel: EquipmentChannel,
    pub order: u16,
    pub requirement_index: u16,
    pub capacity_index: u16,
}

pub(super) struct ResolvedEquipmentPlacement {
    pub(super) placement: EquippedPlacement,
    pub(super) occupancy: Vec<OccupancyRequirement>,
    pub(super) parents: Vec<ParentRequirement>,
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipmentAnchorKind {
    CharacterLocation,
    ItemAttachment,
}

impl ResolvedEquipmentPlacement {
    pub(super) fn selected(item: &Item, placement: &EquippedPlacement) -> Result<Self, String> {
        match placement {
            EquippedPlacement::Authored(id) => {
                let authored = item
                    .equipment_placements
                    .iter()
                    .find(|candidate| candidate.id == id.as_str())
                    .ok_or("Equipped item has no matching authored placement")?;
                Self::from_authored(id.clone(), authored)
            }
            EquippedPlacement::Held(hand) => Ok(Self::held(*hand)),
        }
    }

    pub(super) fn authored(placement: &PersistedEquipmentPlacement) -> Result<Self, String> {
        let id = AuthoredPlacementId::try_from(placement.id.clone())
            .map_err(|error| error.to_string())?;
        Self::from_authored(id, placement)
    }

    pub(super) fn held(hand: EquipmentHand) -> Self {
        Self {
            placement: EquippedPlacement::Held(hand),
            occupancy: vec![OccupancyRequirement {
                location: hand.location(),
                channel: EquipmentChannel::Held,
                order: 0,
                fit_zone: None,
            }],
            parents: Vec::new(),
        }
    }

    fn from_authored(
        id: AuthoredPlacementId,
        authored: &PersistedEquipmentPlacement,
    ) -> Result<Self, String> {
        if authored
            .occupancy
            .iter()
            .any(|requirement| requirement.channel == EquipmentChannel::Held)
        {
            return Err("Held anchors require a generic hand destination".into());
        }
        Ok(Self {
            placement: EquippedPlacement::Authored(id),
            occupancy: authored.occupancy.clone(),
            parents: authored.parents.clone(),
        })
    }
}

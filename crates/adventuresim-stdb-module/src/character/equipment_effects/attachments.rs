//! Validate attachment requirements and reserve distinct authored capacity cells.

use crate::item::{PersistedEquipmentAttachmentPoint, PersistedEquipmentPlacement};

use super::*;

pub(super) struct SelectedAttachments {
    pub(super) occupancies: Vec<ParentOccupancy>,
    pub(super) displaced: BTreeSet<InventoryRowId>,
}

pub(super) struct ParentOccupancy {
    pub(super) parent: InventoryRowId,
    pub(super) point_id: String,
    pub(super) capacity_index: u16,
    pub(super) requirement_index: u16,
    pub(super) requirement: ParentRequirement,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct AttachmentCell {
    parent: InventoryRowId,
    point_id: String,
    capacity_index: u16,
}

impl SelectedAttachments {
    pub(super) fn resolve(
        ctx: &ReducerContext,
        change: &EquipmentPlacementChange,
        definition: &crate::Item,
        placement: &PersistedEquipmentPlacement,
    ) -> Result<Self> {
        if change.targets.len() != placement.parents.len() {
            return Err(format!(
                "Placement {} requires {} attachment target(s), received {}",
                placement.id,
                placement.parents.len(),
                change.targets.len(),
            ));
        }
        let mut selected_requirements = BTreeSet::new();
        let mut reserved = BTreeSet::new();
        let mut result = Self {
            occupancies: Vec::new(),
            displaced: BTreeSet::new(),
        };
        for target in &change.targets {
            let requirement = placement
                .parents
                .get(usize::from(target.requirement_index))
                .ok_or("Invalid attachment parent requirement")?;
            if !selected_requirements.insert(target.requirement_index) {
                return Err("Attachment parent requirement was selected more than once".into());
            }
            let parent = InventoryRowId::try_new(target.parent_inventory_item_id)
                .map_err(|error| error.to_string())?;
            if parent == change.inventory {
                return Err("An item cannot be attached to itself".into());
            }
            let point = resolve_point(ctx, change.actor, parent, &target.attachment_point_id)?;
            occupancy::validate_attachment_requirement(ctx, parent.get(), &point, *requirement)?;
            if !point.accepts_tags.is_empty()
                && !definition
                    .attachment_tags
                    .iter()
                    .any(|tag| point.accepts_tags.contains(tag))
            {
                return Err(format!(
                    "{} is incompatible with attachment point {}",
                    definition.id, point.id
                ));
            }
            let cell = AttachmentCell::select(ctx, change, parent, &point, &reserved)?;
            if let Some(displaced) = ctx
                .db
                .equipment_occupancy()
                .id()
                .find(attachment_occupancy_id(
                    parent.get(),
                    &point.id,
                    cell.capacity_index,
                ))
                .map(|row| row.inventory_item_id)
                .filter(|item| *item != change.inventory.get())
            {
                result
                    .displaced
                    .insert(InventoryRowId::try_new(displaced).map_err(|error| error.to_string())?);
            }
            result.occupancies.push(ParentOccupancy {
                parent,
                point_id: point.id,
                capacity_index: cell.capacity_index,
                requirement_index: target.requirement_index,
                requirement: *requirement,
            });
            reserved.insert(cell);
        }
        Ok(result)
    }
}

impl AttachmentCell {
    fn select(
        ctx: &ReducerContext,
        change: &EquipmentPlacementChange,
        parent: InventoryRowId,
        point: &PersistedEquipmentAttachmentPoint,
        reserved: &BTreeSet<Self>,
    ) -> Result<Self> {
        let cell_at = |capacity_index| Self {
            parent,
            point_id: point.id.clone(),
            capacity_index,
        };
        let occupant_at = |index| {
            ctx.db
                .equipment_occupancy()
                .id()
                .find(attachment_occupancy_id(parent.get(), &point.id, index))
        };
        let capacity_index =
            first_free_attachment_capacity(point.capacity, change.inventory.get(), |index| {
                if reserved.contains(&cell_at(index)) {
                    return AttachmentCapacity::Reserved;
                }
                occupant_at(index).map_or(AttachmentCapacity::Free, |row| {
                    AttachmentCapacity::Occupied(row.inventory_item_id)
                })
            })
            .or_else(|| {
                if change.conflicts == EquipmentConflictPolicy::RejectOccupied {
                    return None;
                }
                (0..point.capacity).find(|index| {
                    !reserved.contains(&cell_at(*index))
                        && occupant_at(*index)
                            .is_some_and(|row| row.inventory_item_id != change.inventory.get())
                })
            })
            .ok_or_else(|| format!("Attachment point {} is full", point.id))?;
        Ok(cell_at(capacity_index))
    }
}

fn resolve_point(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    parent: InventoryRowId,
    point_id: &str,
) -> Result<PersistedEquipmentAttachmentPoint> {
    ctx.db
        .character_equipped_item()
        .inventory_item_id()
        .find(parent.get())
        .filter(|row| row.character_id == actor.get())
        .ok_or("Parent item is not equipped by this character")?;
    let inventory = ctx
        .db
        .inventory_item()
        .id()
        .find(parent.get())
        .filter(|row| row.character_id == actor.get())
        .ok_or("Parent item is not carried by this character")?;
    let definition = ctx
        .db
        .item()
        .id()
        .find(&inventory.item_id)
        .ok_or("Parent item definition is missing")?;
    definition
        .attachment_points
        .into_iter()
        .find(|point| point.id == point_id)
        .ok_or_else(|| format!("Parent has no attachment point {point_id}"))
}

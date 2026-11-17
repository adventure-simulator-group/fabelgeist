//! Capture the complete owned containment graph for an optimistic read set.

use std::collections::{BTreeMap, BTreeSet};

use adventuresim_core::physical_object::InventoryLocation;
use spacetimedb::{SpacetimeType, ViewContext};

use super::InventoryProjectionError;
use crate::{
    InventoryItem,
    inventory_container::{
        ContainerLiquid, InventoryContainment, InventoryObject, container_liquid__view,
        inventory_containment__view, inventory_object__view,
    },
};

#[derive(SpacetimeType)]
pub(super) struct InventoryFootprint {
    pub(super) objects: Vec<InventoryObject>,
    edges: Vec<InventoryContainment>,
    liquids: Vec<ContainerLiquid>,
}

type ProjectionResult<T> = std::result::Result<T, InventoryProjectionError>;

impl InventoryFootprint {
    pub(super) fn capture(ctx: &ViewContext, character_id: u64, rows: &[InventoryItem]) -> ProjectionResult<Self> {
        let mut objects: BTreeMap<_, _> = ctx.db.inventory_object().item_id().filter(""..)
            .filter(|object| matches!(&object.location, InventoryLocation::Personal(location) if location.character_id == character_id))
            .map(|object| (object.id, object)).collect();
        validate_backing(&objects, rows)?;
        let owned: BTreeSet<_> = objects.keys().copied().collect();
        let edges = capture_edges(ctx, &owned);
        for edge in edges.values() {
            if !owned.contains(&edge.child_object_id) {
                return Err(InventoryProjectionError::InvalidContainment);
            }
            if owned.contains(&edge.parent_object_id) { continue; }
            let parent = ctx.db.inventory_object().id().find(edge.parent_object_id)
                .ok_or(InventoryProjectionError::InvalidContainment)?;
            if !parent.location.is_fireplace() {
                return Err(InventoryProjectionError::InvalidContainment);
            }
            objects.insert(parent.id, parent);
        }
        let liquids = objects.keys().filter_map(|id| {
            ctx.db.container_liquid().container_object_id().find(*id)
        }).collect();
        Ok(Self { objects: objects.into_values().collect(), edges: edges.into_values().collect(), liquids })
    }
}

fn validate_backing(objects: &BTreeMap<u64, InventoryObject>, rows: &[InventoryItem]) -> ProjectionResult<()> {
    let mut backing_rows = BTreeSet::new();
    for object in objects.values() {
        let InventoryLocation::Personal(location) = &object.location else {
            return Err(InventoryProjectionError::InvalidIdentity);
        };
        let row = rows.iter().find(|row| row.id == location.row_id)
            .ok_or(InventoryProjectionError::InvalidIdentity)?;
        if row.item_id != object.item_id || row.quantity != 1 || !backing_rows.insert(row.id) {
            return Err(InventoryProjectionError::InvalidIdentity);
        }
    }
    Ok(())
}

fn capture_edges(ctx: &ViewContext, owned: &BTreeSet<u64>) -> BTreeMap<u64, InventoryContainment> {
    let mut edges = BTreeMap::new();
    for id in owned {
        if let Some(edge) = ctx.db.inventory_containment().child_object_id().find(*id) {
            edges.insert(edge.child_object_id, edge);
        }
        for edge in ctx.db.inventory_containment().parent_object_id().filter(*id) {
            edges.insert(edge.child_object_id, edge);
        }
    }
    edges
}

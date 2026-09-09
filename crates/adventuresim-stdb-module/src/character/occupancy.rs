//! Anatomical equipment reservations are validated before reducer mutations.
use super::character_equipped_item;
use crate::item::{PersistedEquipmentPlacement, inventory_item, item};
use adventuresim_core::item_catalog::OccupancyRequirement;
use spacetimedb::ReducerContext;

pub(super) fn character_occupancy_id(
    character_id: u64,
    inventory_item_id: u64,
    requirement_index: u16,
) -> String {
    format!("character:{character_id}:item:{inventory_item_id}:requirement:{requirement_index}")
}

pub(super) fn conflicting_equipment_roots(
    ctx: &ReducerContext,
    character_id: u64,
    inventory_item_id: u64,
    placement: &PersistedEquipmentPlacement,
) -> Result<Vec<(OccupancyRequirement, u64)>, String> {
    let mut conflicts = Vec::new();
    for equipped in ctx
        .db
        .character_equipped_item()
        .character_id()
        .filter(character_id)
    {
        if equipped.inventory_item_id == inventory_item_id {
            continue;
        }
        let inventory = ctx
            .db
            .inventory_item()
            .id()
            .find(equipped.inventory_item_id)
            .ok_or("Equipped item has no inventory row")?;
        let definition = ctx
            .db
            .item()
            .id()
            .find(inventory.item_id)
            .ok_or("Equipped item has no catalog definition")?;
        let current = definition
            .equipment_placements
            .iter()
            .find(|current| current.id == equipped.placement_id)
            .ok_or("Equipped item has no authored placement")?;
        conflicts.extend(conflicting_requirements(
            placement,
            current,
            equipped.inventory_item_id,
        ));
    }
    Ok(conflicts)
}

fn conflicting_requirements(
    proposed: &PersistedEquipmentPlacement,
    current: &PersistedEquipmentPlacement,
    current_item_id: u64,
) -> Vec<(OccupancyRequirement, u64)> {
    proposed
        .occupancy
        .iter()
        .copied()
        .filter(|requirement| {
            current
                .occupancy
                .iter()
                .any(|other| requirement.conflicts_with(*other))
        })
        .map(|requirement| (requirement, current_item_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::item_catalog;

    #[test]
    fn persisted_harness_placements_coexist_and_overlapping_replacement_finds_every_occupant() {
        let mut harness: Vec<PersistedEquipmentPlacement> = Vec::new();
        for item in [
            "morion",
            "gorget",
            "cuirass",
            "fauld",
            "spaulder",
            "rerebrace",
            "couter",
            "vambrace",
            "mitten_gauntlet",
            "cuisse",
            "poleyn",
            "greave",
            "sabaton",
        ] {
            for placement in &item_catalog::definition(item)
                .unwrap()
                .equipment
                .as_ref()
                .unwrap()
                .placements
            {
                let persisted = PersistedEquipmentPlacement {
                    id: placement.id.clone(),
                    occupancy: placement.occupancy.clone(),
                    parents: placement.parents.clone(),
                    protection: placement.protection.clone(),
                };
                for other in &harness {
                    assert!(
                        conflicting_requirements(&persisted, other, 1).is_empty(),
                        "{item}/{}",
                        placement.id
                    );
                }
                assert!(!conflicting_requirements(&persisted, &persisted, 1).is_empty());
                harness.push(persisted);
            }
        }
        assert_eq!(harness.len(), 22);
        let mut whole_arm = harness
            .iter()
            .find(|placement| {
                placement.occupancy[0].fit_zone == Some(item_catalog::EquipmentFitZone::Forearm)
            })
            .unwrap()
            .clone();
        whole_arm.occupancy[0].fit_zone = None;
        let displaced = harness
            .iter()
            .enumerate()
            .flat_map(|(id, current)| conflicting_requirements(&whole_arm, current, id as u64))
            .collect::<Vec<_>>();
        assert_eq!(
            displaced.len(),
            3,
            "whole arm replaces rerebrace, couter, and vambrace"
        );
    }
}

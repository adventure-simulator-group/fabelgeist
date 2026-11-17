//! Admit checked mass, equipment and parent identities as one immutable batch.

use std::collections::BTreeSet;

use adventuresim_core::{
    combat::WeaponContactParameters, equipment::ParametricWeaponCombatGeometry,
    equipment_placement::EquippedPlacement, inventory_command::InventorySnapshotToken,
    inventory_measurement::InventoryMassKg, physical_object::PhysicalObjectId,
};
use adventuresim_stdb_client::{ConnectedInventory, ConnectedPlayerItem, EquipmentAnchorKind};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::Vec3;

use super::super::projection_equipment::{
    projected_armor, projected_holder_appearance, projected_parametric_weapon,
    projected_weapon_appearance,
};
use super::{InventoryAdmissionError, InventoryAdmissionResult};

pub(in super::super) struct PreparedInventory {
    pub(super) token: InventorySnapshotToken,
    pub(super) items: Vec<PreparedInventoryItem>,
}

pub(super) struct PreparedInventoryItem {
    pub(super) row_id: TacticalInventoryItemId,
    pub(super) source: ConnectedPlayerItem,
    pub(super) quantity: TacticalItemQuantity,
    pub(super) unit_mass: InventoryMassKg,
    pub(super) carried_mass: InventoryMassKg,
    pub(super) object_id: Option<PhysicalObjectId>,
    pub(super) weapon_appearance: Option<WeaponAppearance>,
    pub(super) holder_appearance: Option<WeaponHolderAppearance>,
    pub(super) armor: Option<ArmorItem>,
    pub(super) weapon: Option<WeaponItem>,
    pub(super) physical: Option<TacticalEquipmentPhysical>,
    pub(super) occupancies: Vec<PreparedOccupancy>,
    pub(super) placement: Option<EquippedPlacement>,
}

pub(super) struct PreparedOccupancy {
    pub(super) source: adventuresim_stdb_client::ConnectedEquipmentOccupancy,
    pub(super) anchor: PreparedAnchor,
}

pub(super) enum PreparedAnchor {
    CharacterLocation(adventuresim_core::item_catalog::EquipmentLocation),
    ItemAttachment {
        parent_row_id: TacticalInventoryItemId,
        attachment_point_id: String,
    },
}

impl PreparedInventory {
    pub(in super::super) fn new(
        inventory: &ConnectedInventory,
        contact: WeaponContactParameters,
    ) -> InventoryAdmissionResult<Self> {
        let snapshot = match inventory {
            ConnectedInventory::Available(snapshot) => snapshot,
            ConnectedInventory::Unavailable(reason) => {
                return Err(InventoryAdmissionError::Unavailable(*reason));
            }
        };
        let token = InventorySnapshotToken {
            scope: snapshot
                .scope
                .clone()
                .try_into()
                .map_err(|_| InventoryAdmissionError::InvalidSnapshotToken)?,
            revision: snapshot
                .revision
                .clone()
                .try_into()
                .map_err(|_| InventoryAdmissionError::InvalidSnapshotToken)?,
        };
        let items = &snapshot.items;
        let mut rows = BTreeSet::new();
        let mut objects = BTreeSet::new();
        for item in items {
            if item.inventory_item_id == 0 || !rows.insert(item.inventory_item_id) {
                return Err(InventoryAdmissionError::InvalidRowIdentity);
            }
            if let Some(id) = item.physical_object_id
                && !objects.insert(id)
            {
                return Err(InventoryAdmissionError::InvalidObjectIdentity);
            }
        }
        for item in items {
            validate_topology(item, items)?;
        }
        validate_acyclic(items)?;
        let items = items
            .iter()
            .map(|item| PreparedInventoryItem::new(item, contact))
            .collect::<InventoryAdmissionResult<Vec<_>>>()?;
        let _total_mass = items
            .iter()
            .try_fold(InventoryMassKg::ZERO, |total, item| {
                InventoryMassKg::try_from(total.kilograms() + item.carried_mass.kilograms())
                    .map_err(|_| InventoryAdmissionError::InvalidMass)
            })?;
        Ok(Self { token, items })
    }

    pub(in super::super) fn validate_disjoint<'a>(
        inventories: impl IntoIterator<Item = &'a Self>,
    ) -> InventoryAdmissionResult<()> {
        let mut rows = BTreeSet::new();
        let mut objects = BTreeSet::new();
        for inventory in inventories {
            for item in &inventory.items {
                if !rows.insert(item.row_id) {
                    return Err(InventoryAdmissionError::InvalidRowIdentity);
                }
                if let Some(id) = item.object_id
                    && !objects.insert(id)
                {
                    return Err(InventoryAdmissionError::InvalidObjectIdentity);
                }
            }
        }
        Ok(())
    }
}

impl PreparedInventoryItem {
    fn new(
        source: &ConnectedPlayerItem,
        contact: WeaponContactParameters,
    ) -> InventoryAdmissionResult<Self> {
        let quantity = TacticalItemQuantity::new(source.quantity)
            .ok_or(InventoryAdmissionError::InvalidQuantity)?;
        let unit_mass = InventoryMassKg::try_from(source.unit_mass_kg)
            .map_err(|_| InventoryAdmissionError::InvalidMass)?;
        let carried_mass = InventoryMassKg::try_from(source.carried_mass_kg)
            .map_err(|_| InventoryAdmissionError::InvalidMass)?;
        let object_id = source
            .physical_object_id
            .map(PhysicalObjectId::try_new)
            .transpose()
            .map_err(|_| InventoryAdmissionError::InvalidObjectIdentity)?;
        if object_id.is_some() && quantity.get() != 1 {
            return Err(InventoryAdmissionError::InvalidObjectIdentity);
        }
        let geometry = projected_parametric_weapon(source, contact)?;
        let placement = source
            .selected_placement
            .clone()
            .map(EquippedPlacement::try_from)
            .transpose()
            .map_err(|_| InventoryAdmissionError::InvalidEquipment)?;
        let occupancies = source
            .occupancies
            .iter()
            .map(PreparedOccupancy::new)
            .collect::<InventoryAdmissionResult<_>>()?;
        Ok(Self {
            row_id: TacticalInventoryItemId(source.inventory_item_id),
            source: source.clone(),
            quantity,
            unit_mass,
            carried_mass,
            object_id,
            weapon_appearance: projected_weapon_appearance(source)?,
            holder_appearance: projected_holder_appearance(source)?,
            armor: projected_armor(source, placement.as_ref())?,
            weapon: weapon(source, geometry)?,
            physical: physical(source, geometry),
            occupancies,
            placement,
        })
    }
}

impl PreparedOccupancy {
    fn new(
        source: &adventuresim_stdb_client::ConnectedEquipmentOccupancy,
    ) -> InventoryAdmissionResult<Self> {
        let anchor = match source.anchor_kind {
            EquipmentAnchorKind::CharacterLocation => {
                PreparedAnchor::CharacterLocation(super::super::tactical_equipment_location(
                    source
                        .location
                        .ok_or(InventoryAdmissionError::InvalidTopology)?,
                ))
            }
            EquipmentAnchorKind::ItemAttachment => PreparedAnchor::ItemAttachment {
                parent_row_id: TacticalInventoryItemId(
                    source
                        .parent_inventory_item_id
                        .ok_or(InventoryAdmissionError::InvalidTopology)?,
                ),
                attachment_point_id: source
                    .attachment_point_id
                    .clone()
                    .ok_or(InventoryAdmissionError::InvalidTopology)?,
            },
        };
        Ok(Self {
            source: source.clone(),
            anchor,
        })
    }
}

fn physical(
    item: &ConnectedPlayerItem,
    geometry: Option<ParametricWeaponCombatGeometry>,
) -> Option<TacticalEquipmentPhysical> {
    let mut physical = adventuresim_core::item_catalog::definition(&item.item.id)?
        .equipment
        .as_ref()?
        .physical;
    if let Some(instance) = geometry {
        physical.dimensions_m[1] = instance.total_length_m;
        physical.grip_to_tip_m = instance.grip_to_tip_m;
        physical.anchor_offset_m[1] = -(instance.total_length_m - instance.grip_to_tip_m);
    }
    Some(TacticalEquipmentPhysical {
        dimensions_m: Vec3::from_array(physical.dimensions_m),
        grip_to_tip_m: physical.grip_to_tip_m,
        striking_head_length_m: geometry.map_or(
            physical.dimensions_m[0].max(physical.dimensions_m[2]),
            |value| value.striking_head_length_m,
        ),
        anchor_offset_m: Vec3::from_array(physical.anchor_offset_m),
    })
}

fn weapon(
    item: &ConnectedPlayerItem,
    geometry: Option<ParametricWeaponCombatGeometry>,
) -> InventoryAdmissionResult<Option<WeaponItem>> {
    if item.item.kind != adventuresim_stdb_client::CatalogItemKind::Weapon {
        return Ok(None);
    }
    let equipment = adventuresim_core::item_catalog::definition(&item.item.id)
        .and_then(|definition| definition.equipment.as_ref())
        .ok_or(InventoryAdmissionError::InvalidEquipment)?;
    Ok(Some(WeaponItem {
        striking_material: equipment
            .striking_material
            .ok_or(InventoryAdmissionError::InvalidEquipment)?,
        skill_weights: [
            item.item.weapon_skills.polearm,
            item.item.weapon_skills.axe,
            item.item.weapon_skills.bludgeon,
            item.item.weapon_skills.sword,
            item.item.weapon_skills.knife,
            item.item.weapon_skills.bow,
            item.item.weapon_skills.crossbow,
            item.item.weapon_skills.firearm,
            item.item.weapon_skills.throw,
        ],
        prefers_stab: matches!(
            item.item.preferred_melee_style,
            adventuresim_stdb_client::MeleeAttackStyle::Stab
        ),
        precision: geometry.map_or(item.item.precision, |value| {
            value.conditioned_precision(&item.item.id, item.item.precision)
        }),
        reach: geometry.map_or(item.item.reach, |value| value.melee_reach_m()),
        grip_to_tip_m: geometry.map_or(equipment.physical.grip_to_tip_m, |value| {
            value.grip_to_tip_m
        }),
        moment_of_inertia_kg_m2: geometry.map_or(item.item.moment_of_inertia_kg_m_2, |value| {
            value.moment_of_inertia_kg_m2
        }),
        melee: item.item.melee,
        ranged: item.item.ranged,
    }))
}

fn validate_acyclic(items: &[ConnectedPlayerItem]) -> InventoryAdmissionResult<()> {
    let mut pending: BTreeSet<_> = items.iter().map(|item| item.inventory_item_id).collect();
    while !pending.is_empty() {
        let resolved: Vec<_> = items
            .iter()
            .filter(|item| pending.contains(&item.inventory_item_id))
            .filter(|item| {
                item.occupancies.iter().all(|occupancy| {
                    occupancy
                        .parent_inventory_item_id
                        .is_none_or(|parent| !pending.contains(&parent))
                })
            })
            .map(|item| item.inventory_item_id)
            .collect();
        if resolved.is_empty() {
            return Err(InventoryAdmissionError::InvalidTopology);
        }
        for row in resolved {
            pending.remove(&row);
        }
    }
    Ok(())
}

fn validate_topology(
    item: &ConnectedPlayerItem,
    items: &[ConnectedPlayerItem],
) -> InventoryAdmissionResult<()> {
    for occupancy in &item.occupancies {
        match occupancy.anchor_kind {
            EquipmentAnchorKind::CharacterLocation => {
                if occupancy.location.is_none()
                    || occupancy.parent_inventory_item_id.is_some()
                    || occupancy.attachment_point_id.is_some()
                {
                    return Err(InventoryAdmissionError::InvalidTopology);
                }
            }
            EquipmentAnchorKind::ItemAttachment => {
                let parent = occupancy
                    .parent_inventory_item_id
                    .filter(|id| *id != item.inventory_item_id)
                    .and_then(|id| {
                        items
                            .iter()
                            .find(|candidate| candidate.inventory_item_id == id)
                    })
                    .ok_or(InventoryAdmissionError::InvalidTopology)?;
                let point = occupancy
                    .attachment_point_id
                    .as_ref()
                    .filter(|point| !point.is_empty())
                    .ok_or(InventoryAdmissionError::InvalidTopology)?;
                let equipment = adventuresim_core::item_catalog::definition(&parent.item.id)
                    .and_then(|definition| definition.equipment.as_ref())
                    .ok_or(InventoryAdmissionError::InvalidTopology)?;
                if occupancy.location.is_some()
                    || !equipment
                        .attachment_points
                        .iter()
                        .any(|attachment| attachment.id == *point)
                {
                    return Err(InventoryAdmissionError::InvalidTopology);
                }
            }
        }
    }
    Ok(())
}

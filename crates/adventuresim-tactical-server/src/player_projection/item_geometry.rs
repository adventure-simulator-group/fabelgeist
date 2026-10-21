//! Native inventory observations become checked physical projections before commands.

use adventuresim_stdb_client::{CatalogItemKind, ConnectedPlayerItem};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;
use fabelgeist_geometry::{CuboidDimensions, SignedLength};

use super::projection_equipment::{
    EquipmentProjectionError, EquipmentProjectionResult, projected_holder_mass,
    projected_parametric_weapon,
};

pub(super) enum PreparedInventoryItem {
    Empty(ConnectedPlayerItem),
    Admitted {
        item: ConnectedPlayerItem,
        quantity: TacticalItemQuantity,
        geometry: PreparedItemGeometry,
    },
}

pub(super) struct PreparedItemGeometry {
    pub(super) mass: f32,
    pub(super) physical: Option<TacticalEquipmentPhysical>,
    pub(super) weapon: Option<WeaponItem>,
}

impl PreparedInventoryItem {
    pub(super) fn observe(
        item: ConnectedPlayerItem,
        parameters: adventuresim_core::combat::WeaponContactParameters,
    ) -> EquipmentProjectionResult<Self> {
        let Some(quantity) = TacticalItemQuantity::new(item.quantity) else {
            return Ok(Self::Empty(item));
        };
        let geometry = PreparedItemGeometry::observe(&item, parameters)?;
        Ok(Self::Admitted { item, quantity, geometry })
    }

    pub(super) fn inventory_id(&self) -> u64 {
        match self {
            Self::Empty(item) | Self::Admitted { item, .. } => item.inventory_item_id,
        }
    }
}

impl PreparedItemGeometry {
    fn observe(
        item: &ConnectedPlayerItem,
        parameters: adventuresim_core::combat::WeaponContactParameters,
    ) -> EquipmentProjectionResult<Self> {
        let instance = projected_parametric_weapon(item, parameters)?;
        let mass = instance.map(|value| value.mass_kg)
            .or_else(|| projected_holder_mass(item)).unwrap_or(item.item.weight);
        let equipment = adventuresim_core::item_catalog::definition(&item.item.id)
            .and_then(|definition| definition.equipment.as_ref());
        let physical = equipment.map(|equipment| -> EquipmentProjectionResult<_> {
            let mut physical = equipment.physical;
            if let Some(instance) = instance {
                physical.dimensions_m = CuboidDimensions::from_axis_lengths(
                    physical.dimensions_m.x_length(), instance.total_length_m,
                    physical.dimensions_m.z_length(),
                )?;
                physical.grip_to_tip_m = instance.grip_to_tip_m;
                // This is the final native attachment-translation kernel.
                physical.anchor_offset_m[1] = -(instance.total_length_m.metres()
                    - instance.grip_to_tip_m.metres());
            }
            Ok(TacticalEquipmentPhysical {
                dimensions_m: physical.dimensions_m,
                grip_to_tip_m: physical.grip_to_tip_m,
                striking_head_length_m: instance.map_or(
                    physical.dimensions_m.x_length().max(physical.dimensions_m.z_length()),
                    |value| value.striking_head_length_m,
                ),
                anchor_offset_m: Vec3::from_array(physical.anchor_offset_m),
            })
        }).transpose()?;
        let weapon = if item.item.kind == CatalogItemKind::Weapon {
            let equipment = equipment.ok_or_else(|| EquipmentProjectionError::MissingEquipment {
                item_id: item.item.id.clone(),
            })?;
            Some(WeaponItem {
                striking_material: equipment.striking_material.ok_or_else(||
                    EquipmentProjectionError::MissingStrikingMaterial { item_id: item.item.id.clone() })?,
                skill_weights: [
                    item.item.weapon_skills.polearm, item.item.weapon_skills.axe,
                    item.item.weapon_skills.bludgeon, item.item.weapon_skills.sword,
                    item.item.weapon_skills.knife, item.item.weapon_skills.bow,
                    item.item.weapon_skills.crossbow, item.item.weapon_skills.firearm,
                    item.item.weapon_skills.throw,
                ],
                prefers_stab: matches!(item.item.preferred_melee_style,
                    adventuresim_stdb_client::MeleeAttackStyle::Stab),
                precision: instance.map_or(item.item.precision, |value|
                    value.conditioned_precision(&item.item.id, item.item.precision)),
                reach: match instance {
                    Some(instance) => instance.melee_reach_m(),
                    None => SignedLength::from_metres(item.item.reach)?,
                },
                grip_to_tip_m: instance.map_or(equipment.physical.grip_to_tip_m,
                    |value| value.grip_to_tip_m),
                moment_of_inertia_kg_m2: instance.map_or(item.item.moment_of_inertia_kg_m_2,
                    |value| value.moment_of_inertia_kg_m2),
                melee: item.item.melee,
                ranged: item.item.ranged,
            })
        } else { None };
        Ok(Self { mass, physical, weapon })
    }

}

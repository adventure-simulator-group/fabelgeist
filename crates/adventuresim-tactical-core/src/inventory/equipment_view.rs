//! Combat-facing equipment calculations over current owned inventory rows.
//!
//! Carried mass includes every unit in a row. Weapon work uses the mass of the
//! equipped individual item. Ownership comes from `ItemOf`, so transfer and
//! dropping change the actor whose carried load includes that row.

use adventuresim_core::{
    body::{BodyPart, BodySide},
    combat_style::MeleeAttackStyle,
    elapsed::{AnimationElapsed, ElapsedResult},
    item_catalog_schema::EquipmentMaterial,
    prelude::PlayerEquipment,
};

use crate::animation::AttackHand;

use super::{EquipSlot, InventoryView};

impl PlayerEquipment for InventoryView<'_, '_, '_> {
    fn weapon_skill_distribution(&self) -> adventuresim_core::equipment::WeaponSkillDistribution {
        let Some(w) = self
            .equipped_weapon()
            .and_then(|item| item.weapon)
            .map(|weapon| weapon.skill_weights)
        else {
            return adventuresim_core::equipment::WeaponSkillDistribution::UNARMED;
        };
        adventuresim_core::equipment::WeaponSkillDistribution {
            polearm: w[0],
            axe: w[1],
            bludgeon: w[2],
            sword: w[3],
            knife: w[4],
            bow: w[5],
            crossbow: w[6],
            firearm: w[7],
            throw: w[8],
        }
    }
    fn weapon_preferred_melee_style(&self) -> MeleeAttackStyle {
        if self
            .equipped_weapon()
            .and_then(|item| item.weapon)
            .is_some_and(|weapon| weapon.prefers_stab)
        {
            MeleeAttackStyle::Stab
        } else {
            MeleeAttackStyle::Swing
        }
    }

    fn weapon_is_melee(&self) -> bool {
        self.equipped_weapon()
            .and_then(|item| item.weapon)
            .is_none_or(|weapon| weapon.melee)
    }

    fn weapon_is_ranged(&self) -> bool {
        self.equipped_weapon()
            .and_then(|item| item.weapon)
            .is_some_and(|weapon| weapon.ranged)
    }

    fn weapon_is_unarmed(&self) -> bool {
        self.equipped_weapon()
            .and_then(|item| item.weapon)
            .is_none()
    }

    fn weapon_holding_side(&self) -> Option<BodySide> {
        Some(match self.attack_hand {
            AttackHand::Main => BodySide::Right,
            AttackHand::Offhand => BodySide::Left,
        })
    }

    fn weapon_reach(&self) -> f32 {
        self.equipped_weapon()
            .and_then(|item| item.weapon)
            .map(|weapon| weapon.reach)
            .unwrap_or(0.0)
    }

    fn weapon_grip_to_tip(&self) -> f32 {
        self.equipped_weapon()
            .and_then(|item| item.physical)
            .map_or_else(|| self.weapon_reach(), |physical| physical.grip_to_tip_m)
    }

    fn weapon_total_length(&self) -> f32 {
        self.equipped_weapon()
            .and_then(|item| item.physical)
            .map_or_else(
                || self.weapon_grip_to_tip(),
                |physical| physical.dimensions_m.y,
            )
    }

    fn weapon_striking_head_length(&self) -> f32 {
        self.equipped_weapon()
            .and_then(|item| item.physical)
            .map_or(0.0, |physical| physical.striking_head_length_m)
    }

    fn weapon_body_material(&self) -> Option<EquipmentMaterial> {
        let item_id = self.equipped_weapon()?.properties.id.as_str();
        adventuresim_core::item_catalog::definition(item_id)
            .and_then(|definition| definition.equipment.as_ref())
            .and_then(|equipment| equipment.material)
    }

    fn weapon_striking_material(&self) -> Option<EquipmentMaterial> {
        self.equipped_weapon()
            .and_then(|item| item.weapon)
            .map(|weapon| weapon.striking_material)
    }

    fn weapon_windup_secs(&self) -> ElapsedResult<AnimationElapsed> {
        Ok(self
            .melee_timing_for(self.weapon_preferred_melee_style())?
            .preparation_secs)
    }

    fn weapon_windup_secs_for(&self, style: MeleeAttackStyle) -> ElapsedResult<AnimationElapsed> {
        Ok(self.melee_timing_for(style)?.preparation_secs)
    }

    fn weapon_recovery_secs(&self) -> ElapsedResult<AnimationElapsed> {
        Ok(self
            .melee_timing_for(self.weapon_preferred_melee_style())?
            .recovery_secs)
    }

    fn weapon_recovery_secs_for(&self, style: MeleeAttackStyle) -> ElapsedResult<AnimationElapsed> {
        Ok(self.melee_timing_for(style)?.recovery_secs)
    }

    fn weapon_balance(&self) -> f32 {
        self.striking_item()
            .and_then(|item| {
                item.weapon.map(|weapon| {
                    adventuresim_core::equipment::weapon_balance_from_moment(
                        weapon.moment_of_inertia_kg_m2,
                        item.properties.weight,
                        weapon.grip_to_tip_m,
                    )
                })
            })
            .unwrap_or(0.0)
    }

    fn weapon_moment_of_inertia(&self) -> f32 {
        self.equipped_weapon()
            .and_then(|item| item.weapon)
            .map_or(0.0, |weapon| weapon.moment_of_inertia_kg_m2)
    }

    fn armor_range_of_motion(&self, part: BodyPart) -> f32 {
        self.layered_armor_for(part).range_of_motion
    }

    fn inventory_weight(&self) -> f32 {
        self.iter()
            .map(|item| item.mass.carried_kilograms(item.properties, *item.quantity))
            .sum()
    }

    fn shield_block_bonus(&self) -> f32 {
        self.equipped_shield()
            .and_then(|item| item.shield)
            .map(|shield| shield.block)
            .unwrap_or_default()
    }

    fn shield_holding_side(&self) -> Option<BodySide> {
        self.equipped_shield()
            .and_then(|item| item.slot)
            .and_then(|slot| match slot {
                EquipSlot::HoldingLeft => Some(BodySide::Left),
                EquipSlot::HoldingRight => Some(BodySide::Right),
                _ => None,
            })
    }

    fn weapon_weight(&self) -> f32 {
        self.striking_item()
            .map(|item| item.properties.weight)
            .unwrap_or_default()
    }

    fn weapon_precision(&self) -> f32 {
        self.equipped_weapon()
            .and_then(|item| item.weapon)
            .map(|weapon| weapon.precision)
            .unwrap_or_default()
    }

    fn armor_resistance(&self, part: BodyPart) -> f32 {
        self.layered_armor_for(part).resistance
    }

    fn armor_padding(&self, part: BodyPart) -> f32 {
        self.layered_armor_for(part).padding
    }

    fn armor_flexibility(&self, part: BodyPart) -> f32 {
        self.layered_armor_for(part).flexibility
    }

    fn armor_coverage(&self, part: BodyPart) -> f32 {
        self.layered_armor_for(part).coverage
    }

    fn armor_surface(
        &self,
        part: BodyPart,
        sample: f32,
    ) -> Option<adventuresim_core::equipment::ArmorSurface> {
        self.armor_layer_chain(part, sample)
            .into_iter()
            .find(|layer| layer.selected)
            .map(|layer| layer.surface)
    }
}

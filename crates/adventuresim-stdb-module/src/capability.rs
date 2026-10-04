use adventuresim_core::identity::CharacterId;
use adventuresim_core::item_catalog::{EquipmentBodyPart, EquipmentChannel, EquipmentLocation};
use adventuresim_core::physical_object::{CarriedInventoryScope, OperationalCustody};
use adventuresim_core::prelude::*;
use adventuresim_core::projectile::ProjectileKind;
use spacetimedb::{ReducerContext, Table, reducer, table};

use crate::character::{character_equipped_item as _, equipment_occupancy as _};
use crate::condition::character_condition as _;
use crate::food::food_lot as _;
use crate::item::item as _;
use crate::repair::item_condition as _;
use crate::{
    CatalogItemKind, CharacterAttributes, CharacterLimbs, CharacterSkills, CharacterStats,
    InventoryItem, Item, character_attributes, character_limbs, character_skills, character_stats,
    character_strategic_condition, inventory_item,
};

mod armor;
mod equipment_projection;
mod error;
mod evaluation;
mod inputs;
pub(crate) use evaluation::{evaluate_character, load_combatant};
use inputs::CapabilityInputs;
mod refresh;
use error::CapabilityComponent;
pub(crate) use error::CapabilityEvaluationError;
pub(crate) use refresh::refresh_character_capability;
mod mass;
use armor::*;
use equipment_projection::combat_weapon;

#[derive(Clone, Debug, PartialEq)]
#[table(accessor = character_capability)]
pub struct CharacterCapability {
    #[primary_key]
    pub character_id: u64,
    pub melee: bool,
    pub ranged: bool,
    pub heavy: bool,
    pub quarter_armor: bool,
    pub half_armor: bool,
    pub three_quarter_armor: bool,
    pub full_armor: bool,
    pub athletics: f32,
    pub endurance: f32,
    pub physiology: f32,
    pub knife: f32,
    pub tailoring: f32,
    pub surgery: f32,
    pub command: f32,
    pub religion: f32,
    #[default(0.0)]
    pub weapon_precision: f32,
    #[default(0u64)]
    pub autoresolve_combat_power: u64,
}

impl From<(u64, CharacterCapabilities)> for CharacterCapability {
    fn from((character_id, value): (u64, CharacterCapabilities)) -> Self {
        Self {
            character_id,
            melee: value.melee,
            ranged: value.ranged,
            heavy: value.heavy,
            quarter_armor: value.quarter_armor,
            half_armor: value.half_armor,
            three_quarter_armor: value.three_quarter_armor,
            full_armor: value.full_armor,
            athletics: value.athletics,
            endurance: value.endurance,
            physiology: value.physiology,
            knife: value.knife,
            tailoring: value.tailoring,
            surgery: value.surgery,
            command: value.command,
            religion: value.religion,
            weapon_precision: value.weapon_precision,
            autoresolve_combat_power: 0,
        }
    }
}

#[reducer]
pub fn refresh_capabilities(ctx: &ReducerContext, character_id: u64) -> Result<(), String> {
    refresh_character_capability(ctx, (character_id).into())
        .map(|_| ())
        .map_err(|error: CapabilityEvaluationError| error.to_string())
}

impl PlayerBody for CharacterLimbs {
    fn body_part_health(&self, part: BodyPart) -> f32 {
        match part {
            BodyPart::LeftArm => self.left_arm_health,
            BodyPart::RightArm => self.right_arm_health,
            BodyPart::LeftLeg => self.left_leg_health,
            BodyPart::RightLeg => self.right_leg_health,
            BodyPart::Chest => self.chest_health,
            BodyPart::Stomach => self.stomach_health,
            BodyPart::Head => self.head_health,
        }
    }

    fn body_weight(&self) -> f32 {
        70.0
    }

    fn primary_side(&self) -> BodySide {
        BodySide::Right
    }
}

impl PlayerEssentials for CharacterStats {
    fn calories_used_today(&self) -> f32 {
        self.calories_used
    }

    fn focus_level(&self) -> f32 {
        self.focus
    }
}

impl PlayerAttributes for CharacterAttributes {
    fn raw_limb_attr(&self, attr: LimbAttribute, limb: BodyPart) -> f32 {
        self.values().raw_limb_attr(attr, limb)
    }

    fn raw_single_body_part_attr(&self, attr: SimpleAttribute) -> f32 {
        self.values().raw_single_body_part_attr(attr)
    }
}

impl PlayerSkills for CharacterSkills {
    fn skill_hours_trained(&self, skill: Skill) -> f32 {
        match skill {
            Skill::Polearm => self.polearm_hours,
            Skill::Axe => self.axe_hours,
            Skill::Bludgeon => self.bludgeon_hours,
            Skill::Sword => self.sword_hours,
            Skill::Knife => self.knife_hours,
            Skill::Block => self.block_hours,
            Skill::Dodge => self.dodge_hours,
            Skill::Bow => self.bow_hours,
            Skill::Crossbow => self.crossbow_hours,
            Skill::Firearm => self.firearm_hours,
            Skill::Throw => self.throw_hours,
            Skill::Will => self.will_hours,
            Skill::Insight => self.insight_hours,
            Skill::Charm => self.charm_hours,
            Skill::Command => self.command_hours,
            Skill::Deception => self.deception_hours,
            Skill::Physiology => self.physiology_hours,
            Skill::Cooking => self.cooking_hours,
            Skill::Herbalism => self.herbalism_hours,
            // Generic recruitment/tactical summaries use the character's best-covered
            // tradition. Authoritative religious morale always selects a tradition.
            Skill::Religion => self.religion_hours.maximum_effective(),
            Skill::Bestiary => self.bestiary_hours.aggregate_effective(),
            Skill::Surgery => self.surgery_hours,
            Skill::Stealth => self.stealth_hours,
            Skill::Balance => self.balance_hours,
            Skill::TerrainPlains => self.terrain_plains_hours,
            Skill::TerrainForest => self.terrain_forest_hours,
            Skill::TerrainHills => self.terrain_hills_hours,
            Skill::TerrainWetlands => self.terrain_wetlands_hours,
            Skill::TerrainUrban => self.terrain_urban_hours,
            Skill::TerrainSnow => self.terrain_snow_hours,
            Skill::Tailoring => self.tailoring_hours,
            Skill::Smithing => self.smithing_hours,
        }
    }

    fn bestiary_hours_for(&self, category: adventuresim_world_schema::BestiaryCategory) -> f32 {
        self.bestiary_hours.effective(category)
    }
}

pub(crate) struct StrategicEquipment {
    hands: [Option<Item>; 2],
    weapon: Option<Item>,
    weapon_side: Option<BodySide>,
    melee_weapon: Option<Item>,
    melee_weapon_inventory_id: Option<u64>,
    melee_weapon_geometry: Option<adventuresim_core::equipment::ParametricWeaponCombatGeometry>,
    melee_weapon_side: Option<BodySide>,
    ranged_weapon: Option<Item>,
    ranged_weapon_inventory_id: Option<u64>,
    ranged_weapon_side: Option<BodySide>,
    ammunition: u32,
    shield: Option<Item>,
    shield_inventory_id: Option<u64>,
    shield_side: Option<BodySide>,
    armor: [adventuresim_core::equipment::LayeredArmor; 7],
    armor_inventory_item_ids: [Option<u64>; 7],
    armor_materials: [Option<adventuresim_core::item_catalog::EquipmentMaterial>; 7],
    armor_coverage_geometry: [Option<adventuresim_core::combat::AuthoredArmorCoverage>; 7],
    survival_clothing: adventuresim_core::survival::ClothingExposure,
    inventory_weight: f32,
}

impl StrategicEquipment {
    pub(crate) fn load(ctx: &ReducerContext, character_id: CharacterId) -> Self {
        let definition = |inventory_id| effective_item_definition(ctx, inventory_id);
        let normalized_hand = |location| {
            ctx.db
                .equipment_occupancy()
                .character_id()
                .filter(u64::from(character_id))
                .find(|row| row.location == Some(location) && row.channel == EquipmentChannel::Held)
                .map(|row| row.inventory_item_id)
        };
        let hand_inventory_ids = [
            normalized_hand(EquipmentLocation::LeftHand),
            normalized_hand(EquipmentLocation::RightHand),
        ];
        let hands = [
            definition(hand_inventory_ids[0]),
            definition(hand_inventory_ids[1]),
        ];
        let weapon_index = hands.iter().position(|item| {
            item.as_ref()
                .is_some_and(|item| item.kind == CatalogItemKind::Weapon)
        });
        let weapon = weapon_index.and_then(|index| hands[index].clone());
        let weapon_side = weapon_index.map(|index| {
            if index == 0 {
                BodySide::Left
            } else {
                BodySide::Right
            }
        });
        let shield_index = hands.iter().position(|item| {
            item.as_ref()
                .is_some_and(|item| item.kind == CatalogItemKind::Shield)
        });
        let shield = shield_index.and_then(|index| hands[index].clone());
        let shield_inventory_id = shield_index.and_then(|index| hand_inventory_ids[index]);
        let shield_side = shield_index.map(hand_side);
        let melee_weapon = hands
            .iter()
            .flatten()
            .find(|item| item.kind == CatalogItemKind::Weapon && item.melee)
            .cloned();
        let melee_weapon_side = hands
            .iter()
            .position(|item| {
                item.as_ref()
                    .is_some_and(|item| item.kind == CatalogItemKind::Weapon && item.melee)
            })
            .map(hand_side);
        let melee_weapon_inventory_id = hands
            .iter()
            .position(|item| {
                item.as_ref()
                    .is_some_and(|item| item.kind == CatalogItemKind::Weapon && item.melee)
            })
            .and_then(|index| hand_inventory_ids[index]);
        let melee_weapon_geometry = melee_weapon
            .as_ref()
            .zip(melee_weapon_inventory_id)
            .and_then(|(item, inventory_id)| {
                crate::weapon_instance::combat_geometry(ctx, inventory_id, &item.id)
            });
        if let Some(item) = &melee_weapon
            && adventuresim_weapon_model::default_design(&item.id).is_some()
            && melee_weapon_geometry.is_none()
        {
            panic!("parametric weapon {} has no valid physical recipe", item.id);
        }
        let ranged_weapon = hands
            .iter()
            .flatten()
            .find(|item| item.kind == CatalogItemKind::Weapon && item.ranged)
            .cloned();
        let ranged_weapon_side = hands
            .iter()
            .position(|item| {
                item.as_ref()
                    .is_some_and(|item| item.kind == CatalogItemKind::Weapon && item.ranged)
            })
            .map(hand_side);
        let ranged_weapon_inventory_id = hands
            .iter()
            .position(|item| {
                item.as_ref()
                    .is_some_and(|item| item.kind == CatalogItemKind::Weapon && item.ranged)
            })
            .and_then(|index| hand_inventory_ids[index]);
        let ammunition = ctx
            .db
            .inventory_item()
            .character_id()
            .filter(u64::from(character_id))
            .filter(|inventory| inventory.item_id == "arrow")
            .filter(|inventory| {
                !crate::inventory_container::row_is_fireplace_rooted(
                    ctx,
                    CarriedInventoryScope::Personal,
                    inventory.id,
                )
            })
            .map(|inventory| inventory.quantity)
            .sum();
        let mut armor = [adventuresim_core::equipment::LayeredArmor::default(); 7];
        let mut armor_inventory_item_ids = [None; 7];
        let mut armor_materials = [None; 7];
        let mut armor_coverage_geometry = [None; 7];
        let mut survival_layers = Vec::new();
        let mut weatherproofing_total = 0_u32;
        let mut peripheral_protection_bps = [0_u16; 4];
        for part in BodyPart::FULL_BODY.iter() {
            let pieces = wearable_protection_for_part(ctx, character_id, part);
            survival_layers.extend(pieces.iter().map(|piece| (piece.padding, piece.coverage)));
            if let Some(piece) =
                adventuresim_core::equipment::outermost_wearable(part, pieces.iter().copied())
            {
                let part_index = body_part_index(part);
                armor_inventory_item_ids[part_index] = Some(piece.inventory_item_id);
                armor_materials[part_index] = armor_material(ctx, piece.inventory_item_id);
                armor_coverage_geometry[part_index] =
                    equipped_armor_coverage(ctx, piece.inventory_item_id, part);
                let protection = adventuresim_core::survival::weatherproofing_from_outer_layer(
                    piece.resistance,
                    piece.coverage,
                );
                weatherproofing_total = weatherproofing_total.saturating_add(u32::from(protection));
                let peripheral_index = match part {
                    BodyPart::LeftArm => Some(0),
                    BodyPart::RightArm => Some(1),
                    BodyPart::LeftLeg => Some(2),
                    BodyPart::RightLeg => Some(3),
                    _ => None,
                };
                if let Some(index) = peripheral_index {
                    peripheral_protection_bps[index] = protection;
                }
            }
            armor[body_part_index(part)] =
                adventuresim_core::equipment::aggregate_layered_armor(part, pieces);
        }
        let dry_inventory_weight = mass::dry_inventory_weight(ctx, character_id);
        let personal_custody = OperationalCustody::character((u64::from(character_id)).into())
            .expect("persisted character identities must be nonzero");
        let contained_water_weight =
            crate::inventory_container::contained_water_ml(ctx, &personal_custody)
                .map_or(f32::INFINITY, |water_ml| water_ml as f32 / 1_000.0);
        Self {
            hands,
            weapon,
            weapon_side,
            melee_weapon,
            melee_weapon_inventory_id,
            melee_weapon_geometry,
            melee_weapon_side,
            ranged_weapon,
            ranged_weapon_inventory_id,
            ranged_weapon_side,
            ammunition,
            shield,
            shield_inventory_id,
            shield_side,
            armor,
            armor_inventory_item_ids,
            armor_materials,
            armor_coverage_geometry,
            survival_clothing: adventuresim_core::survival::ClothingExposure {
                insulation_bps: adventuresim_core::survival::insulation_from_layers(
                    survival_layers,
                ),
                // Coverage scales continuously across the seven stable body
                // regions; resistance is capped at leather equivalence.
                weatherproofing_bps: (weatherproofing_total / 7)
                    .min(u32::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE))
                    as u16,
                peripheral_protection_bps,
            },
            inventory_weight: dry_inventory_weight + contained_water_weight,
        }
    }

    pub(crate) fn combat_training_profile(
        &self,
    ) -> adventuresim_core::strategic_schedule::CombatTrainingProfile {
        use adventuresim_core::strategic_schedule::EquippedCombatItem;
        adventuresim_core::strategic_schedule::CombatTrainingProfile::from_equipped_hands(
            self.hands.iter().flatten().map(|item| EquippedCombatItem {
                weapons: item.weapon_skills,
                shield: item.kind == CatalogItemKind::Shield,
                balance: item.balance,
            }),
        )
    }

    fn armor_for(&self, part: BodyPart) -> adventuresim_core::equipment::LayeredArmor {
        self.armor[body_part_index(part)]
    }

    pub(crate) fn survival_clothing(&self) -> adventuresim_core::survival::ClothingExposure {
        self.survival_clothing
    }

    pub(crate) fn combat_equipment(&self) -> CombatEquipment {
        let mut armor = [CombatArmor {
            flexibility: 1.0,
            range_of_motion: 1.0,
            ..CombatArmor::default()
        }; 7];
        for part in BodyPart::FULL_BODY.iter() {
            let item = self.armor_for(part);
            armor[body_part_index(part)] = CombatArmor {
                inventory_item_id: self.armor_inventory_item_ids[body_part_index(part)],
                material: self.armor_materials[body_part_index(part)],
                resistance: item.resistance,
                padding: item.padding,
                flexibility: item.flexibility,
                range_of_motion: item.range_of_motion,
                coverage: item.coverage,
                coverage_geometry: self.armor_coverage_geometry[body_part_index(part)]
                    .unwrap_or_default(),
            };
        }
        CombatEquipment {
            weapon: self
                .weapon
                .as_ref()
                .map(|item| combat_weapon(item, self.melee_weapon_geometry.filter(|_| item.melee))),
            melee_weapon: self
                .melee_weapon
                .as_ref()
                .map(|item| combat_weapon(item, self.melee_weapon_geometry)),
            ranged_weapon: self
                .ranged_weapon
                .as_ref()
                .map(|item| combat_weapon(item, None)),
            melee_weapon_id: self.melee_weapon_inventory_id,
            ranged_weapon_id: self.ranged_weapon_inventory_id,
            ranged_projectile_kind: self.ranged_weapon.as_ref().map(|weapon| {
                if weapon.id.contains("arquebus") {
                    ProjectileKind::Ball
                } else {
                    ProjectileKind::Arrowhead
                }
            }),
            defense_item_id: self.shield_inventory_id.or(self.melee_weapon_inventory_id),
            ammunition: self.ammunition,
            holding_side: self.weapon_side.unwrap_or(BodySide::Right),
            melee_holding_side: self.melee_weapon_side.unwrap_or(BodySide::Right),
            ranged_holding_side: self.ranged_weapon_side.unwrap_or(BodySide::Right),
            shield_block_bonus: self.shield.as_ref().map_or(0.0, |item| item.block),
            shield_side: self.shield_side,
            armor,
            inventory_weight: self.inventory_weight,
        }
    }
}

fn hand_side(index: usize) -> BodySide {
    if index == 0 {
        BodySide::Left
    } else {
        BodySide::Right
    }
}

fn runtime_body_part(part: EquipmentBodyPart) -> BodyPart {
    use EquipmentBodyPart as E;
    match part {
        E::LeftArm => BodyPart::LeftArm,
        E::RightArm => BodyPart::RightArm,
        E::LeftLeg => BodyPart::LeftLeg,
        E::RightLeg => BodyPart::RightLeg,
        E::Chest => BodyPart::Chest,
        E::Stomach => BodyPart::Stomach,
        E::Head => BodyPart::Head,
    }
}

impl PlayerEquipment for StrategicEquipment {
    fn weapon_skill_distribution(&self) -> adventuresim_core::equipment::WeaponSkillDistribution {
        self.weapon
            .as_ref()
            .map_or_else(Default::default, |item| item.weapon_skills)
    }
    fn weapon_is_melee(&self) -> bool {
        self.weapon.as_ref().is_some_and(|item| item.melee)
    }
    fn weapon_is_ranged(&self) -> bool {
        self.weapon.as_ref().is_some_and(|item| item.ranged)
    }
    fn weapon_is_unarmed(&self) -> bool {
        self.weapon.is_none()
    }
    fn weapon_preferred_melee_style(&self) -> MeleeAttackStyle {
        self.weapon
            .as_ref()
            .map_or(MeleeAttackStyle::Swing, |item| item.preferred_melee_style)
    }
    fn weapon_weight(&self) -> f32 {
        self.weapon.as_ref().map_or(0.0, |item| item.weight)
    }
    fn weapon_precision(&self) -> f32 {
        self.weapon.as_ref().map_or(0.0, |item| {
            if item.melee
                && let Some(geometry) = self.melee_weapon_geometry
            {
                geometry.conditioned_precision(&item.id, item.precision)
            } else {
                item.precision
            }
        })
    }
    fn weapon_reach(&self) -> f32 {
        self.weapon.as_ref().map_or(0.0, |item| item.reach)
    }
    fn weapon_holding_side(&self) -> Option<BodySide> {
        self.weapon_side
    }
    fn weapon_balance(&self) -> f32 {
        self.weapon.as_ref().map_or(0.0, |item| item.balance)
    }
    fn weapon_moment_of_inertia(&self) -> f32 {
        self.weapon
            .as_ref()
            .map_or(0.0, |item| item.moment_of_inertia_kg_m2)
    }
    fn shield_block_bonus(&self) -> f32 {
        self.shield.as_ref().map_or(0.0, |item| item.block)
    }
    fn armor_resistance(&self, part: BodyPart) -> f32 {
        self.armor_for(part).resistance
    }
    fn armor_padding(&self, part: BodyPart) -> f32 {
        self.armor_for(part).padding
    }
    fn armor_flexibility(&self, part: BodyPart) -> f32 {
        self.armor_for(part).flexibility
    }
    fn armor_range_of_motion(&self, part: BodyPart) -> f32 {
        self.armor_for(part).range_of_motion
    }
    fn armor_coverage(&self, part: BodyPart) -> f32 {
        self.armor_for(part).coverage
    }
    fn inventory_weight(&self) -> f32 {
        self.inventory_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combat_weapon_preserves_authored_contact_geometry_and_materials() {
        let item = Item {
            id: "halberd".to_owned(),
            melee: true,
            ..Item::default()
        };

        let weapon = combat_weapon(&item, None);
        let definition = adventuresim_core::item_catalog::definition(&(&item.id).into()).unwrap();
        let equipment = definition.equipment.as_ref().unwrap();

        assert_eq!(weapon.grip_to_tip_m, equipment.physical.grip_to_tip_m);
        assert_eq!(weapon.total_length_m, equipment.physical.dimensions_m[1]);
        assert_eq!(
            weapon.striking_head_length_m,
            equipment.physical.dimensions_m[0].max(equipment.physical.dimensions_m[2])
        );
        assert!(weapon.distal_headed);
        assert_eq!(weapon.body_material, equipment.material);
        assert_eq!(weapon.striking_material, equipment.striking_material);
    }

    #[test]
    fn autoresolve_weapon_uses_per_instance_reach_mass_inertia_and_precision() {
        let item = Item {
            id: "halberd".to_owned(),
            melee: true,
            reach: 2.0,
            precision: adventuresim_core::item_catalog::weapon_precision(&("halberd").into())
                .unwrap(),
            moment_of_inertia_kg_m2: 4.0,
            ..Item::default()
        };
        let short = adventuresim_core::equipment::ParametricWeaponCombatGeometry::new(
            2.1, 1.9, 1.7, 0.25, 3.2, 0.52, 1.2,
        )
        .unwrap();
        let long = adventuresim_core::equipment::ParametricWeaponCombatGeometry::new(
            2.5, 2.3, 2.1, 0.25, 5.1, 0.49, 0.9,
        )
        .unwrap();
        let short = combat_weapon(&item, Some(short));
        let long = combat_weapon(&item, Some(long));

        assert!((short.precision - 1.2).abs() < 1e-6);
        assert!((long.precision - 0.9).abs() < 1e-6);
        assert!(long.melee_reach > short.melee_reach);
        assert!(long.weight > short.weight);
        assert!(long.moment_of_inertia_kg_m2 > short.moment_of_inertia_kg_m2);
        assert!(long.attack_interval_seconds > short.attack_interval_seconds);
        assert_eq!(long.striking_head_length_m, short.striking_head_length_m);
    }

    #[test]
    fn water_burden_comes_only_from_physical_containers() {
        let source = crate::production_source(include_str!("capability.rs"));
        assert!(source.contains("contained_water_ml"));
        assert!(!source.contains("carried_water_ml"));
    }
}

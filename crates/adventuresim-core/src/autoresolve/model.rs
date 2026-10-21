//! Body, condition, skills and equipment snapshots used by pure battle resolution.

use super::*;
use adventuresim_world_schema::{BestiaryCategory, BestiaryHours};
use serde::Serialize;
use fabelgeist_geometry::SignedLength;

#[derive(Clone, Debug, Serialize)]
pub struct CombatBody {
    pub health: [f32; 7],
    pub weight_kg: f32,
    pub primary_side: BodySide,
}

#[derive(Clone, Debug, Default)]
pub struct CombatEssentials {
    pub calories_used_today: f32,
    pub focus_level: f32,
}

#[derive(Clone, Debug, Default)]
pub struct CombatSkills {
    pub polearm_hours: f32,
    pub axe_hours: f32,
    pub bludgeon_hours: f32,
    pub sword_hours: f32,
    pub knife_hours: f32,
    pub dodge_hours: f32,
    pub block_hours: f32,
    pub bow_hours: f32,
    pub crossbow_hours: f32,
    pub firearm_hours: f32,
    pub throw_hours: f32,
    pub will_hours: f32,
    pub insight_hours: f32,
    pub charm_hours: f32,
    pub command_hours: f32,
    pub deception_hours: f32,
    pub physiology_hours: f32,
    pub religion_hours: f32,
    pub stealth_hours: f32,
    pub balance_hours: f32,
    pub bestiary_hours: BestiaryHours,
    pub surgery_hours: f32,
    pub tailoring_hours: f32,
    pub smithing_hours: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CombatArmor {
    pub inventory_item_id: Option<u64>,
    pub material: Option<crate::item_catalog_schema::EquipmentMaterial>,
    pub resistance: f32,
    pub padding: f32,
    pub flexibility: f32,
    pub range_of_motion: f32,
    pub coverage: f32,
    pub coverage_geometry: AuthoredArmorCoverage,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CombatWeapon {
    pub skills: crate::equipment::WeaponSkillDistribution,
    pub melee: bool,
    pub ranged: bool,
    pub preferred_melee_style: crate::combat_style::MeleeAttackStyle,
    pub weight: f32,
    pub moment_of_inertia_kg_m2: f32,
    pub precision: f32,
    pub melee_reach: SignedLength,
    pub grip_to_tip_m: SignedLength,
    pub total_length_m: SignedLength,
    pub striking_head_length_m: SignedLength,
    pub distal_headed: bool,
    pub body_material: Option<crate::item_catalog_schema::EquipmentMaterial>,
    pub striking_material: Option<crate::item_catalog_schema::EquipmentMaterial>,
    pub ranged_range: SignedLength,
    pub attack_interval_seconds: crate::elapsed::AnimationElapsed,
    pub balance: f32,
    pub ranged_force_joules: f32,
}

#[derive(Clone, Debug)]
pub struct CombatEquipment {
    pub weapon: Option<CombatWeapon>,
    pub melee_weapon: Option<CombatWeapon>,
    pub ranged_weapon: Option<CombatWeapon>,
    pub melee_weapon_id: Option<u64>,
    pub ranged_weapon_id: Option<u64>,
    pub ranged_projectile_kind: Option<ProjectileKind>,
    pub defense_item_id: Option<u64>,
    pub ammunition: u32,
    pub holding_side: BodySide,
    pub melee_holding_side: BodySide,
    pub ranged_holding_side: BodySide,
    pub shield_block_bonus: f32,
    pub shield_side: Option<BodySide>,
    pub armor: [CombatArmor; 7],
    pub inventory_weight: f32,
}

impl Default for CombatBody {
    fn default() -> Self {
        Self {
            health: [1.0; 7],
            weight_kg: 70.0,
            primary_side: BodySide::Right,
        }
    }
}

impl CombatBody {
    pub fn health(&self, part: BodyPart) -> f32 {
        self.health[body_part_index(part)]
    }

    pub fn apply_damage(&mut self, part: BodyPart, damage: f32) -> f32 {
        let health = &mut self.health[body_part_index(part)];
        apply_clamped_limb_damage(health, damage)
    }

    pub fn total_damage(&self) -> f32 {
        self.health
            .iter()
            .map(|health| (1.0 - health).max(0.0))
            .sum()
    }
}

impl PlayerBody for CombatBody {
    fn body_part_health(&self, part: BodyPart) -> f32 {
        self.health(part)
    }

    fn body_weight(&self) -> f32 {
        self.weight_kg
    }

    fn primary_side(&self) -> BodySide {
        self.primary_side
    }
}

impl PlayerEssentials for CombatEssentials {
    // Fatigue and burden are already part of live combat incapacitation.
    fn physical_skill_condition_by_parts(
        &self,
        _attr: &impl PlayerAttributes,
        _body: &impl PlayerBody,
        _equipment: &impl PlayerEquipment,
    ) -> f32 {
        1.0
    }

    fn calories_used_today(&self) -> f32 {
        self.calories_used_today
    }

    fn focus_level(&self) -> f32 {
        self.focus_level
    }
}

impl PlayerSkills for CombatSkills {
    fn skill_hours_trained(&self, skill: Skill) -> f32 {
        match skill {
            Skill::Polearm => self.polearm_hours,
            Skill::Axe => self.axe_hours,
            Skill::Bludgeon => self.bludgeon_hours,
            Skill::Sword => self.sword_hours,
            Skill::Knife => self.knife_hours,
            Skill::Dodge => self.dodge_hours,
            Skill::Block => self.block_hours,
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
            Skill::Cooking => 0.0,
            Skill::Herbalism => 0.0,
            Skill::Religion => self.religion_hours,
            Skill::Bestiary => self.bestiary_hours.aggregate_effective(),
            Skill::Stealth => self.stealth_hours,
            Skill::Balance => self.balance_hours,
            Skill::TerrainPlains
            | Skill::TerrainForest
            | Skill::TerrainHills
            | Skill::TerrainWetlands
            | Skill::TerrainUrban
            | Skill::TerrainSnow => 0.0,
            Skill::Surgery => self.surgery_hours,
            Skill::Tailoring => self.tailoring_hours,
            Skill::Smithing => self.smithing_hours,
        }
    }

    fn bestiary_hours_for(&self, category: BestiaryCategory) -> f32 {
        self.bestiary_hours.effective(category)
    }
}

impl CombatArmor {
    /// Full-body anatomical protection using the ordinary armor calculation.
    pub fn innate(resistance: f32, padding: f32) -> Self {
        Self {
            inventory_item_id: None,
            material: None,
            resistance,
            padding,
            flexibility: 0.5,
            range_of_motion: 1.0,
            coverage: 1.0,
            coverage_geometry: AuthoredArmorCoverage::from_span(ArmorCoverageSpan::centered(1.0)),
        }
    }
}

impl Default for CombatEquipment {
    fn default() -> Self {
        Self {
            weapon: None,
            melee_weapon: None,
            ranged_weapon: None,
            melee_weapon_id: None,
            ranged_weapon_id: None,
            ranged_projectile_kind: None,
            defense_item_id: None,
            ammunition: 0,
            holding_side: BodySide::Right,
            melee_holding_side: BodySide::Right,
            ranged_holding_side: BodySide::Right,
            shield_block_bonus: 0.0,
            shield_side: None,
            armor: [CombatArmor {
                inventory_item_id: None,
                material: None,
                range_of_motion: 1.0,
                flexibility: 1.0,
                ..CombatArmor::default()
            }; 7],
            inventory_weight: 0.0,
        }
    }
}

impl CombatEquipment {
    pub(super) fn for_melee(&self) -> Self {
        let mut equipment = self.clone();
        equipment.weapon = self.melee_weapon;
        equipment.holding_side = self.melee_holding_side;
        equipment
    }

    pub(super) fn for_ranged(&self) -> Self {
        let mut equipment = self.clone();
        equipment.weapon = self.ranged_weapon;
        equipment.holding_side = self.ranged_holding_side;
        equipment
    }
}

impl PlayerEquipment for CombatEquipment {
    fn weapon_skill_distribution(&self) -> crate::equipment::WeaponSkillDistribution {
        self.weapon.map_or(
            crate::equipment::WeaponSkillDistribution::UNARMED,
            |weapon| weapon.skills,
        )
    }
    fn weapon_is_melee(&self) -> bool {
        self.weapon.is_none_or(|weapon| weapon.melee)
    }
    fn weapon_is_ranged(&self) -> bool {
        self.weapon.is_some_and(|weapon| weapon.ranged)
    }
    fn weapon_is_unarmed(&self) -> bool {
        self.weapon.is_none()
    }
    fn weapon_preferred_melee_style(&self) -> crate::combat_style::MeleeAttackStyle {
        self.weapon
            .map_or(crate::combat_style::MeleeAttackStyle::Swing, |weapon| {
                weapon.preferred_melee_style
            })
    }
    fn weapon_weight(&self) -> f32 {
        self.weapon.map_or(0.0, |weapon| weapon.weight)
    }
    fn weapon_precision(&self) -> f32 {
        self.weapon.map_or(0.0, |weapon| weapon.precision)
    }
    fn weapon_reach(&self) -> SignedLength {
        self.weapon.map_or(SignedLength::ZERO, |weapon| weapon.melee_reach)
    }
    fn weapon_grip_to_tip(&self) -> SignedLength {
        self.weapon.map_or(SignedLength::ZERO, |weapon| weapon.grip_to_tip_m)
    }
    fn weapon_total_length(&self) -> SignedLength {
        self.weapon.map_or(SignedLength::ZERO, |weapon| weapon.total_length_m)
    }
    fn weapon_striking_head_length(&self) -> SignedLength {
        self.weapon
            .map_or(SignedLength::ZERO, |weapon| weapon.striking_head_length_m)
    }
    fn weapon_body_material(&self) -> Option<crate::item_catalog_schema::EquipmentMaterial> {
        self.weapon.and_then(|weapon| weapon.body_material)
    }
    fn weapon_striking_material(&self) -> Option<crate::item_catalog_schema::EquipmentMaterial> {
        self.weapon.and_then(|weapon| weapon.striking_material)
    }
    fn weapon_holding_side(&self) -> Option<BodySide> {
        self.weapon.map(|_| self.holding_side)
    }
    fn weapon_balance(&self) -> f32 {
        self.weapon.map_or(0.0, |weapon| weapon.balance)
    }
    fn weapon_moment_of_inertia(&self) -> f32 {
        self.weapon
            .map_or(0.0, |weapon| weapon.moment_of_inertia_kg_m2)
    }
    fn weapon_ranged_force_joules(&self) -> f32 {
        self.weapon.map_or(0.0, |weapon| weapon.ranged_force_joules)
    }
    fn shield_block_bonus(&self) -> f32 {
        self.shield_block_bonus
    }
    fn shield_holding_side(&self) -> Option<BodySide> {
        self.shield_side
    }
    fn armor_resistance(&self, part: BodyPart) -> f32 {
        self.armor[body_part_index(part)].resistance
    }
    fn armor_padding(&self, part: BodyPart) -> f32 {
        self.armor[body_part_index(part)].padding
    }
    fn armor_flexibility(&self, part: BodyPart) -> f32 {
        self.armor[body_part_index(part)].flexibility
    }
    fn armor_range_of_motion(&self, part: BodyPart) -> f32 {
        self.armor[body_part_index(part)].range_of_motion
    }
    fn armor_coverage(&self, part: BodyPart) -> f32 {
        self.armor[body_part_index(part)].coverage
    }
    fn armor_surface(&self, part: BodyPart, sample: f32) -> Option<crate::equipment::ArmorSurface> {
        let armor = self.armor[body_part_index(part)];
        armor
            .coverage_geometry
            .contains(sample)
            .then_some(crate::equipment::ArmorSurface {
                inventory_item_id: armor.inventory_item_id,
                material: armor.material,
                resistance: armor.resistance,
                padding: armor.padding,
                flexibility: armor.flexibility,
            })
    }
    fn inventory_weight(&self) -> f32 {
        self.inventory_weight
    }
}

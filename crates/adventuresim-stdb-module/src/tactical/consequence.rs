//! Durable consequences submitted at the end of a tactical mission.

use spacetimedb::SpacetimeType;

#[derive(Clone, Debug, PartialEq, SpacetimeType)]
pub struct TacticalHitInjury {
    pub body_part: TacticalReceiptBodyPart,
    pub cut_damage: f32,
    pub blunt_damage: f32,
    /// Largest individual blunt hit represented by this per-limb aggregate.
    pub max_single_hit_blunt_damage: f32,
}

#[derive(Clone, Debug, PartialEq, SpacetimeType)]
pub struct TacticalCharacterConsequence {
    pub character_id: u64,
    pub injuries: Vec<TacticalHitInjury>,
    /// Incremental fraction of maximum blood lost during this session.
    pub blood_loss_fraction: f32,
    pub ammunition_used: u32,
}

#[derive(Clone, Debug, PartialEq, SpacetimeType)]
pub struct TacticalEquipmentContact {
    pub character_id: u64,
    pub inventory_item_id: u64,
    pub contact_stress: f32,
    pub role: TacticalEquipmentContactRole,
}

#[derive(Clone, Debug, Default, PartialEq, SpacetimeType)]
pub struct TacticalConsequenceReceipt {
    pub party: Vec<TacticalCharacterConsequence>,
    pub equipment_contacts: Vec<TacticalEquipmentContact>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, SpacetimeType)]
pub enum TacticalMissionResolution {
    Failed,
    Defeated,
    DrivenOff,
    Captured,
    CaptureTargetKilled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, SpacetimeType)]
pub enum TacticalReceiptBodyPart {
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    Chest,
    Stomach,
    Head,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, SpacetimeType)]
pub enum TacticalEquipmentContactRole {
    AttackerWeapon,
    DefenderEquipment,
}

//! The supported initial-loadout anchors shared by player and enemy generation.

use serde::{Deserialize, Serialize};

/// Initial loadouts support a restricted set of body anchors. The full
/// EquipmentLocation topology also includes pockets, belts, and attachments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadoutSlot {
    LeftHand,
    RightHand,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    LeftFoot,
    RightFoot,
    Head,
    Chest,
    Stomach,
}

impl From<LoadoutSlot> for crate::item_catalog_schema::EquipmentLocation {
    fn from(slot: LoadoutSlot) -> Self {
        match slot {
            LoadoutSlot::LeftHand => Self::LeftHand,
            LoadoutSlot::RightHand => Self::RightHand,
            LoadoutSlot::LeftArm => Self::LeftArm,
            LoadoutSlot::RightArm => Self::RightArm,
            LoadoutSlot::LeftLeg => Self::LeftLeg,
            LoadoutSlot::RightLeg => Self::RightLeg,
            LoadoutSlot::LeftFoot => Self::LeftFoot,
            LoadoutSlot::RightFoot => Self::RightFoot,
            LoadoutSlot::Head => Self::Head,
            LoadoutSlot::Chest => Self::Chest,
            LoadoutSlot::Stomach => Self::Stomach,
        }
    }
}

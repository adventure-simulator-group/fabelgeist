//! Map the shared hand identity to its projected native equipment slot.

use adventuresim_core::equipment_placement::EquipmentHand;

use super::EquipSlot;

impl From<EquipmentHand> for EquipSlot {
    fn from(hand: EquipmentHand) -> Self {
        match hand {
            EquipmentHand::Left => Self::HoldingLeft,
            EquipmentHand::Right => Self::HoldingRight,
        }
    }
}

//! Admit SDK equipment selections into the shared canonical placement types.

use adventuresim_core::equipment_placement as core;

use crate::{EquipmentHand, EquippedPlacement};

impl From<EquipmentHand> for core::EquipmentHand {
    fn from(hand: EquipmentHand) -> Self {
        match hand {
            EquipmentHand::Left => Self::Left,
            EquipmentHand::Right => Self::Right,
        }
    }
}

impl From<core::EquipmentHand> for EquipmentHand {
    fn from(hand: core::EquipmentHand) -> Self {
        match hand {
            core::EquipmentHand::Left => Self::Left,
            core::EquipmentHand::Right => Self::Right,
        }
    }
}

impl TryFrom<EquippedPlacement> for core::EquippedPlacement {
    type Error = core::AuthoredPlacementIdentityError;

    fn try_from(placement: EquippedPlacement) -> Result<Self, Self::Error> {
        Ok(match placement {
            EquippedPlacement::Authored(id) => Self::Authored(id.value.try_into()?),
            EquippedPlacement::Held(hand) => Self::Held(hand.into()),
        })
    }
}

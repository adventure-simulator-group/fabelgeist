//! Authored loadout for a validated starting roster slot.
use super::{CandidateSlot, LoadoutSlot};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Defense {
    Block,
    Dodge,
}
pub(super) struct StartingLoadout {
    pub background: &'static str,
    pub weapon: &'static str,
    pub weapon_slot: LoadoutSlot,
    pub armor: &'static str,
    pub defense: Defense,
    pub currency_base: u32,
}
impl StartingLoadout {
    pub(super) fn for_slot(slot: CandidateSlot) -> Self {
        let (background, weapon, weapon_slot, armor, defense, currency_base) = match slot.get() {
            0 => (
                "Militia runner",
                "katzbalger",
                LoadoutSlot::RightHand,
                "arming_doublet",
                Defense::Block,
                90,
            ),
            1 => (
                "Woodland hunter",
                "longbow",
                LoadoutSlot::RightHand,
                "quilted_sleeve",
                Defense::Dodge,
                65,
            ),
            2 => (
                "Caravan guard",
                "hunting_spear",
                LoadoutSlot::RightHand,
                "padded_chausses",
                Defense::Dodge,
                125,
            ),
            3 => (
                "Town watch apprentice",
                "light_crossbow",
                LoadoutSlot::RightHand,
                "arming_cap",
                Defense::Block,
                155,
            ),
            _ => (
                "Camp follower turned scout",
                "bauernwehr",
                LoadoutSlot::RightHand,
                "padded_skirt",
                Defense::Dodge,
                105,
            ),
        };
        Self {
            background,
            weapon,
            weapon_slot,
            armor,
            defense,
            currency_base,
        }
    }
}

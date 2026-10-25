//! Physical weapon cadence observed by the tactical inventory view.

use super::*;

impl InventoryView<'_, '_, '_> {
    pub(super) fn melee_timing_for(
        &self,
        style: MeleeAttackStyle,
    ) -> ElapsedResult<adventuresim_core::equipment::MeleeAttackTiming> {
        let weapon = self.equipped_weapon().and_then(|item| item.weapon);
        adventuresim_core::equipment::melee_attack_timing(
            style,
            weapon.map_or(0.0, |weapon| weapon.moment_of_inertia_kg_m2),
            weapon.is_none(),
        )
    }
}

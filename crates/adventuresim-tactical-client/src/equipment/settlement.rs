//! Whether character equipment has finished generating for the loading screen.
use crate::equipment::{
    ProceduralEquipmentResolved, runtime_equipment::RuntimeEquipmentPresentation,
};
use adventuresim_tactical_core::prelude::{CharacterId, ItemOf};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

type GeneratedEquipmentState = (
    &'static RuntimeEquipmentPresentation,
    Has<ProceduralEquipmentResolved>,
);

#[derive(SystemParam)]
pub(crate) struct EquipmentSettlement<'w, 's> {
    generated: Query<'w, 's, GeneratedEquipmentState>,
    owners: Query<'w, 's, &'static ItemOf>,
    characters: Query<'w, 's, (), With<CharacterId>>,
}

impl EquipmentSettlement<'_, '_> {
    /// Whether generation has finished, successfully or not, for everything
    /// characters carry. Play goes on without an item that cannot be generated,
    /// and attaching the result is up to the rig, which may hide it on purpose.
    pub(crate) fn is_settled(&self) -> bool {
        self.generated.iter().all(|(presentation, resolved)| {
            let carried = self
                .owners
                .get(presentation.item)
                .is_ok_and(|owner| self.characters.contains(owner.0));
            resolved || !carried
        })
    }
}

//! Readiness covers generated geometry and wearer bindings, not just the body rig.
use crate::equipment::{
    ItemPlaceholder, ProceduralEquipmentFailed, ProceduralEquipmentPart,
    ProceduralEquipmentResolved, runtime_equipment::RuntimeEquipmentPresentation,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

type EquipmentRootState = (
    &'static ItemPlaceholder,
    &'static Visibility,
    Has<RuntimeEquipmentPresentation>,
    Has<ProceduralEquipmentResolved>,
    Has<ProceduralEquipmentFailed>,
);

#[derive(SystemParam)]
pub(crate) struct EquipmentReadiness<'w, 's> {
    roots: Query<'w, 's, EquipmentRootState>,
    parts: Query<'w, 's, (&'static ProceduralEquipmentPart, &'static Visibility)>,
}

impl EquipmentReadiness<'_, '_> {
    pub(crate) fn pending_summary(&self, items: impl Iterator<Item = Entity>) -> String {
        let items: std::collections::HashSet<_> = items.collect();
        let roots = self
            .roots
            .iter()
            .filter(|(root, ..)| items.contains(&root.0))
            .collect::<Vec<_>>();
        let hidden = roots
            .iter()
            .filter(|(_, visibility, ..)| **visibility == Visibility::Hidden)
            .count();
        let unresolved = roots
            .iter()
            .filter(|(_, _, runtime, resolved, _)| *runtime && !*resolved)
            .count();
        let unbound = self
            .parts
            .iter()
            .filter(|(part, visibility)| {
                items.contains(&part.item) && **visibility == Visibility::Hidden
            })
            .count();
        format!(
            "{} items, {} roots, {hidden} hidden, {unresolved} awaiting geometry, {unbound} awaiting presentation",
            items.len(),
            roots.len()
        )
    }

    pub(crate) fn check(&self, items: impl Iterator<Item = Entity>) -> Result<bool, &'static str> {
        let items: std::collections::HashSet<_> = items.collect();
        let mut ready = 0;
        for (placeholder, visibility, runtime, resolved, failed) in &self.roots {
            if !items.contains(&placeholder.0) {
                continue;
            }
            if failed {
                return Err("Could not generate character equipment");
            }
            if *visibility != Visibility::Hidden && (!runtime || resolved) {
                ready += 1;
            }
        }
        Ok(ready == items.len()
            && self.parts.iter().all(|(part, visibility)| {
                !items.contains(&part.item) || *visibility != Visibility::Hidden
            }))
    }
}

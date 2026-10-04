//! Readiness covers generated geometry and wearer bindings, not just the body rig.
use crate::equipment::{
    ItemPlaceholder, ProceduralEquipmentFailed, ProceduralEquipmentPart,
    ProceduralEquipmentResolved, RuntimeEquipmentWarmup,
    runtime_equipment::RuntimeEquipmentPresentation,
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
    warmup: Res<'w, RuntimeEquipmentWarmup>,
    roots: Query<'w, 's, EquipmentRootState>,
    parts: Query<'w, 's, (&'static ProceduralEquipmentPart, &'static Visibility)>,
}

impl EquipmentReadiness<'_, '_> {
    pub(crate) fn pending_summary(&self, items: impl Iterator<Item = Entity>) -> String {
        if self.warmup.check() == Ok(false) {
            return if self.warmup.is_preparing() {
                "Preparing character equipment pipelines"
            } else {
                "Loading character equipment definitions"
            }
            .into();
        }
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
        if !self.warmup.check()? {
            return Ok(false);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn empty_city_waits_for_preparation_before_reporting_ready() {
        let mut world = World::new();
        world.init_resource::<RuntimeEquipmentWarmup>();
        let readiness = |world: &mut World| {
            world
                .run_system_once(|readiness: EquipmentReadiness| {
                    readiness.check(std::iter::empty())
                })
                .unwrap()
        };
        assert_eq!(readiness(&mut world), Ok(false));
        world.insert_resource(RuntimeEquipmentWarmup::ready_for_tests());
        assert_eq!(readiness(&mut world), Ok(true));
    }
}

//! The exact exterior owner decides which dynamic leaves survive into Facade.
use super::*;
use std::collections::BTreeSet;

pub(super) fn exact_facade(plan: &BuildingPlan) -> bool {
    plan.small_church.is_some()
        || plan.church.is_some()
        || matches!(
            plan.archetype,
            crate::BuildingArchetype::FachwerkCottage
                | crate::BuildingArchetype::HallHouse
                | crate::BuildingArchetype::TownHouse
                | crate::BuildingArchetype::FachwerkMerchantHouse
        )
}

impl BuildingPlan {
    /// Operable openings whose Facade host has an actual aperture.
    pub fn facade_dynamic_openings(&self) -> BTreeSet<crate::OpeningAssemblyId> {
        if !exact_facade(self) {
            return BTreeSet::new();
        }
        let walls = compilation::retained_facade_runs(self)
            .into_iter()
            .flat_map(|run| run.source_walls)
            .collect::<BTreeSet<_>>();
        let dynamic = crate::detail::dynamic_closure_solids(self);
        self.opening_assemblies
            .iter()
            .filter(|opening| {
                walls.contains(&opening.host_wall)
                    && opening.closure_solids.iter().any(|id| dynamic.contains(id))
            })
            .map(|opening| opening.id)
            .collect()
    }
}

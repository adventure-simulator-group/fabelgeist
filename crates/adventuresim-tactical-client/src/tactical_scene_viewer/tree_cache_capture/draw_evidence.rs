//! One selected evidence owner; never accept a missing alternative observer.
use super::*;
use draw_policy::DrawPolicy;

#[derive(Resource, Clone)]
pub(super) enum DrawEvidence {
    Content(material_draws::SpecimenDraws),
    Ordinary(ordinary_batches::OrdinaryBatches),
}

impl DrawEvidence {
    pub(super) fn install(app: &mut App, policy: DrawPolicy) -> Self {
        observation::install(app);
        match policy {
            DrawPolicy::ContentSingletons => {
                Self::Content(material_draws::SpecimenDraws::install(app))
            }
            DrawPolicy::OrdinaryGpu(control) => {
                Self::Ordinary(ordinary_batches::OrdinaryBatches::install(app, control))
            }
        }
    }

    pub(super) fn proof(&self, view: usize) -> Result<Option<serde_json::Value>> {
        match self {
            Self::Content(draws) => Ok(draws.proof(view)),
            Self::Ordinary(draws) => draws.proof(view),
        }
    }

    pub(super) fn invalidate(&self, view: usize) {
        match self {
            Self::Content(draws) => draws.invalidate(view),
            Self::Ordinary(draws) => draws.invalidate(view),
        }
    }

    pub(super) fn inventory_verified(&self) -> bool {
        match self {
            Self::Content(draws) => draws.inventory_verified(),
            Self::Ordinary(draws) => draws.inventory_verified(),
        }
    }

    pub(super) fn all_valid(&self, count: usize) -> bool {
        match self {
            Self::Content(draws) => draws.all_valid(count),
            Self::Ordinary(draws) => draws.all_valid(count),
        }
    }

    pub(super) fn prepare_tags(
        &self,
        world: &mut World,
        inventory: &inventory::CurrentInventory,
    ) -> Result<()> {
        match self {
            Self::Content(_) => Ok(()),
            Self::Ordinary(draws) => draws.prepare_tags(world, inventory),
        }
    }

    pub(super) fn check_capture(
        &self,
        view: usize,
        inventory: &inventory::CurrentInventory,
        tags: &Query<&bevy::mesh::MeshTag>,
    ) -> Result<()> {
        match self {
            Self::Content(draws) => draws.check_background(view, &inventory.background),
            Self::Ordinary(draws) => draws.check_capture(view, inventory, tags),
        }
    }
}

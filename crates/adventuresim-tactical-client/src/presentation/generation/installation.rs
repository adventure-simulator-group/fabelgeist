//! Install products from one retained presentation owner.
use super::*;

impl PreparedProducts {
    pub(super) fn generated_scene(
        &self,
        input: &TacticalSceneInput,
    ) -> PreparationResult<GeneratedTacticalScene> {
        let mut scene = self.scene_for_installation(input)?;
        for placement in &self.placements {
            if !scene
                .buildings
                .iter()
                .any(|b| b.placement.id == placement.id)
            {
                let venue = self
                    .venues
                    .iter()
                    .find(|v| v.recipe.program == placement.program)
                    .ok_or(PreparationError::NotPrepared {
                        product: ProductKind::Venue,
                    })?;
                let distant = input
                    .distant_buildings
                    .iter()
                    .find(|b| b.id == placement.id)
                    .ok_or(PreparationError::MissingDistantPlacement {
                        building: placement.id,
                    })?;
                if *placement != (*distant).into() {
                    return Err(PreparationError::PromotedPlacementMismatch {
                        building: placement.id,
                    });
                }
                scene
                    .buildings
                    .push(adventuresim_tactical_core::scene_input::GeneratedBuilding {
                        placement: placement.clone(),
                        plan: venue.recipe.plan.clone(),
                        collision: venue.recipe.collision.clone(),
                    });
            }
        }
        for building in &scene.buildings {
            let venue = self
                .venues
                .iter()
                .find(|v| v.recipe.program == building.placement.program)
                .ok_or(PreparationError::NotPrepared {
                    product: ProductKind::Venue,
                })?;
            scene
                .furniture
                .install_interior(building, venue.interior.clone())?;
        }
        Ok(scene)
    }
}

pub(in crate::presentation) fn take_venue_geometry(
    owner: GenerationOwner,
    program: &BuildingProgram,
) -> PreparationResult<Arc<VenueGeometry>> {
    active_products(owner)?
        .venues
        .iter_mut()
        .find(|v| v.recipe.program == *program)
        .and_then(|v| v.geometry.take())
        .ok_or(PreparationError::NotPrepared {
            product: ProductKind::VenueGeometry,
        })
}

pub(in crate::presentation) fn take_facade(
    owner: GenerationOwner,
    program: &BuildingProgram,
) -> PreparationResult<Arc<PreparedFacade>> {
    let mut products = active_products(owner)?;
    let index = products
        .facades
        .iter()
        .position(|facade| facade.program == *program)
        .ok_or(PreparationError::NotPrepared {
            product: ProductKind::Facade,
        })?;
    Ok(products.facades.swap_remove(index))
}

pub(in crate::presentation) fn release_unused_facades(
    owner: GenerationOwner,
) -> PreparationResult<()> {
    active_products(owner)?.facades.clear();
    Ok(())
}

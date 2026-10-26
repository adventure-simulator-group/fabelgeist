//! Install products from one retained presentation owner.
use super::*;

impl PreparedProducts {
    pub(super) fn generated_scene(
        &self,
        input: &TacticalSceneInput,
    ) -> PreparationResult<PreparedStrategicScene> {
        let digest = input.digest()?;
        self.scene
            .as_ref()
            .filter(|scene| scene.digest == digest)
            .map(|scene| (**scene).clone())
            .ok_or(PreparationError::NotPrepared {
                product: ProductKind::Scene,
            })
    }
}

pub(in crate::presentation) fn take_venue_geometry(
    owner: PresentationOwner,
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
    owner: PresentationOwner,
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
    owner: PresentationOwner,
) -> PreparationResult<()> {
    active_products(owner)?.facades.clear();
    Ok(())
}

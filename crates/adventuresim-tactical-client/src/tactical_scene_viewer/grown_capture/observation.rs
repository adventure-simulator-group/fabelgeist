//! Whole inventory after actual visibility and terrain material preparation.

use super::super::capture_state::{CapturePhase, SceneCaptureState};
use super::*;
use crate::presentation::TacticalGameplayCamera;
use bevy::{camera::visibility::VisibilitySystems, ecs::system::SystemState};

#[derive(Resource, Default)]
pub(super) struct VisibilityMark(pub(super) Option<usize>);

pub(super) fn install(app: &mut App) {
    app.init_resource::<VisibilityMark>()
        .init_resource::<ordinary_batches::inventory::CurrentInventory>()
        .add_systems(
            PostUpdate,
            observe
                .after(VisibilitySystems::CheckVisibility)
                .after(crate::presentation::interior_lighting::InteriorLightingSystems::Prepare),
        );
}

pub(super) fn observe(world: &mut World) -> bevy::prelude::Result {
    world.resource_mut::<VisibilityMark>().0 = None;
    let state = world.resource::<SceneCaptureState>();
    if state.phase == CapturePhase::Configure || state.view >= state.views.len() {
        return Ok(());
    }
    let view = state.view;
    let CapturePose::Grown(expected) = state.views[view].pose else {
        return Err("grown pose required".into());
    };
    // Update can add real terrain after startup; instrument before extraction.
    background::instrument(world);
    let (camera, transform, projection) = world
        .query_filtered::<(&Camera, &GlobalTransform, &Projection), With<TacticalGameplayCamera>>()
        .single(world)?;
    if !expected.matches(
        camera,
        transform,
        projection,
        world.resource::<cameras::PerspectiveDepth>(),
    ) {
        world.resource::<Evidence>().camera_drift(view)?;
        return Ok(());
    }
    let checked = {
        let mut inspection = SystemState::<inventory::Inspector>::new(world);
        inspection
            .get(world)
            .map_err(anyhow::Error::from)?
            .check(world.resource::<SceneCaptureState>())
    };
    match checked {
        Ok(mut inventory) => {
            if let Some(selection) = &inventory.selection {
                world
                    .resource_mut::<generated::selection::SelectionHistory>()
                    .seal(selection.clone())?;
            }
            let tagging = world
                .resource::<ordinary_batches::OrdinaryBatches>()
                .clone()
                .prepare_tags(world, &inventory);
            if let Err(error) = tagging {
                world
                    .resource::<Evidence>()
                    .input_fault(view, format!("{error:#}"))?;
                return Err(error.into());
            }

            if matches!(world.resource::<Subject>(), Subject::Generated { .. })
                && !generated::demand::hold(world, view, &mut inventory)?
            {
                return Ok(());
            }
            world.insert_resource(inventory);
            world.resource_mut::<VisibilityMark>().0 = Some(view);
        }
        Err(error) => {
            world
                .resource::<Evidence>()
                .input_fault(view, format!("{error:#}"))?;
            return Err(error.into());
        }
    }
    Ok(())
}

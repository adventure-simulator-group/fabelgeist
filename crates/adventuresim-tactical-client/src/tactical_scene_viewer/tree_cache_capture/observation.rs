//! Shared structural observation after actual visibility and material conversion.
use super::super::capture_state::{CapturePhase, SceneCaptureState};
use super::*;
use crate::presentation::TacticalGameplayCamera;
use bevy::{camera::visibility::VisibilitySystems, ecs::system::SystemState};

#[derive(Resource, Default)]
pub(super) struct VisibilityMark(pub(super) Option<usize>);

pub(super) fn install(app: &mut App) {
    app.init_resource::<VisibilityMark>()
        .init_resource::<draw_policy::RootVisibility>()
        .init_resource::<inventory::CurrentInventory>()
        .add_systems(
            PostUpdate,
            observe
                .after(VisibilitySystems::CheckVisibility)
                .after(crate::presentation::interior_lighting::InteriorLightingSystems::Prepare),
        );
}

fn observe(world: &mut World) -> bevy::prelude::Result {
    world.resource_mut::<VisibilityMark>().0 = None;
    let state = world.resource::<SceneCaptureState>();
    if state.phase == CapturePhase::Configure || state.view >= state.views.len() {
        return Ok(());
    }
    let view = state.view;
    let CapturePose::Cache(expected) = state.views[view].pose else {
        return Ok(());
    };
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
        world
            .resource::<draw_evidence::DrawEvidence>()
            .invalidate(view);
        return Ok(());
    }
    if world
        .resource::<Review>()
        .draw_policy
        .singleton_instrumentation()
    {
        background::instrument(world);
        inventory::OwnedTrees::instrument_children(world)?;
    }
    let policy = world.resource::<Review>().draw_policy;
    if *world.resource::<draw_policy::RootVisibility>()
        != draw_policy::RootVisibility::for_view(policy, view)
    {
        return Err("root visibility control ordering drift".into());
    }
    let checked = {
        let mut inspection = SystemState::<inventory::Inspector>::new(world);
        inspection.get(world).map_err(anyhow::Error::from)?.check()
    };
    match checked {
        Ok(inventory) => {
            let draws = world.resource::<draw_evidence::DrawEvidence>().clone();
            draws.prepare_tags(world, &inventory)?;
            world.insert_resource(inventory);
            world.resource_mut::<VisibilityMark>().0 = Some(view);
        }
        Err(error) => {
            world
                .resource::<Evidence>()
                .input_fault(view, format!("{error:#}"))?;
            world
                .resource::<draw_evidence::DrawEvidence>()
                .invalidate(view);
            return Err(error.into());
        }
    }
    Ok(())
}

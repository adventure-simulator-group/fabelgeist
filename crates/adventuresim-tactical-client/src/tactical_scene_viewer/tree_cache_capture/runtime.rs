//! One actual production source or prepared cache per held capture process.
use super::super::{
    capture_state::{CapturePhase, SceneCaptureState},
    setup_scene,
};
use super::*;
use crate::presentation::{InstalledTreeSpecimen, TacticalGameplayCamera, TreeLodRenderOverride};

#[derive(Resource)]
pub(in super::super) struct ReadyPose(usize);

pub(in super::super) fn install(app: &mut App, review: Review) {
    let draws = material_draws::SpecimenDraws::install(app);
    let evidence = Evidence::new(
        review.metadata.clone(),
        review.role,
        review.representation,
        draws,
    );
    app.insert_resource(review)
        .insert_resource(evidence)
        .add_systems(
            PostStartup,
            setup
                .after(setup_scene)
                .after(super::super::freeze_capture_clock),
        )
        .add_systems(PreUpdate, advance)
        .add_systems(
            PostUpdate,
            guard_camera.after(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
        )
        .add_systems(
            PostUpdate,
            isolate.before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
        );
}

fn setup(world: &mut World) -> bevy::prelude::Result {
    setup_inner(world).map_err(Into::into)
}

fn setup_inner(world: &mut World) -> Result<()> {
    let projection = world
        .query_filtered::<&Projection, With<TacticalGameplayCamera>>()
        .single(world)?;
    let depth = cameras::PerspectiveDepth::from_projection(projection)?;
    world.insert_resource(depth);
    let first = *world
        .resource::<SceneCaptureState>()
        .views
        .first()
        .context("first cache view")?;
    ensure!(
        first.render_lod_override == Some(world.resource::<Review>().representation.lod())
            && first.leaf_lod_override == world.resource::<Review>().representation.leaf()
            && first.projected_scale.is_none(),
        "cache review requires fixed Full before first Update"
    );
    world.insert_resource(TreeLodRenderOverride {
        lod: first.render_lod_override,
        leaf: first.leaf_lod_override,
        projected_scale: first.projected_scale,
    });
    let role = world.resource::<Review>().role;
    let root = world.resource::<Review>().root;
    let owned = match role {
        Role::Reference => {
            let source = world.resource::<Review>().source;
            inventory::OwnedSpecimen::from_cache(
                InstalledTreeSpecimen::from_generated(world, source, root)?,
                world,
            )?
        }
        Role::Candidate => {
            let cache = world
                .resource_mut::<Review>()
                .cache
                .take()
                .context("cache consumed once")?;
            inventory::OwnedSpecimen::from_cache(cache.spawn(world, root)?, world)?
        }
    };
    owned.retain_cpu_data(world)?;
    world.insert_resource(owned);
    {
        let mut state = world.resource_mut::<SceneCaptureState>();
        ensure!(
            state.profile == PROFILE,
            "isolated setup requires the dedicated profile"
        );
        state.expected_trees = 0;
        state.expected_rocks = 0;
        state.expects_grass = false;
        state.tree_focus = None;
        state.tree_focus_entity = None;
        state.tree_review_entities.clear();
        state.tree_review_leaf_entities.clear();
    }
    isolate(world);
    let evidence = world.resource::<Evidence>().clone();
    world
        .resource_mut::<SceneCaptureState>()
        .tree_cache_evidence = Some(evidence);
    let adapter = world
        .get_resource::<bevy::render::renderer::RenderAdapterInfo>()
        .context("actual native renderer adapter information")?;
    let metadata = serde_json::json!({
        "adapter": {"name": adapter.name, "backend": format!("{:?}", adapter.backend),
            "device_type": format!("{:?}", adapter.device_type), "driver": adapter.driver,
            "driver_info": adapter.driver_info},
        "projection_depth": world.resource::<cameras::PerspectiveDepth>(),
        "retained_cache": world.resource::<inventory::OwnedSpecimen>().retained_report(),
        "actual_virtual_seconds": world.resource::<Time<Virtual>>().elapsed_secs_f64(),
        "scope": "actual native renderer, isolated held production-cache content comparison"});
    fs::write(
        world
            .resource::<SceneCaptureState>()
            .output
            .join("cache-review-runtime.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    Ok(())
}

fn advance(world: &mut World) -> bevy::prelude::Result {
    let Some(state) = world.get_resource::<SceneCaptureState>() else {
        return Ok(());
    };
    if state.view >= state.views.len() || state.phase == CapturePhase::Configure {
        return Ok(());
    }
    let view = state.view;
    let CapturePose::Cache(expected) = state.views[view].pose else {
        return Err("cache pose required".into());
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
        return Err("held review camera/projection drift".into());
    }
    let actual_camera = expected.actual(transform, projection)?;
    if let Some(proof) = world
        .resource::<material_draws::SpecimenDraws>()
        .proof(view)
    {
        world.resource::<Evidence>().record(
            view,
            serde_json::json!({"camera": actual_camera, "draws": proof, "whole_inventory": world.resource::<inventory::CurrentInventory>().report()}),
        )?;
        world.insert_resource(ReadyPose(view));
    }
    Ok(())
}

pub(in super::super) fn ready(
    state: Option<Res<SceneCaptureState>>,
    review: Option<Res<Review>>,
    ready: Option<Res<ReadyPose>>,
    evidence: Option<Res<Evidence>>,
) -> bool {
    let Some(state) = state else {
        return true;
    };
    if review.is_none() {
        return state.profile != PROFILE;
    }
    if state.phase == CapturePhase::Configure {
        return true;
    }
    ready.is_some_and(|ready| ready.0 == state.view)
        && evidence.is_some_and(|evidence| evidence.camera_intact())
}

fn isolate(world: &mut World) {
    use crate::presentation::{
        GroundScatterLayer, UnderstoryReviewSpecimen, VistaTreePresentation,
    };
    use adventuresim_tactical_core::prelude::SceneObstacle;
    let entities: Vec<_> = world
        .query_filtered::<Entity, Or<(
            With<SceneObstacle>,
            With<GroundScatterLayer>,
            With<VistaTreePresentation>,
            With<UnderstoryReviewSpecimen>,
            With<super::super::TreeReviewSpecimen>,
            With<super::super::TreeReviewBackdrop>,
        )>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
}

fn guard_camera(
    state: Option<Res<SceneCaptureState>>,
    evidence: Res<Evidence>,
    depth: Res<cameras::PerspectiveDepth>,
    cameras: Query<
        (&bevy::prelude::Camera, &GlobalTransform, &Projection),
        With<TacticalGameplayCamera>,
    >,
) -> bevy::prelude::Result {
    let Some(state) = state else {
        return Ok(());
    };
    if state.phase == CapturePhase::Configure {
        return Ok(());
    }
    let Some(view) = state.views.get(state.view) else {
        return Ok(());
    };
    let CapturePose::Cache(expected) = view.pose else {
        return Ok(());
    };
    if !cameras
        .single()
        .is_ok_and(|(camera, transform, projection)| {
            expected.matches(camera, transform, projection, &depth)
        })
    {
        evidence
            .camera_drift(state.view)
            .map_err(bevy::ecs::error::BevyError::from)?;
    }
    Ok(())
}

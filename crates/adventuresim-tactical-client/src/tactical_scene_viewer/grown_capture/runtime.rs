//! Install one shared source closure and hold three actual grounded instances.

use super::super::{
    capture_state::{CapturePhase, SceneCaptureState},
    setup_scene,
};
use super::*;
use crate::presentation::TacticalGameplayCamera;

#[derive(Resource)]
pub(in super::super) struct ReadyPose(usize);

pub(in super::super) fn install(app: &mut App, review: Review) {
    app.add_plugins(GrownMaterialPlugin);
    observation::install(app);
    app.init_resource::<controls::RootVisibility>();
    ordinary_batches::OrdinaryBatches::install(app, review.control);
    app.insert_resource(review)
        .add_systems(
            PostStartup,
            setup
                .after(setup_scene)
                .after(super::super::freeze_capture_clock),
        )
        .add_systems(PreUpdate, advance)
        .add_systems(
            PostUpdate,
            isolate.before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
        );
}

pub(in super::super) fn ready(
    state: Option<Res<SceneCaptureState>>,
    review: Option<Res<Review>>,
    ready: Option<Res<ReadyPose>>,
    evidence: Option<Res<Evidence>>,
) -> bool {
    let Some(state) = state else { return true };
    if review.is_none() {
        return state.profile != PROFILE;
    }
    if state.phase == CapturePhase::Configure {
        return true;
    }
    ready.is_some_and(|ready| ready.0 == state.view)
        && evidence.is_some_and(|evidence| evidence.camera_intact())
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
    let mut review = world
        .remove_resource::<Review>()
        .context("grown review missing at setup")?;
    let geometry = review
        .geometry
        .take()
        .context("prepared grown geometry missing")?;
    let digest = world.resource::<SceneCaptureState>().digest.clone();
    review.control.apply_original_batch(world)?;
    let original_batch =
        super::super::obstacle_setup::CapturedObstacleBatch::check(world, &digest)?;
    review.metadata["pre_isolation_generated_obstacle_batch"] =
        serde_json::to_value(original_batch)?;
    let resolved = cameras::resolve(world, &geometry, &review.placements)?;
    let cameras = resolved.cameras;
    let support = resolved.support;
    let views = view_selection::select(
        &cameras.views(),
        &review.requested,
        view_selection::ViewSequence::Independent,
    )
    .map_err(anyhow::Error::msg)?;
    {
        let mut state = world.resource_mut::<SceneCaptureState>();
        ensure!(
            state.phase == CapturePhase::Configure
                && state.view == 0
                && state.captures.is_empty()
                && state.views.is_empty(),
            "grown cameras must resolve once before capture"
        );
        state.requested_views = views
            .iter()
            .filter(|view| !view.warmup)
            .map(|view| view.slug.to_owned())
            .collect();
        state.views = views;
    }
    review.metadata["cameras"] = serde_json::to_value(&cameras)?;
    review.metadata["camera_support"] = support.report();
    let evidence = Evidence::new(
        review.metadata.clone(),
        review.role,
        review.source,
        review.level,
        world
            .resource::<ordinary_batches::OrdinaryBatches>()
            .clone(),
    );
    let placements = review.placements;
    let control = review.control;
    world.insert_resource(review);
    world.insert_resource(support);
    world.insert_resource(evidence);
    let owned = inventory::OwnedGrown::install(geometry, placements, control, world)?;
    world.insert_resource(owned);
    {
        let mut state = world.resource_mut::<SceneCaptureState>();
        ensure!(state.profile == PROFILE, "grown setup profile mismatch");
        state.expected_trees = 0;
        state.expected_rocks = 0;
        state.expects_grass = false;
        state.tree_focus = None;
        state.tree_focus_entity = None;
        state.tree_review_entities.clear();
        state.tree_review_leaf_entities.clear();
    }
    isolate(world);
    background::instrument(world);
    let evidence = world.resource::<Evidence>().clone();
    world.resource_mut::<SceneCaptureState>().grown_evidence = Some(evidence);
    let adapter = world
        .get_resource::<bevy::render::renderer::RenderAdapterInfo>()
        .context("grown renderer adapter")?;
    let metadata = serde_json::json!({"adapter":{"name":adapter.name,"backend":format!("{:?}",adapter.backend),"device_type":format!("{:?}",adapter.device_type),"driver":adapter.driver,"driver_info":adapter.driver_info},"projection_depth":world.resource::<cameras::PerspectiveDepth>(),"actual_virtual_seconds":world.resource::<Time<Virtual>>().elapsed_secs_f64(),"scope":"three grounded native grown instances sharing one asset set; ordinary main-pass GPU membership proof; no arrival or performance"});
    fs::write(
        world
            .resource::<SceneCaptureState>()
            .output
            .join("grown-runtime.json"),
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
    let CapturePose::Grown(expected) = state.views[view].pose else {
        return Err("grown held pose required".into());
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
        return Err("grown held camera drift".into());
    }
    let actual = expected.actual(camera, transform, projection)?;
    if let Some(proof) = world
        .resource::<ordinary_batches::OrdinaryBatches>()
        .proof(view)?
    {
        world.resource::<Evidence>().record(view,serde_json::json!({"camera":actual,"draws":proof,"whole_inventory":world.resource::<inventory::CurrentInventory>().report()}))?;
        world.insert_resource(ReadyPose(view));
    }
    Ok(())
}

fn isolate(world: &mut World) {
    use crate::presentation::{
        GroundScatterLayer, UnderstoryReviewSpecimen, VistaGrassPresentation,
        VistaRockPresentation, VistaTreePresentation,
    };
    use adventuresim_tactical_core::prelude::SceneObstacle;
    let entities: Vec<_> = world
        .query_filtered::<Entity, Or<(
            With<SceneObstacle>,
            With<GroundScatterLayer>,
            With<VistaTreePresentation>,
            With<VistaGrassPresentation>,
            With<VistaRockPresentation>,
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

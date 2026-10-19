//! One source model per process, one fresh bank observation per fixed pose.

use super::super::{
    capture_state::{CapturePhase, SceneCaptureState},
    setup_scene,
};
use super::*;
use crate::presentation::TacticalGameplayCamera;
use adventuresim_environment::render::{
    ForestDetail, ForestViewCamera, Level, LevelSelection,
    benchmark::{
        CompletionPolicy, ForestBenchmark, ForestRequestId, ValidationPolicy, WindowStart,
    },
    trees::{TreeFeed, TreeForestPlugin, TreeMeshStats},
};

#[derive(Resource)]
struct PoseProbe {
    view: usize,
    probe: ForestBenchmark,
}

#[derive(Resource)]
pub(in super::super) struct ReadyPose(usize);

pub(in super::super) fn install(app: &mut App, review: Review) {
    let source = if review.role == Role::Source {
        Some(source_draws::SourceDraws::install(app))
    } else {
        app.add_plugins(TreeForestPlugin);
        ForestBenchmark::new(
            web_time::Instant::now(),
            ValidationPolicy::EveryFrame,
            ForestRequestId::FIRST,
            CompletionPolicy::WaitedDiagnostic,
        )
        .install_shared(app);
        None
    };
    let evidence = Evidence::new(review.metadata.clone(), source);
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
    let depth = {
        let projection = world
            .query_filtered::<&Projection, With<TacticalGameplayCamera>>()
            .single(world)?;
        cameras::PerspectiveDepth::from_projection(projection)?
    };
    world.insert_resource(depth);
    let role = world.resource::<Review>().role;
    let root = world.resource::<Review>().root;
    let source = world
        .resource_mut::<Review>()
        .source
        .take()
        .context("source geometry consumed once")?;
    if role == Role::Source {
        source.spawn(world, root)?;
    } else {
        let camera = world
            .query_filtered::<Entity, With<TacticalGameplayCamera>>()
            .single(world)?;
        world.entity_mut(camera).insert(ForestViewCamera);
        let mut detail = world.resource::<Review>().forest.bank().library().detail;
        detail.level = LevelSelection::Fixed(Level::Full);
        world.insert_resource::<ForestDetail>(detail);
    }
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
    world.resource_mut::<SceneCaptureState>().tree_bank_evidence = Some(evidence);
    let adapter = world
        .get_resource::<bevy::render::renderer::RenderAdapterInfo>()
        .context("actual native renderer adapter information")?;
    let metadata = serde_json::json!({
        "adapter": {"name": adapter.name, "backend": format!("{:?}", adapter.backend), "device_type": format!("{:?}", adapter.device_type), "driver": adapter.driver, "driver_info": adapter.driver_info},
        "projection_depth": world.resource::<cameras::PerspectiveDepth>(),
        "actual_virtual_seconds": world.resource::<Time<Virtual>>().elapsed_secs_f64(),
        "scope": "actual native renderer, isolated held content comparison"});
    let output = world.resource::<SceneCaptureState>().output.clone();
    fs::write(
        output.join("bank-review-runtime.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    Ok(())
}

fn advance(world: &mut World) -> bevy::prelude::Result {
    advance_inner(world).map_err(Into::into)
}

fn advance_inner(world: &mut World) -> Result<()> {
    let Some(state) = world.get_resource::<SceneCaptureState>() else {
        return Ok(());
    };
    if state.view >= state.views.len() || state.phase == CapturePhase::Configure {
        return Ok(());
    }
    let view = state.view;
    let CapturePose::Bank(expected_camera) = state.views[view].pose else {
        anyhow::bail!("bank review pose required");
    };
    let (camera, transform, projection) = world.query_filtered::<(&bevy::prelude::Camera, &GlobalTransform, &Projection), With<TacticalGameplayCamera>>().single(world)?;
    if !expected_camera.matches(
        camera,
        transform,
        projection,
        world.resource::<cameras::PerspectiveDepth>(),
    ) {
        world.resource::<Evidence>().camera_drift(view)?;
        anyhow::bail!("held review camera/projection drift");
    }
    let actual_camera = expected_camera.actual(transform, projection)?;
    let role = world.resource::<Review>().role;
    if role == Role::Source {
        if let Some(proof) = world.resource::<source_draws::SourceDraws>().proof(view) {
            world.resource::<Evidence>().record(
                view,
                serde_json::json!({"camera": actual_camera, "draws": proof}),
            )?;
            world.insert_resource(ReadyPose(view));
        }
        return Ok(());
    }
    if world
        .get_resource::<PoseProbe>()
        .is_none_or(|pose| pose.view != view)
    {
        let forest = Arc::clone(&world.resource::<Review>().forest);
        let request = world
            .resource::<ForestBenchmark>()
            .request_id()
            .successor()
            .context("fixed pose request overflow")?;
        let probe = ForestBenchmark::new(
            web_time::Instant::now(),
            ValidationPolicy::EveryFrame,
            request,
            CompletionPolicy::WaitedDiagnostic,
        );
        probe.expect_instances(forest.expected_instances());
        probe.activate(world);
        world.resource_mut::<TreeFeed>().show(forest);
        world.resource::<Evidence>().track(probe.clone())?;
        world.insert_resource(PoseProbe { view, probe });
        return Ok(());
    }
    if world
        .get_resource::<ReadyPose>()
        .is_some_and(|ready| ready.0 == view)
    {
        return Ok(());
    }
    let probe = world.resource::<PoseProbe>().probe.clone();
    ensure!(!probe.status().failed, "bank draw observation failed");
    if !world.resource::<TreeFeed>().is_drawn() || probe.first_completed_frame().is_none() {
        return Ok(());
    }
    if matches!(
        probe.validation().begin_sampling(),
        WindowStart::Waiting | WindowStart::Opened
    ) {
        return Ok(());
    }
    if !probe.validation().finish_sampling() {
        return Ok(());
    }
    let camera = world
        .query_filtered::<Entity, With<TacticalGameplayCamera>>()
        .single(world)?;
    let receipt = probe
        .final_completed_frame()
        .context("missing final bank receipt")?;
    ensure!(
        receipt.camera() == camera && receipt.request() == probe.request_id(),
        "bank receipt view/request mismatch"
    );
    let counts = probe.counts();
    ensure!(
        counts.queued_writes == counts.submitted_writes
            && counts.queued_writes > 0
            && counts.discarded_writes == 0
            && counts.superseded_writes == 0
            && counts.gpu_failures == 0
            && counts.expected_branch_instances == 1
            && counts.expected_leaf_instances == 1,
        "incomplete one-tree bank work"
    );
    let inventory = probe.native_residency().context("missing bank inventory")?;
    let stats = world.resource::<TreeMeshStats>();
    let proof = serde_json::json!({"role": Role::Candidate, "view": view, "receipt": receipt,
        "validation": probe.validation().evidence(), "native_residency": inventory, "mesh_stats": stats,
        "queued_writes": counts.queued_writes, "submitted_writes": counts.submitted_writes,
        "gpu_failures": counts.gpu_failures,
        "ground_scope": "forest ground membership empty; tactical terrain retained separately in screenshots"});
    world.resource::<Evidence>().record(
        view,
        serde_json::json!({"camera": actual_camera, "draws": proof}),
    )?;
    world.insert_resource(ReadyPose(view));
    Ok(())
}

pub(in super::super) fn ready(
    state: Option<Res<SceneCaptureState>>,
    review: Option<Res<Review>>,
    ready: Option<Res<ReadyPose>>,
    cameras: Query<
        (&bevy::prelude::Camera, &GlobalTransform, &Projection),
        With<TacticalGameplayCamera>,
    >,
    depth: Option<Res<cameras::PerspectiveDepth>>,
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
    let Some(view) = state.views.get(state.view) else {
        return false;
    };
    let CapturePose::Bank(expected) = view.pose else {
        return false;
    };
    let Ok((camera, transform, projection)) = cameras.single() else {
        return false;
    };
    ready.is_some_and(|ready| ready.0 == state.view)
        && depth.is_some_and(|depth| expected.matches(camera, transform, projection, &depth))
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
    let CapturePose::Bank(expected) = view.pose else {
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

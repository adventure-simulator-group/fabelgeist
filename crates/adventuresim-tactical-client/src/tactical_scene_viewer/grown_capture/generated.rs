//! Fixed and adaptive capture of actual generated physics roots and live cache.
use super::super::{
    capture_state::{CapturePhase, SceneCaptureState},
    obstacle_setup::CapturedObstacleBatch,
};
use super::*;
use crate::presentation::{GrownSceneRequest, GrownSource, TreeCacheInspection};
use bevy::{camera::visibility::VisibilitySystems, ecs::system::SystemState};

pub(super) use inspection::Inspector;
mod adaptive_plates;
mod census;
pub(super) mod demand;
mod inspection;
mod plates;
pub(super) mod selection;
mod vista;

// Aggregate of two selected models and all three native geometry levels.
const CATALOGUE_MESH_BYTE_LIMIT: u64 = 256 * 1024 * 1024;

#[derive(Resource)]
pub(in super::super) struct Review {
    pub(super) request: GrownSceneRequest,
    pub(super) subject: Subject,
    control: controls::BatchControl,
    requested: Vec<String>,
    metadata: serde_json::Value,
}

#[derive(Resource)]
pub(in super::super) struct Configured;

impl Review {
    pub(super) fn admit(
        options: GrownCaptureOptions,
        setup: &SceneSetupData,
        requested: &[String],
    ) -> Result<Self> {
        ensure!(
            options.role.is_none(),
            "generated mode does not accept a specimen encoding role"
        );
        let first = PlantId::from_word(options.source.context("first generated source required")?);
        let second = PlantId::from_word(
            options
                .second_source
                .context("second generated source required")?,
        );
        let selection = options.level.context("generated selection required")?;
        let control = options.control.context("generated control required")?;
        ensure!(
            control != controls::BatchControl::MissingInactiveTag
                || selection == LevelSelection::Adaptive,
            "inactive tag control requires adaptive selection"
        );
        let bytes = fs::read(options.library.context("generated library required")?)?;
        let bank = Arc::new(TreeBank::from_bytes(&bytes)?);
        let request = GrownSceneRequest::from_sources(
            bank.clone(),
            vec![first, second],
            GrownMeshBudget {
                part_vertices: GrownPartVertexLimit::from_vertices(PART_VERTEX_LIMIT)?,
                total_mesh_bytes: GrownMeshBytes::from_bytes(CATALOGUE_MESH_BYTE_LIMIT),
            },
            selection,
        )?;
        let sources: Vec<_> = bank
            .library()
            .variant_indices()
            .zip(&bank.library().models)
            .filter(|(_, model)| model.source == first || model.source == second)
            .map(|(variant, model)| GrownSource {
                plant: model.source,
                variant,
            })
            .collect();
        let sources: [GrownSource; 2] = sources
            .try_into()
            .map_err(|_| anyhow::anyhow!("generated source pair"))?;
        ensure!(
            sources.iter().all(|source| {
                let model = &bank.library().models[source.variant.index()];
                model.branches.count > 0 && model.leaves.count > 0
            }),
            "generated leafy witness requires wood and leaves in both selected sources"
        );
        let trees = setup
            .generated
            .obstacles
            .iter()
            .filter(|obstacle| {
                matches!(
                    obstacle,
                    adventuresim_tactical_core::prelude::GeneratedObstacle::Tree { .. }
                )
            })
            .count();
        ensure!(trees > 0, "generated trial requires actual trees");
        let slugs = match selection {
            LevelSelection::Fixed(_) => plates::slugs(),
            LevelSelection::Adaptive => adaptive_plates::slugs(),
        };
        view_selection::validate(&slugs, requested).map_err(anyhow::Error::msg)?;
        let subject = Subject::Generated {
            bank: bank.id(),
            sources,
            selection,
            trees,
        };
        let source_metadata = sources
            .iter()
            .map(|source| {
                let model = &bank.library().models[source.variant.index()];
                serde_json::json!({"source":source,"branches":model.branches,"leaves":model.leaves,
                "bounds":model.bounds,"species":bank.library().species[model.species.index()].name})
            })
            .collect::<Vec<_>>();
        let metadata = serde_json::json!({"subject":subject,"library_bytes":bytes.len(),
            "sources":source_metadata,"input_digest":setup.input.digest()?,
            "scene_digest":setup.generated.digest,"environment":setup.environment,
            "part_vertex_limit":PART_VERTEX_LIMIT,"catalogue_mesh_byte_limit":CATALOGUE_MESH_BYTE_LIMIT,
            "encoding":"address","batch_control":control,"detail":bank.library().detail,
            "scope":"actual generated playable physical roots and render-only grown vista roots, two genuine generic sources, owned levels and explicit selection; recorded-frame ordinary color instance proof, not exact screenshot-frame, species calibration, weather, continuous motion, browser, physical-memory or arrival acceptance",
            "excluded":"Recipe diagnostic specimens excluded at their producers; grown vista shares the scene catalogue at Coarse; terrain, grass, rocks and original collider roots retained"});
        Ok(Self {
            request,
            subject,
            control,
            requested: requested.to_vec(),
            metadata,
        })
    }
}

pub(super) fn install(app: &mut App, review: Review) {
    app.init_resource::<demand::Demand>()
        .init_resource::<selection::SelectionHistory>();
    observation::install(app);
    ordinary_batches::OrdinaryBatches::install(app, review.control);
    app.insert_resource(review.subject)
        .insert_resource(review)
        .add_systems(
            PostStartup,
            preflight
                .after(super::super::setup_scene)
                .after(super::super::freeze_capture_clock),
        )
        .add_systems(PreUpdate, runtime::advance)
        .add_systems(
            PostUpdate,
            configure
                .after(VisibilitySystems::CheckVisibility)
                .before(observation::observe),
        );
}

fn preflight(world: &mut World) -> bevy::prelude::Result {
    let control = world.resource::<Review>().control;
    control.apply_original_batch(world)?;
    let digest = world.resource::<SceneCaptureState>().digest.clone();
    let receipt = CapturedObstacleBatch::check(world, &digest)?;
    world.resource_mut::<Review>().metadata["original_obstacle_batch"] =
        serde_json::to_value(receipt)?;
    let projection = world
        .query_filtered::<&Projection, With<crate::presentation::TacticalGameplayCamera>>()
        .single(world)?;
    let depth = cameras::PerspectiveDepth::from_projection(projection)?;
    world.insert_resource(depth);
    Ok(())
}

fn configure(world: &mut World) -> bevy::prelude::Result {
    configure_inner(world).map_err(Into::into)
}

fn configure_inner(world: &mut World) -> Result<()> {
    if world.contains_resource::<Configured>() {
        return Ok(());
    }
    // Update's existing dispatcher has now run; a mismatched cache is a failure.
    let assets = SystemState::<TreeCacheInspection>::new(world)
        .get(world)?
        .grown_scene()?;
    if world
        .get::<crate::presentation::VistaTreeCensus>(assets.scene())
        .is_none()
    {
        return Ok(());
    }
    let held = census::HeldScene::admit(world, assets)?;
    let resolved = match held.assets.selection() {
        LevelSelection::Fixed(_) => plates::resolve(world, &held)?,
        LevelSelection::Adaptive => adaptive_plates::resolve(world, &held)?,
    };
    let review = world.resource::<Review>();
    let views = view_selection::select(
        &resolved.cameras,
        &review.requested,
        match held.assets.selection() {
            LevelSelection::Fixed(_) => view_selection::ViewSequence::Independent,
            LevelSelection::Adaptive => view_selection::ViewSequence::Prefix,
        },
    )
    .map_err(anyhow::Error::msg)?;
    let mut metadata = review.metadata.clone();
    metadata["catalogue_and_roots"] = held.report();
    metadata["camera_support"] = resolved.support.report();
    metadata["cameras"] = serde_json::to_value(
        views
            .iter()
            .map(|view| match view.pose {
                CapturePose::Grown(camera) => Ok(camera),
                _ => Err(anyhow::anyhow!("generated plate camera missing")),
            })
            .collect::<Result<Vec<_>>>()?,
    )?;
    let observer = world
        .resource::<ordinary_batches::OrdinaryBatches>()
        .clone();
    {
        metadata["retained_tag_admission"] = observer.admit_retained_members(
            world,
            held.roots
                .iter()
                .flat_map(|root| root.parts().map(|part| part.member.entity))
                .chain(held.vista.entities())
                .collect(),
            views.len(),
        )?;
    }
    let review = world.resource::<Review>();
    let evidence = Evidence::new(metadata, review.subject, observer);
    {
        let mut state = world.resource_mut::<SceneCaptureState>();
        ensure!(
            state.profile == GENERATED_PROFILE
                && state.phase == CapturePhase::Configure
                && state.view == 0
                && state.views.is_empty()
                && state.captures.is_empty(),
            "generated mode must configure once before capture"
        );
        state.requested_views = views
            .iter()
            .filter(|view| !view.warmup)
            .map(|view| view.slug.to_owned())
            .collect();
        state.views = views;
        state.grown_evidence = Some(evidence.clone());
    }
    world.insert_resource(held);
    world.insert_resource(resolved.support);
    world.insert_resource(evidence);
    world.insert_resource(Configured);
    let adapter = world.resource::<bevy::render::renderer::RenderAdapterInfo>();
    let runtime = serde_json::json!({"adapter":{"name":adapter.name,
        "backend":format!("{:?}",adapter.backend),"device_type":format!("{:?}",adapter.device_type)},
        "subject":world.resource::<Subject>(),
        "actual_virtual_seconds":world.resource::<Time<Virtual>>().elapsed_secs_f64(),
        "scope":"actual generated-root explicit-policy native diagnostic; not an arrival benchmark"});
    fs::write(
        world
            .resource::<SceneCaptureState>()
            .output
            .join("grown-runtime.json"),
        serde_json::to_vec_pretty(&runtime)?,
    )?;
    Ok(())
}

pub(in super::super) fn pending_adaptive_configuration(
    state: Option<Res<SceneCaptureState>>,
    subject: Option<Res<Subject>>,
    configured: Option<Res<Configured>>,
) -> bool {
    configured.is_some()
        && subject.is_some_and(|subject| {
            matches!(
                *subject,
                Subject::Generated {
                    selection: LevelSelection::Adaptive,
                    ..
                }
            )
        })
        && state.is_some_and(|state| {
            state.phase == CapturePhase::Configure && state.view < state.views.len()
        })
}

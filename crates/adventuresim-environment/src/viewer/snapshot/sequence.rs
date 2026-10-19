//! Serial full-population arrivals in one renderer, with fresh request evidence.

use std::{num::NonZeroU64, path::PathBuf, sync::Arc, time::Instant};

use adventuresim_environment::{
    prepared::{PopulationId, TreeBank},
    render::{
        ForestDetail, LevelSelection,
        benchmark::{ForestBenchmark, ForestRequestId, ValidationPolicy},
        trees::{TreeBatching, TreeBranchMaterial, TreeLeafMaterial, TreeResidencyPolicy},
    },
};
use anyhow::{Result, ensure};
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::{
        storage::ShaderBuffer,
        view::screenshot::{Captured, Screenshot},
    },
};
use serde::{Deserialize, Serialize};

use super::{Framing, Request, Still};
use crate::{
    benchmark::{self, ArrivalPath, Measurement},
    input::{self, FrozenForest, Representation, Source},
};

const MAX_MANIFEST_BYTES: u64 = 8192;
pub(super) const RETIREMENT_FRAMES: u32 = 60;

/// Checked native diagnostic manifest; no population is prefetched for B/return.
pub struct Plan {
    initial_population: PathBuf,
    unseen_population: PathBuf,
    output: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    bank_directory: PathBuf,
    unseen_population: PathBuf,
    capture_view: PathBuf,
}

#[derive(Resource)]
pub(super) struct Progress {
    plan: Plan,
    bank: Arc<TreeBank>,
    initial_id: PopulationId,
    phase: Phase,
    state: PhaseState,
    checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Serialize)]
struct Checkpoint {
    phase: Phase,
    request: ForestRequestId,
    assets: AssetCounts,
}

/// Main-world asset values, not physical GPU allocations or RSS.
#[derive(Clone, Copy, Serialize)]
pub(super) struct AssetCounts {
    mesh_values: usize,
    storage_values: usize,
    branch_materials: usize,
    leaf_materials: usize,
    standard_materials: usize,
    images: usize,
    screenshot_entities: usize,
    scene_mesh_entities: usize,
    render_mesh_assets: usize,
    render_storage_assets: usize,
}

#[derive(SystemParam)]
pub(super) struct Inventory<'w, 's> {
    meshes: Res<'w, Assets<Mesh>>,
    buffers: Res<'w, Assets<ShaderBuffer>>,
    branches: Option<Res<'w, Assets<TreeBranchMaterial>>>,
    leaves: Option<Res<'w, Assets<TreeLeafMaterial>>>,
    standard: Res<'w, Assets<StandardMaterial>>,
    images: Res<'w, Assets<Image>>,
    screenshots: Query<'w, 's, (Entity, Has<Captured>), With<Screenshot>>,
    entities: Query<'w, 's, Entity, With<Mesh3d>>,
}

#[derive(Clone, Copy, Serialize)]
enum Phase {
    Initial,
    Unseen,
    Return,
}

enum PhaseState {
    Capturing,
    Complete,
    Finished,
}

impl Phase {
    fn request(self) -> ForestRequestId {
        let ordinal = match self {
            Self::Initial => NonZeroU64::MIN,
            Self::Unseen => NonZeroU64::MIN.saturating_add(1),
            Self::Return => NonZeroU64::MIN.saturating_add(2),
        };
        ForestRequestId::from_ordinal(ordinal)
    }
    fn folder(self) -> &'static str {
        match self {
            Self::Initial => "01-initial",
            Self::Unseen => "02-unseen",
            Self::Return => "03-return",
        }
    }
    fn next(self) -> Option<Self> {
        match self {
            Self::Initial => Some(Self::Unseen),
            Self::Unseen => Some(Self::Return),
            Self::Return => None,
        }
    }
}

impl Inventory<'_, '_> {
    pub(super) fn snapshot(
        &self,
        probe: &ForestBenchmark,
        captured: Entity,
    ) -> Result<AssetCounts> {
        ensure!(
            self.screenshots
                .iter()
                .all(|(entity, complete)| entity == captured && complete)
                && self.screenshots.get(captured).is_ok(),
            "pending or stale screenshot remains at the final request-owned receipt"
        );
        let native = probe
            .native_residency()
            .ok_or_else(|| anyhow::anyhow!("missing final native residency"))?;
        ensure!(
            native.matches_scene_entities(self.entities.iter()),
            "scene meshes differ from the current request's validated membership"
        );
        let native = native.asset_counts();
        Ok(AssetCounts {
            mesh_values: self.meshes.len(),
            storage_values: self.buffers.len(),
            branch_materials: self.branches.as_ref().map_or(0, |assets| assets.len()),
            leaf_materials: self.leaves.as_ref().map_or(0, |assets| assets.len()),
            standard_materials: self.standard.len(),
            images: self.images.len(),
            screenshot_entities: self.screenshots.iter().count(),
            scene_mesh_entities: self.entities.iter().count(),
            render_mesh_assets: native.meshes,
            render_storage_assets: native.storage,
        })
    }
}

impl Progress {
    pub(super) fn new(plan: Plan, forest: &FrozenForest) -> Result<Self> {
        let FrozenForest::Shared(forest) = forest else {
            anyhow::bail!("arrival sequence requires shared assets")
        };
        ensure!(
            forest.population().frame.terrain.is_some(),
            "sequence requires complete terrain"
        );
        Ok(Self {
            plan,
            bank: Arc::clone(forest.bank()),
            initial_id: forest.id(),
            phase: Phase::Initial,
            state: PhaseState::Capturing,
            checkpoints: Vec::new(),
        })
    }
    pub(super) fn finish(&mut self, request: ForestRequestId, assets: AssetCounts) -> Result<()> {
        ensure!(
            request == self.phase.request() && matches!(self.state, PhaseState::Capturing),
            "stale sequence screenshot"
        );
        if let Some(first) = self.checkpoints.first() {
            ensure!(
                assets.mesh_values <= first.assets.mesh_values
                    && assets.storage_values <= first.assets.storage_values
                    && assets.branch_materials <= first.assets.branch_materials
                    && assets.leaf_materials <= first.assets.leaf_materials
                    && assets.standard_materials <= first.assets.standard_materials
                    && assets.images <= first.assets.images
                    && assets.screenshot_entities <= first.assets.screenshot_entities,
                "post-drain main-world assets exceed the first bank checkpoint"
            );
            ensure!(
                assets.render_mesh_assets <= first.assets.render_mesh_assets
                    && assets.render_storage_assets <= first.assets.render_storage_assets,
                "post-drain render assets exceed the first bank checkpoint"
            );
        }
        self.checkpoints.push(Checkpoint {
            phase: self.phase,
            request,
            assets,
        });
        std::fs::write(
            self.plan.output.join("sequence.json"),
            serde_json::to_vec_pretty(&self.checkpoints)?,
        )?;
        self.state = PhaseState::Complete;
        Ok(())
    }
    fn advance(&mut self, world: &mut World) -> Result<()> {
        let Some(phase) = self.phase.next() else {
            self.state = PhaseState::Finished;
            world.write_message(AppExit::Success);
            return Ok(());
        };
        let started = Instant::now();
        let path = match phase {
            Phase::Unseen => &self.plan.unseen_population,
            Phase::Return => &self.plan.initial_population,
            Phase::Initial => anyhow::bail!("initial arrival cannot follow another request"),
        };
        let loaded = input::load_population(Arc::clone(&self.bank), path)?;
        let FrozenForest::Shared(forest) = &loaded.forest else {
            anyhow::bail!("shared population loader returned unique input")
        };
        ensure!(
            forest.population().frame.terrain.is_some(),
            "sequence requires complete terrain"
        );
        match phase {
            Phase::Unseen => ensure!(forest.id() != self.initial_id, "unseen population equals A"),
            Phase::Return => ensure!(forest.id() == self.initial_id, "return population changed"),
            Phase::Initial => {}
        }
        let folder = self.plan.output.join(phase.folder());
        std::fs::create_dir(&folder)?;
        let detail = *world.resource::<ForestDetail>();
        let view = world.resource::<Still>().view.clone();
        let probe = ForestBenchmark::new(started, ValidationPolicy::default(), phase.request());
        let mut instances = loaded.forest.expected_instances();
        if !detail.leaves {
            instances.leaves = 0;
        }
        probe.expect_instances(instances);
        probe.activate(world);
        world.insert_resource(Measurement::new(
            benchmark::Request {
                identity: phase.request(),
                started,
                report: folder.join("measurement.json"),
            },
            loaded.preparation,
            &loaded.frame,
            detail,
            view,
            ArrivalPath::PreparedShared,
        ));
        world
            .resource_mut::<Still>()
            .restart(loaded.forest, folder.join("forest.png"));
        self.phase = phase;
        self.state = PhaseState::Capturing;
        Ok(())
    }
    fn is_complete(&self) -> bool {
        matches!(self.state, PhaseState::Complete)
    }
}

pub fn run(
    manifest: PathBuf,
    output: PathBuf,
    batching: TreeBatching,
    residency: TreeResidencyPolicy,
    started: Instant,
) -> Result<()> {
    ensure!(
        std::fs::metadata(&manifest)?.len() <= MAX_MANIFEST_BYTES,
        "sequence manifest exceeds8KiB"
    );
    let document: Document = serde_json::from_slice(&std::fs::read(&manifest)?)?;
    crate::capture_view::CaptureView::read(&document.capture_view)?;
    ensure!(!output.exists(), "sequence output already exists");
    let initial_population = document.bank_directory.join("population.forest");
    ensure!(
        document.unseen_population != initial_population,
        "B must have a separate descriptor file"
    );
    std::fs::create_dir_all(&output)?;
    let first = output.join(Phase::Initial.folder());
    std::fs::create_dir(&first)?;
    super::run(Request {
        sequence: Some(Plan {
            initial_population,
            unseen_population: document.unseen_population,
            output,
        }),
        residency,
        view: Some(document.capture_view),
        tree_layout: Default::default(),
        tree_batching: batching,
        validation: Default::default(),
        render_profile: None,
        path: first.join("forest.png"),
        source: Source::Prepared {
            directory: document.bank_directory,
            representation: Representation::Shared,
        },
        level: LevelSelection::Adaptive,
        framing: Framing {
            closeup: false,
            aim: 0.0,
            yaw: None,
            zoom: 1.0,
        },
        frames: None,
        orbit: false,
        refeed: None,
        benchmark: Some(benchmark::Request {
            identity: ForestRequestId::FIRST,
            report: first.join("measurement.json"),
            started,
        }),
    })
}

pub(super) fn advance(world: &mut World) {
    if !world.resource::<Progress>().is_complete() {
        return;
    }
    let Some(mut progress) = world.remove_resource::<Progress>() else {
        return;
    };
    if let Err(error) = progress.advance(world) {
        error!("{error:#}");
        world.write_message(AppExit::error());
    }
    world.insert_resource(progress);
}

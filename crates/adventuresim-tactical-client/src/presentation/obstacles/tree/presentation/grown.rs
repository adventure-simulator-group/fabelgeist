//! An admitted model catalogue, shared by real roots through the normal cache.

use std::sync::Arc;

use adventuresim_environment::{
    prepared::{GrownMeshBudget, GrownMeshBytes, TreeBank, VariantIndex},
    render::{
        Level, LevelSelection,
        grown::{GrownAssets, GrownBankAssets},
    },
};
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;

use super::SceneEnvironment;

pub(crate) use attachment::{GrownRootAttachment, GrownRootHierarchy};
pub(super) use presentation::present;
pub(in crate::presentation) use selection::install_grown_streaming;
pub(crate) use streaming::GrownPartBinding;
pub(in crate::presentation) use streaming::{GrownTreeStream, GrownVisualFrame};

mod attachment;
mod construction;
mod presentation;
mod residency;
mod roots;
mod selection;
mod streaming;

const GROWN_SOURCE_SELECTION: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("visual.tree.grown-catalogue");

/// Checked source membership and explicit level selection policy.
#[derive(Clone)]
pub(crate) struct GrownSceneRequest {
    bank: Arc<TreeBank>,
    sources: Vec<GrownSource>,
    budget: GrownMeshBudget,
    selection: LevelSelection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GrownSource {
    pub(crate) plant: PlantId,
    pub(crate) variant: VariantIndex,
}

pub(super) struct GrownTreeCache {
    pub catalogue: Arc<GrownCatalogueAssets>,
    pub selection: LevelSelection,
}

pub(crate) struct GrownCatalogueAssets {
    bank: Arc<GrownBankAssets>,
    sources: Vec<GrownSource>,
    part_vertices: adventuresim_environment::prepared::GrownPartVertexLimit,
    logical_mesh_bytes: GrownMeshBytes,
    models: Vec<GrownModelAssets>,
}

/// Read-only ownership snapshot obtained from the live scene cache inspector.
pub(crate) struct GrownSceneAssets {
    scene: Entity,
    catalogue: Arc<GrownCatalogueAssets>,
    selection: LevelSelection,
}

struct GrownModelAssets {
    source: GrownSource,
    levels: GrownLevels<GrownAssets>,
}

#[derive(Clone)]
struct GrownLevels<T> {
    full: T,
    middle: T,
    coarse: T,
}

impl GrownSceneAssets {
    pub(super) fn from_cache(scene: Entity, cache: &GrownTreeCache) -> Self {
        Self {
            scene,
            catalogue: cache.catalogue.clone(),
            selection: cache.selection,
        }
    }
    pub(crate) fn scene(&self) -> Entity {
        self.scene
    }
    pub(crate) fn catalogue(&self) -> &Arc<GrownCatalogueAssets> {
        &self.catalogue
    }
    pub(crate) fn selection(&self) -> LevelSelection {
        self.selection
    }
}

impl PartialEq for GrownSceneRequest {
    fn eq(&self, other: &Self) -> bool {
        self.bank.id() == other.bank.id()
            && self.sources == other.sources
            && self.budget == other.budget
            && self.selection == other.selection
    }
}

impl Eq for GrownSceneRequest {}

impl GrownSceneRequest {
    /// A new scene intent; live attachments reject a changed fixed level.
    pub(crate) fn for_fixed_level(&self, level: Level) -> Self {
        Self {
            selection: LevelSelection::Fixed(level),
            ..self.clone()
        }
    }

    pub(crate) fn from_sources(
        bank: Arc<TreeBank>,
        mut plants: Vec<PlantId>,
        budget: GrownMeshBudget,
        selection: LevelSelection,
    ) -> Result<Self> {
        ensure!(!plants.is_empty(), "empty grown catalogue");
        plants.sort_by_key(|plant| plant.word());
        ensure!(
            plants.windows(2).all(|pair| pair[0] != pair[1]),
            "duplicate grown catalogue source"
        );
        let mut sources = Vec::with_capacity(plants.len());
        for plant in plants {
            let mut matches = bank
                .library()
                .variant_indices()
                .zip(&bank.library().models)
                .filter(|(_, model)| model.source == plant);
            let (variant, model) = matches.next().context("grown catalogue source is absent")?;
            ensure!(matches.next().is_none(), "ambiguous grown catalogue source");
            ensure!(model.branches.count > 0, "empty grown source");
            bank.library()
                .species
                .get(model.species.index())
                .context("grown catalogue species is absent")?;
            sources.push(GrownSource { plant, variant });
        }
        Ok(Self {
            bank,
            sources,
            budget,
            selection,
        })
    }

    pub(super) fn selection(&self) -> LevelSelection {
        self.selection
    }
}

impl GrownCatalogueAssets {
    pub(crate) fn sources(&self) -> &[GrownSource] {
        &self.sources
    }
    pub(crate) fn bank_assets(&self) -> &Arc<GrownBankAssets> {
        &self.bank
    }
    pub(crate) fn logical_mesh_bytes(&self) -> GrownMeshBytes {
        self.logical_mesh_bytes
    }

    pub(super) fn matches(&self, request: &GrownSceneRequest) -> bool {
        self.bank.bank().id() == request.bank.id()
            && self.sources == request.sources
            && self.part_vertices == request.budget.part_vertices
    }

    pub(super) fn verify_reuse(&self, request: &GrownSceneRequest, world: &World) -> Result<()> {
        ensure!(self.matches(request), "grown catalogue identity changed");
        ensure!(
            self.logical_mesh_bytes <= request.budget.total_mesh_bytes,
            "grown catalogue exceeds requested aggregate mesh budget"
        );
        GrownAssets::validate_target(world)?;
        self.bank.validate_world(world)
    }

    pub(super) fn bank_for(&self, request: &GrownSceneRequest) -> Option<Arc<GrownBankAssets>> {
        (self.bank.bank().id() == request.bank.id()).then(|| self.bank.clone())
    }

    pub(crate) fn select(&self, position: Vec3) -> GrownSource {
        let seed = crate::presentation::obstacle_seed(position);
        let index = GROWN_SOURCE_SELECTION
            .rng(seed, &[])
            .index(self.sources.len());
        self.sources[index]
    }

    pub(crate) fn assets(&self, source: GrownSource, level: Level) -> Result<&GrownAssets> {
        let model = self
            .models
            .iter()
            .find(|model| model.source == source)
            .context("grown root source is absent from the resident catalogue")?;
        Ok(model.levels.get(level))
    }
}

impl<T> GrownLevels<T> {
    fn get(&self, level: Level) -> &T {
        match level {
            Level::Full => &self.full,
            Level::Middle => &self.middle,
            Level::Coarse => &self.coarse,
        }
    }
}

impl serde::Serialize for GrownSource {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        // Botanical identity becomes a native number only at this JSON boundary.
        let mut record = serializer.serialize_struct("GrownSource", 2)?;
        record.serialize_field("plant", &self.plant.word())?;
        record.serialize_field("variant", &self.variant)?;
        record.end()
    }
}

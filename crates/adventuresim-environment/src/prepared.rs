//! Frozen rendering assets: offline growth, reusable tree models and placements.
//!
//! These assets contain rendering state, not a resumable ecosystem. The format
//! admits plain junctions and no grove proxies or vines. Representative trees
//! retain their source light samples; reuse approximates the local environment.

use std::num::NonZeroU16;
use std::sync::Arc;

use fabelgeist_botany::Biome;
use fabelgeist_botany::identity::{PlantId, SpeciesId};
use fabelgeist_botany::viewer::sim::{
    BranchInstance, LeafInstance, PlantInstance, Snapshot, SpeciesInfo, StructureGeometry,
    TerrainGeometry,
};
use fabelgeist_math::units::{Metres, Ratio};
use serde::{Deserialize, Serialize};

use crate::render::ForestDetail;

mod bake;
mod codec;
mod form;
mod grids;
#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
mod grown;
mod meshes;
mod resident;
mod selection;
mod validation;
mod visibility;

pub use codec::{
    decode_library, decode_population, decode_unique, encode_library, encode_population,
    encode_unique,
};
pub use grids::{AddressMesh, MeshClass, RecordCapacity, RecordCount};
#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
pub use grown::{
    GrownAuthoringContext, GrownExpansionBudget, GrownGeometry, GrownGeometryPart,
    GrownPartVertexLimit,
};
pub(crate) use meshes::admit_address_mesh_bytes;
pub use meshes::{CanonicalTopologyBytes, TopologyRequirements};
pub use resident::{PopulationId, SharedForest, TreeBank};
pub use visibility::TreeVisibility;

#[cfg(target_family = "wasm")]
pub use codec::BrowserIdentityFailure;

const LIBRARY_DIGEST_BYTES: usize = 32;

pub type Result<T> = std::result::Result<T, PreparedError>;

/// Stable identity and root in the source area's world frame, in metres.
#[derive(Clone, Copy, Debug)]
pub struct TreeRoot {
    pub id: PlantId,
    pub position: [Metres; 3],
}

/// A library-local ordinal, unrelated to stable botanical identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariantIndex(usize);

/// SHA-256 over the complete encoded model library, including its LOD settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryId([u8; LIBRARY_DIGEST_BYTES]);

/// Model-local to area-world transform: positive XZ/height scales, then Y yaw,
/// then root translation. Normals require the inverse transpose of the scale.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TreePose {
    origin: [Metres; 3],
    horizontal: Ratio,
    vertical: Ratio,
    yaw: TreeYaw,
}

/// Rotation about the tree's upward axis.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TreeYaw(f32);

#[derive(Clone, Copy, Debug)]
pub struct TreePlacement {
    pub id: PlantId,
    pub variant: VariantIndex,
    pub pose: TreePose,
    pub visibility: TreeVisibility,
}

/// A conservative box in model-local metres, valid for all stored LODs.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TreeBounds {
    pub low: [Metres; 3],
    pub high: [Metres; 3],
}

/// The original unique forest, frozen before renderer preparation.
pub struct PreparedUnique {
    pub snapshot: Snapshot,
    pub roots: Vec<TreeRoot>,
    pub detail: ForestDetail,
    pub label: String,
    pub focus: Option<TreeFocus>,
}

/// Frozen source framing, so model selection cannot move the comparison camera.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TreeFocus {
    pub root: [Metres; 3],
    pub height: Metres,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ForestCounts {
    pub plants: usize,
    pub branches: usize,
    pub leaves: usize,
}

#[derive(Clone)]
pub struct ForestFrame {
    pub label: String,
    pub detail: ForestDetail,
    pub time: f32,
    pub source: ForestCounts,
    pub bounds: [[f32; 3]; 2],
    pub biome: Biome,
    pub terrain: Option<Arc<TerrainGeometry>>,
    pub structures: Option<Arc<StructureGeometry>>,
    pub focus: Option<TreeFocus>,
}

/// Representative biological records; canonical topology is derived at use.
pub struct TreeLibrary {
    pub species: Vec<SpeciesInfo>,
    pub branches: Vec<BranchInstance>,
    pub leaves: Vec<LeafInstance>,
    pub plants: Vec<PlantInstance>,
    pub models: Vec<TreeModel>,
    pub detail: ForestDetail,
}

pub struct TreeModel {
    pub source: PlantId,
    pub species: SpeciesId,
    pub bounds: TreeBounds,
    pub branches: RecordSpan,
    pub leaves: RecordSpan,
}

/// An explicit GPU record-array boundary: a start word and record count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordSpan {
    pub start: u32,
    pub count: u32,
}

/// Regional descriptors and ground, bound to a particular model bank.
pub struct TreePopulation {
    pub library: LibraryId,
    pub placements: Vec<TreePlacement>,
    pub frame: ForestFrame,
}

#[derive(Clone, Copy, Debug)]
pub enum VariantBudget {
    AllPlants,
    PerSpecies(NonZeroU16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum TreeParts {
    Empty,
    Branches,
    Leaves,
    Both,
}

impl TreeParts {
    fn from_counts(branches: usize, leaves: usize) -> Self {
        match (branches > 0, leaves > 0) {
            (false, false) => Self::Empty,
            (true, false) => Self::Branches,
            (false, true) => Self::Leaves,
            (true, true) => Self::Both,
        }
    }
}

impl TreeModel {
    pub(super) fn parts(&self) -> TreeParts {
        TreeParts::from_counts(self.branches.count as usize, self.leaves.count as usize)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PreparedError {
    #[cfg(target_family = "wasm")]
    #[error(transparent)]
    BrowserIdentity(#[from] BrowserIdentityFailure),
    #[error("prepared forest address mesh count disagrees with capacity")]
    MeshCardinality,
    #[error("invalid prepared forest: {0}")]
    Invalid(&'static str),
    #[error("prepared forest metadata: {0}")]
    Metadata(#[from] serde_json::Error),
}

impl VariantIndex {
    pub fn index(self) -> usize {
        self.0
    }
}

impl TreeYaw {
    pub fn from_radians(value: f32) -> Result<Self> {
        if !value.is_finite() {
            return Err(PreparedError::Invalid("nonfinite yaw"));
        }
        Ok(Self(value))
    }
    pub fn radians(self) -> f32 {
        self.0
    }
}

impl TreePose {
    pub fn new(
        origin: [Metres; 3],
        horizontal: Ratio,
        vertical: Ratio,
        yaw: TreeYaw,
    ) -> Result<Self> {
        let pose = Self {
            origin,
            horizontal,
            vertical,
            yaw,
        };
        pose.validate()?;
        Ok(pose)
    }
    pub fn origin(self) -> [Metres; 3] {
        self.origin
    }
    pub fn horizontal(self) -> Ratio {
        self.horizontal
    }
    pub fn vertical(self) -> Ratio {
        self.vertical
    }
    pub fn yaw(self) -> TreeYaw {
        self.yaw
    }
    pub fn transform(self, point: [Metres; 3]) -> [Metres; 3] {
        let [x, y, z] = point.map(Metres::metres);
        let (sin, cos) = self.yaw.0.sin_cos();
        let h = self.horizontal.ratio();
        let v = self.vertical.ratio();
        let [ox, oy, oz] = self.origin.map(Metres::metres);
        [
            ox + h * (cos * x + sin * z),
            oy + v * y,
            oz + h * (-sin * x + cos * z),
        ]
        .map(Metres::from_metres)
    }
}

impl TreeBounds {
    pub fn transformed(self, pose: TreePose) -> Self {
        let mut low = [f32::INFINITY; 3];
        let mut high = [f32::NEG_INFINITY; 3];
        for mask in 0..8 {
            let corner = std::array::from_fn(|axis| {
                if mask & (1 << axis) == 0 {
                    self.low[axis]
                } else {
                    self.high[axis]
                }
            });
            let point = pose.transform(corner).map(Metres::metres);
            for axis in 0..3 {
                low[axis] = low[axis].min(point[axis]);
                high[axis] = high[axis].max(point[axis]);
            }
        }
        Self {
            low: low.map(Metres::from_metres),
            high: high.map(Metres::from_metres),
        }
    }
}

impl PreparedUnique {
    pub fn validate(&self) -> Result<()> {
        validation::unique(self)
    }
    pub fn bake(&self, budget: VariantBudget) -> Result<TreeLibrary> {
        bake::library(self, budget)
    }
    pub fn population(self, library: &TreeLibrary, id: LibraryId) -> Result<TreePopulation> {
        bake::population(self, library, id)
    }
}

impl ForestFrame {
    pub fn from_unique(unique: &PreparedUnique) -> Self {
        let snapshot = &unique.snapshot;
        Self {
            label: unique.label.clone(),
            detail: unique.detail,
            time: snapshot.time,
            source: ForestCounts {
                plants: snapshot.plants,
                branches: snapshot.branches.len(),
                leaves: snapshot.leaves.len(),
            },
            bounds: snapshot.bounds,
            biome: snapshot.biome,
            terrain: snapshot.terrain.clone(),
            structures: snapshot.structures.clone(),
            focus: unique.focus,
        }
    }
}

impl TreeLibrary {
    /// Library-owned ordinals, each guaranteed to address an existing model.
    pub fn variant_indices(&self) -> impl ExactSizeIterator<Item = VariantIndex> {
        (0..self.models.len()).map(VariantIndex)
    }
    pub fn validate(&self) -> Result<()> {
        validation::library(self)
    }
    /// Prospective shared address topology, without building any vertex/index arrays.
    pub fn topology_requirements(&self) -> Result<TopologyRequirements> {
        TopologyRequirements::from_models(&self.models, &self.detail)
    }
}

impl TreePopulation {
    pub fn validate(&self, library: &TreeLibrary, id: LibraryId) -> Result<()> {
        validation::population(self, library, id)
    }
}

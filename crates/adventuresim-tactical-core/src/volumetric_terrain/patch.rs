//! Immutable mesh buffers admitted before physics or renderer indexing.
use super::{TerrainRecipeError, TerrainRecipeResult, TerrainTransitionCollar};
use bevy::prelude::{Component, Reflect, ReflectComponent};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, opaque)]
#[serde(try_from = "PatchWire")]
pub struct SceneTerrainPatch {
    pub(super) transition_collar: TerrainTransitionCollar,
    pub(super) positions: Vec<[f32; 3]>,
    pub(super) normals: Vec<[f32; 3]>,
    pub(super) indices: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PatchWire {
    transition_collar: TerrainTransitionCollar,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}
impl TryFrom<PatchWire> for SceneTerrainPatch {
    type Error = TerrainRecipeError;
    fn try_from(wire: PatchWire) -> TerrainRecipeResult<Self> {
        Self::from_mesh_buffers(
            wire.transition_collar,
            wire.positions,
            wire.normals,
            wire.indices,
        )
    }
}
impl SceneTerrainPatch {
    /// Native renderer buffers use scene east/up/north metres. Producers supply
    /// unit normals; admission checks finite buffers, cardinality and indices.
    pub fn from_mesh_buffers(
        transition_collar: TerrainTransitionCollar,
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        indices: Vec<u32>,
    ) -> TerrainRecipeResult<Self> {
        if positions.is_empty() || indices.is_empty() {
            return Err(TerrainRecipeError::EmptySurface);
        }
        if positions.len() != normals.len()
            || u32::try_from(positions.len()).is_err()
            || !indices.len().is_multiple_of(3)
            || indices
                .iter()
                .any(|&index| index as usize >= positions.len())
        {
            return Err(TerrainRecipeError::MeshTopology);
        }
        if positions
            .iter()
            .chain(&normals)
            .flatten()
            .any(|value| !value.is_finite())
        {
            return Err(TerrainRecipeError::NonFiniteField);
        }
        Ok(Self {
            transition_collar,
            positions,
            normals,
            indices,
        })
    }
    pub fn transition_collar(&self) -> TerrainTransitionCollar {
        self.transition_collar
    }
    pub fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }
    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

/// Consumed renderer buffers, after admission by `SceneTerrainPatch`.
/// Mutating these native upload buffers does not mutate an admitted patch.
pub struct TerrainMeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}
impl SceneTerrainPatch {
    pub fn into_mesh_buffers(self) -> TerrainMeshBuffers {
        TerrainMeshBuffers {
            positions: self.positions,
            normals: self.normals,
            indices: self.indices,
        }
    }
}

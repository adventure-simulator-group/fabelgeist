//! Native serialization boundary: closed botanical enums and stable ID words.

use std::sync::Arc;

use fabelgeist_botany::{
    Biome,
    species::Foliage,
    viewer::sim::{SpeciesInfo, StructureGeometry, TerrainGeometry},
};
use serde::{Deserialize, Serialize};

use super::{Reader, records};
use crate::{
    prepared::{ForestCounts, ForestFrame, PreparedError, Result, TreeFocus},
    render::ForestDetail,
};

#[derive(Serialize, Deserialize)]
pub(super) struct SpeciesHeader {
    name: String,
    bark: [f32; 3],
    leaf: [f32; 3],
    foliage: u8,
    plants: usize,
}

#[derive(Serialize, Deserialize)]
pub(super) struct FrameHeader {
    pub label: String,
    pub detail: ForestDetail,
    pub time: f32,
    pub source: ForestCounts,
    pub bounds: [[f32; 3]; 2],
    biome: usize,
    terrain: bool,
    structures: bool,
    pub focus: Option<TreeFocus>,
}

impl From<&SpeciesInfo> for SpeciesHeader {
    fn from(info: &SpeciesInfo) -> Self {
        Self {
            name: info.name.clone(),
            bark: info.bark,
            leaf: info.leaf,
            foliage: info.foliage as u8,
            plants: info.plants,
        }
    }
}

impl SpeciesHeader {
    pub fn into_info(self) -> Result<SpeciesInfo> {
        let foliage = match self.foliage {
            0 => Foliage::Broadleaf,
            1 => Foliage::Needles,
            2 => Foliage::Fronds,
            _ => return Err(PreparedError::Invalid("foliage kind")),
        };
        Ok(SpeciesInfo {
            name: self.name,
            bark: self.bark,
            leaf: self.leaf,
            foliage,
            plants: self.plants,
        })
    }
}

impl FrameHeader {
    pub fn new(frame: &ForestFrame) -> Result<Self> {
        let biome = Biome::ALL
            .iter()
            .position(|value| *value == frame.biome)
            .ok_or(PreparedError::Invalid("biome"))?;
        Ok(Self {
            label: frame.label.clone(),
            detail: frame.detail,
            time: frame.time,
            source: frame.source,
            bounds: frame.bounds,
            biome,
            terrain: frame.terrain.is_some(),
            structures: frame.structures.is_some(),
            focus: frame.focus,
        })
    }
    pub fn read(self, reader: &mut Reader<'_>) -> Result<ForestFrame> {
        let biome = *Biome::ALL
            .get(self.biome)
            .ok_or(PreparedError::Invalid("biome ordinal"))?;
        let terrain = TerrainGeometry {
            vertices: reader.records()?,
            indices: reader.records()?,
        };
        let structures = StructureGeometry {
            vertices: reader.records()?,
            indices: reader.records()?,
        };
        if (!self.terrain && (!terrain.vertices.is_empty() || !terrain.indices.is_empty()))
            || (!self.structures
                && (!structures.vertices.is_empty() || !structures.indices.is_empty()))
        {
            return Err(PreparedError::Invalid("ground presence mismatch"));
        }
        Ok(ForestFrame {
            label: self.label,
            detail: self.detail,
            time: self.time,
            source: self.source,
            bounds: self.bounds,
            biome,
            terrain: self.terrain.then(|| Arc::new(terrain)),
            structures: self.structures.then(|| Arc::new(structures)),
            focus: self.focus,
        })
    }
}

pub(super) fn ground(bytes: &mut Vec<u8>, frame: &ForestFrame) {
    records(
        bytes,
        frame
            .terrain
            .as_ref()
            .map_or(&[], |m| m.vertices.as_slice()),
    );
    records(
        bytes,
        frame.terrain.as_ref().map_or(&[], |m| m.indices.as_slice()),
    );
    records(
        bytes,
        frame
            .structures
            .as_ref()
            .map_or(&[], |m| m.vertices.as_slice()),
    );
    records(
        bytes,
        frame
            .structures
            .as_ref()
            .map_or(&[], |m| m.indices.as_slice()),
    );
}

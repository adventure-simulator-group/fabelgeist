//! Captured geographic samples shared by all scene representations.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    tag = "kind",
    content = "id",
    deny_unknown_fields
)]
pub enum SceneSource {
    ImportedPackage(String),
    SyntheticFixture(String),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentalSample {
    pub canopy_bps: u16,
    pub wetland_bps: u16,
    pub cultivation_bps: u16,
    pub water_bps: u16,
    pub hilly_bps: u16,
    pub crossing_bps: u16,
    pub surface: TacticalSurface,
}

/// Captured scene transport vocabulary. The dispatcher exhaustively adapts
/// terrain-pack routing cells; tactical consumers use the immutable snapshot
/// without depending on the terrain package runtime or updating routing data.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticalSurface {
    Road,
    #[default]
    Open,
    SparseWoods,
    DeepWoods,
    Water,
    Wetland,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainSampleGrid {
    /// Vertex dimensions; samples are row-major with X varying fastest.
    pub width: u16,
    pub depth: u16,
    pub spacing_metres: f32,
    /// Relative metres around the tactical origin.
    pub heights_metres: Vec<f32>,
    /// One environment sample per height vertex.
    pub environment: Vec<EnvironmentalSample>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VistaLod {
    pub level: u8,
    pub spacing_metres: f32,
    pub width: u16,
    pub depth: u16,
    pub origin_east_metres: f64,
    pub origin_north_metres: f64,
    pub heights_metres: Vec<f32>,
    pub environment: Vec<EnvironmentalSample>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VistaSample {
    pub lods: Vec<VistaLod>,
}

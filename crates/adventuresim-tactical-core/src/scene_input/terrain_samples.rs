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

/// Ordinal of a supplied vista ring. Ring count and ordering are validated by
/// VistaSample's owning scene contract; this does not classify distance roles.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VistaLevelIndex(u8);
impl VistaLevelIndex {
    pub const fn new(index: u8) -> Self {
        Self(index)
    }
    pub const fn index(self) -> u8 {
        self.0
    }
}
impl std::fmt::Display for VistaLevelIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VistaLod {
    pub level: VistaLevelIndex,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vista_ordinal_preserves_numeric_wire_identity_without_assigning_a_role() {
        let level = VistaLevelIndex::new(17);
        assert_eq!(serde_json::to_string(&level).unwrap(), "17");
        assert_eq!(
            serde_json::from_str::<VistaLevelIndex>("17").unwrap(),
            level
        );
        assert!(serde_json::from_str::<VistaLevelIndex>("-1").is_err());
        assert!(serde_json::from_str::<VistaLevelIndex>("256").is_err());
    }
}

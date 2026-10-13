//! Bounded, immutable geographic windows for regional environment presentation.
//! Missing source samples remain explicit holes, never fabricated terrain.
use crate::scene_input::{EnvironmentalSample, SourcePackageDigest};
use adventuresim_world_schema::{ElevationMeters, coordinates::Wgs84CoordinateMicrodegrees};
use serde::{Deserialize, Serialize};

pub const REGIONAL_TERRAIN_SIDE: usize = 65;
pub const REGIONAL_TERRAIN_VERTICES: usize = REGIONAL_TERRAIN_SIDE * REGIONAL_TERRAIN_SIDE;
pub type Result<T> = std::result::Result<T, RegionalTerrainError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionalTerrainRequest {
    pub origin: Wgs84CoordinateMicrodegrees,
    pub scale: RegionalTerrainScale,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionalTerrainVertex {
    /// Absolute source elevation, not the scene-local floor or graded ground.
    pub elevation: ElevationMeters,
    pub environment: EnvironmentalSample,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RegionalTerrainWire", deny_unknown_fields)]
pub struct RegionalTerrain {
    request: RegionalTerrainRequest,
    source: SourcePackageDigest,
    /// Row-major; east varies fastest and north increases with row number.
    vertices: Vec<Option<RegionalTerrainVertex>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalTerrainWire {
    request: RegionalTerrainRequest,
    source: SourcePackageDigest,
    vertices: Vec<Option<RegionalTerrainVertex>>,
}

/// Presentation distances are roles, separate from an ordinal vista-ring index.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionalTerrainScale {
    Neighborhood,
    District,
    Region,
    Country,
    Continent,
}

#[derive(Debug, thiserror::Error)]
pub enum RegionalTerrainError {
    #[error("regional terrain contains {provided} vertices; expected {REGIONAL_TERRAIN_VERTICES}")]
    VertexCount { provided: usize },
}

impl RegionalTerrainScale {
    /// Native numerical-kernel handoff in scene-local east/north metres.
    /// These fixed spacings bound every request to the same vertex count.
    pub const fn spacing_metres(self) -> f32 {
        match self {
            Self::Neighborhood => 30.0,
            Self::District => 250.0,
            Self::Region => 2_000.0,
            Self::Country => 16_000.0,
            Self::Continent => 128_000.0,
        }
    }
}

impl RegionalTerrain {
    pub fn new(
        request: RegionalTerrainRequest,
        source: SourcePackageDigest,
        vertices: Vec<Option<RegionalTerrainVertex>>,
    ) -> Result<Self> {
        if vertices.len() != REGIONAL_TERRAIN_VERTICES {
            return Err(RegionalTerrainError::VertexCount {
                provided: vertices.len(),
            });
        }
        Ok(Self {
            request,
            source,
            vertices,
        })
    }

    pub const fn request(&self) -> RegionalTerrainRequest {
        self.request
    }

    pub fn source(&self) -> &SourcePackageDigest {
        &self.source
    }

    pub fn vertices(&self) -> &[Option<RegionalTerrainVertex>] {
        &self.vertices
    }
}

impl TryFrom<RegionalTerrainWire> for RegionalTerrain {
    type Error = RegionalTerrainError;

    fn try_from(wire: RegionalTerrainWire) -> Result<Self> {
        Self::new(wire.request, wire.source, wire.vertices)
    }
}

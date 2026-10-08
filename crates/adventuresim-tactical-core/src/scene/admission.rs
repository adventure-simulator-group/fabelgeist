//! Decode and native-grid admission precede terrain queries and framework use.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TerrainAdmissionError {
    #[error(
        "terrain needs at least two finite grid vertices on each axis within u32 mesh capacity"
    )]
    Dimensions,
    #[error("terrain grid scale and represented extents must be finite and positive")]
    Scale,
    #[error("terrain coarse stride must divide both cell dimensions")]
    Stride,
    #[error("terrain height samples must be finite")]
    Heights,
}

#[derive(Deserialize)]
pub(super) struct TerrainWire {
    heightmap: Vec<f32>,
    width: usize,
    scale: f32,
    coarse_stride: usize,
    geometry: TerrainGeometry,
}
impl TryFrom<TerrainWire> for SceneTerrain {
    type Error = TerrainAdmissionError;
    fn try_from(wire: TerrainWire) -> Result<Self, Self::Error> {
        let terrain = Self {
            heightmap: wire.heightmap,
            width: wire.width,
            scale: wire.scale,
            coarse_stride: wire.coarse_stride,
            geometry: wire.geometry,
        };
        terrain.validate()?;
        Ok(terrain)
    }
}

pub(super) fn validate_dimensions(
    width: usize,
    depth: usize,
    scale: f32,
    stride: usize,
) -> Result<(), TerrainAdmissionError> {
    if width < 2
        || depth < 2
        || width
            .checked_mul(depth)
            .is_none_or(|n| n > u32::MAX as usize)
        || (width - 1)
            .checked_mul(depth - 1)
            .and_then(|n| n.checked_mul(6))
            .is_none_or(|n| n > u32::MAX as usize)
    {
        return Err(TerrainAdmissionError::Dimensions);
    }
    if !scale.is_finite()
        || scale <= 0.0
        || !((width - 1) as f32 * scale).is_finite()
        || !((depth - 1) as f32 * scale).is_finite()
    {
        return Err(TerrainAdmissionError::Scale);
    }
    if stride == 0 || !(width - 1).is_multiple_of(stride) || !(depth - 1).is_multiple_of(stride) {
        return Err(TerrainAdmissionError::Stride);
    }
    Ok(())
}
impl SceneTerrain {
    pub(super) fn grid_depth_checked(&self) -> Option<usize> {
        (self.width != 0 && self.heightmap.len().is_multiple_of(self.width))
            .then(|| self.heightmap.len() / self.width)
    }
    pub(super) fn validate(&self) -> Result<(), TerrainAdmissionError> {
        let depth = self
            .grid_depth_checked()
            .ok_or(TerrainAdmissionError::Dimensions)?;
        validate_dimensions(self.width, depth, self.scale, self.coarse_stride)?;
        if self.heightmap.iter().any(|h| !h.is_finite()) {
            return Err(TerrainAdmissionError::Heights);
        }
        Ok(())
    }
}
impl Default for SceneTerrain {
    /// A finite flat metre-square grid is the framework default.
    fn default() -> Self {
        Self {
            heightmap: vec![0.0; 4],
            width: 2,
            scale: 1.0,
            coarse_stride: 1,
            geometry: TerrainGeometry::Sampled,
        }
    }
}

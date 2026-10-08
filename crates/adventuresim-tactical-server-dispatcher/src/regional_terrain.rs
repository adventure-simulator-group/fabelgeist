//! Read-only bounded terrain capture, independent of settlements and simulation.
use crate::terrain_sampling::environment_sample;
use adventuresim_tactical_core::{
    regional_terrain::{
        REGIONAL_TERRAIN_SIDE, REGIONAL_TERRAIN_VERTICES, RegionalTerrain, RegionalTerrainError,
        RegionalTerrainRequest, RegionalTerrainVertex,
    },
    scene_input::{SceneValidationError, SourcePackageDigest},
};
use adventuresim_terrain::TerrainPack;
use adventuresim_world_schema::ElevationMeters;
use adventuresim_world_schema::coordinates::terrain_projection::NativeTerrainCoordinate;

pub type Result<T> = std::result::Result<T, RegionalTerrainCaptureError>;

#[derive(Debug, thiserror::Error)]
pub enum RegionalTerrainCaptureError {
    #[error(transparent)]
    Terrain(#[from] adventuresim_terrain::Error),
    #[error(transparent)]
    Admission(#[from] RegionalTerrainError),
    #[error(transparent)]
    Source(#[from] SceneValidationError),
    #[error("terrain source elevation {metres} is outside the supported geographic range")]
    Elevation { metres: i16 },
}

pub fn capture(pack: &TerrainPack, request: RegionalTerrainRequest) -> Result<RegionalTerrain> {
    let source = SourcePackageDigest::from_hex(pack.digest())?;
    let center = (REGIONAL_TERRAIN_SIDE - 1) as f64 * 0.5;
    let spacing = f64::from(request.scale.spacing_metres());
    let origin = NativeTerrainCoordinate::from(request.origin.to_e7());
    let mut vertices = Vec::with_capacity(REGIONAL_TERRAIN_VERTICES);
    for row in 0..REGIONAL_TERRAIN_SIDE {
        for column in 0..REGIONAL_TERRAIN_SIDE {
            // TerrainPack's native numerical kernel takes continuous degrees and
            // scene-local east/north metres; no intermediate coordinate rounding.
            let point = origin.at_offset(
                (column as f64 - center) * spacing,
                (row as f64 - center) * spacing,
            );
            let vertex = pack
                .cell(point.latitude_degrees, point.longitude_degrees)?
                .map(|cell| -> Result<RegionalTerrainVertex> {
                    Ok(RegionalTerrainVertex {
                        elevation: ElevationMeters::new(cell.elevation_m).ok_or(
                            RegionalTerrainCaptureError::Elevation {
                                metres: cell.elevation_m,
                            },
                        )?,
                        environment: environment_sample(cell),
                    })
                })
                .transpose()?;
            vertices.push(vertex);
        }
    }
    Ok(RegionalTerrain::new(request, source, vertices)?)
}

//! Reproducible geographic input stages for offline support diagnostics.
use super::*;
use adventuresim_tactical_core::scene_coordinates::ScenePlanPoint;
use adventuresim_world_schema::ElevationMeters;

#[derive(Debug, thiserror::Error)]
pub enum TerrainCaptureError {
    #[error(transparent)]
    Geometry(#[from] adventuresim_building_generator::spatial_geometry::GeometryError),
    #[error(transparent)]
    Source(#[from] adventuresim_tactical_core::scene_input::SceneValidationError),
    #[error(transparent)]
    Terrain(#[from] adventuresim_terrain::Error),
    #[error("source terrain is absent at scene point {point:?}")]
    OutsideSource { point: ScenePlanPoint },
    #[error(
        "absolute elevation {metres} m at scene point {point:?} is outside the world elevation contract"
    )]
    InvalidElevation { point: ScenePlanPoint, metres: i16 },
    #[error("vista resampling: {0}")]
    VistaSampling(String),
}

#[derive(Clone, Copy, Debug)]
pub struct SourceElevationSample {
    pub scene_point: ScenePlanPoint,
    pub absolute_elevation: ElevationMeters,
}
impl SourceElevationSample {
    fn from_cell(point: ScenePlanPoint, cell: Cell) -> Result<Self, TerrainCaptureError> {
        Ok(Self {
            scene_point: point,
            absolute_elevation: ElevationMeters::new(cell.elevation_m).ok_or(
                TerrainCaptureError::InvalidElevation {
                    point,
                    metres: cell.elevation_m,
                },
            )?,
        })
    }
}
// Capture JSON has named fields and scalar metres at its explicit wire boundary.
impl serde::Serialize for SourceElevationSample {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(serde::Serialize)]
        struct WireSample {
            scene_point: ScenePlanPoint,
            absolute_elevation: i16,
        }
        WireSample {
            scene_point: self.scene_point,
            absolute_elevation: self.absolute_elevation.get(),
        }
        .serialize(serializer)
    }
}

pub struct ImportedTerrainCapture {
    pub source_digest: adventuresim_tactical_core::scene_input::SourcePackageDigest,
    pub absolute_elevation: ElevationMeters,
    pub source_transects: Vec<SourceElevationSample>,
    pub ungraded_vista: VistaSample,
}

impl ImportedTerrainCapture {
    /// Observe source cells under exact scene-coordinate property probes.
    pub fn sample_points(
        pack: &TerrainPack,
        coordinates: Wgs84CoordinateE7,
        points: impl IntoIterator<Item = ScenePlanPoint>,
    ) -> Result<Vec<SourceElevationSample>, TerrainCaptureError> {
        points
            .into_iter()
            .map(|point| {
                let GeographicSampleCoordinate {
                    latitude_degrees: lat,
                    longitude_degrees: lon,
                } = offset_coordinate(
                    coordinates.latitude().degrees(),
                    coordinates.longitude().degrees(),
                    f64::from(point.metres().x),
                    f64::from(point.metres().y),
                );
                let cell = pack
                    .cell(lat, lon)?
                    .ok_or(TerrainCaptureError::OutsideSource { point })?;
                SourceElevationSample::from_cell(point, cell)
            })
            .collect()
    }

    /// Capture the production resampler and fixed east/north hill transects.
    /// The caller supplies the exact scene seed, coordinate and source package.
    pub fn sample(
        pack: &TerrainPack,
        coordinates: Wgs84CoordinateE7,
        seed: fabelgeist_determinism::Seed,
    ) -> Result<Self, TerrainCaptureError> {
        const TRANSECT_HALF_LENGTH_METRES: i32 = 3_000;
        const TRANSECT_SPACING_METRES: usize = 25;
        let latitude = coordinates.latitude().degrees();
        let longitude = coordinates.longitude().degrees();
        let centre = pack
            .cell(latitude, longitude)?
            .ok_or(TerrainCaptureError::OutsideSource {
                point: ScenePlanPoint::ORIGIN,
            })?;
        let mut source_transects = Vec::new();
        for axis in [Vec2::X, Vec2::Y] {
            for distance in (-TRANSECT_HALF_LENGTH_METRES..=TRANSECT_HALF_LENGTH_METRES)
                .step_by(TRANSECT_SPACING_METRES)
            {
                let point = ScenePlanPoint::try_from(axis * distance as f32)?;
                let GeographicSampleCoordinate {
                    latitude_degrees: lat,
                    longitude_degrees: lon,
                } = offset_coordinate(
                    latitude,
                    longitude,
                    f64::from(point.metres().x),
                    f64::from(point.metres().y),
                );
                let cell = pack
                    .cell(lat, lon)?
                    .ok_or(TerrainCaptureError::OutsideSource { point })?;
                source_transects.push(SourceElevationSample::from_cell(point, cell)?);
            }
        }
        Ok(Self {
            source_digest: adventuresim_tactical_core::scene_input::SourcePackageDigest::from_hex(
                pack.digest(),
            )?,
            absolute_elevation: SourceElevationSample::from_cell(ScenePlanPoint::ORIGIN, centre)?
                .absolute_elevation,
            source_transects,
            ungraded_vista: sample_city_vista(
                pack,
                coordinates,
                SourceElevationSample::from_cell(ScenePlanPoint::ORIGIN, centre)?
                    .absolute_elevation,
                seed,
            )
            .map_err(TerrainCaptureError::VistaSampling)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_samples_keep_scene_position_absolute_height_and_typed_rejections() {
        let point = ScenePlanPoint::from_metres(Vec2::new(-25.0, 50.0)).unwrap();
        let cell = Cell {
            elevation_m: 321,
            ..Default::default()
        };
        let sample = SourceElevationSample::from_cell(point, cell).unwrap();
        let wire = serde_json::to_value(sample).unwrap();
        assert_eq!(wire["scene_point"], serde_json::json!([-25.0, 50.0]));
        assert_eq!(wire["absolute_elevation"], 321);
        let bad = Cell {
            elevation_m: 9_001,
            ..Default::default()
        };
        assert!(matches!(SourceElevationSample::from_cell(point, bad),
            Err(TerrainCaptureError::InvalidElevation { point: rejected, metres: 9_001 })
            if rejected == point));
    }
}

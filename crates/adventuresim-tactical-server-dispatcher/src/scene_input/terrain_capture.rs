//! Reproducible geographic input stages for offline support diagnostics.
use super::*;

pub struct ImportedTerrainCapture {
    pub source_digest: String,
    pub absolute_elevation_metres: i16,
    pub source_transects: Vec<(Vec2, i16)>,
    pub ungraded_vista: VistaSample,
}

impl ImportedTerrainCapture {
    /// Observe source cells under exact scene-coordinate property probes.
    pub fn sample_points(
        pack: &TerrainPack,
        coordinates: Wgs84CoordinateE7,
        points: impl IntoIterator<Item = Vec2>,
    ) -> Result<Vec<(Vec2, i16)>, String> {
        points
            .into_iter()
            .map(|point| {
                let (lat, lon) = offset_coordinate(
                    coordinates.latitude().degrees(),
                    coordinates.longitude().degrees(),
                    f64::from(point.x),
                    f64::from(point.y),
                );
                let cell = pack
                    .cell(lat, lon)
                    .map_err(|error| error.to_string())?
                    .ok_or("property probe leaves the source terrain")?;
                Ok((point, cell.elevation_m))
            })
            .collect()
    }

    /// Capture the production resampler and fixed east/north hill transects.
    /// The caller supplies the exact scene seed, coordinate and source package.
    pub fn sample(
        pack: &TerrainPack,
        coordinates: Wgs84CoordinateE7,
        seed: u64,
    ) -> Result<Self, String> {
        const TRANSECT_HALF_LENGTH_METRES: i32 = 3_000;
        const TRANSECT_SPACING_METRES: usize = 25;
        let latitude = coordinates.latitude().degrees();
        let longitude = coordinates.longitude().degrees();
        let centre = pack
            .cell(latitude, longitude)
            .map_err(|error| error.to_string())?
            .ok_or("transect centre leaves the source terrain")?;
        let mut source_transects = Vec::new();
        for axis in [Vec2::X, Vec2::Y] {
            for distance in (-TRANSECT_HALF_LENGTH_METRES..=TRANSECT_HALF_LENGTH_METRES)
                .step_by(TRANSECT_SPACING_METRES)
            {
                let point = axis * distance as f32;
                let (lat, lon) =
                    offset_coordinate(latitude, longitude, f64::from(point.x), f64::from(point.y));
                let cell = pack
                    .cell(lat, lon)
                    .map_err(|error| error.to_string())?
                    .ok_or("transect leaves the source terrain")?;
                source_transects.push((point, cell.elevation_m));
            }
        }
        Ok(Self {
            source_digest: pack.digest().into(),
            absolute_elevation_metres: centre.elevation_m,
            source_transects,
            ungraded_vista: sample_city_vista(
                pack,
                coordinates,
                f32::from(centre.elevation_m),
                seed,
            )?,
        })
    }
}

//! The native terrain sampler's shared local east/north projection.
//! Checked geographic origins enter this numerical port once. Intermediate
//! samples retain continuous degrees, including points outside source coverage.
use super::Wgs84CoordinateE7;

pub const METRES_PER_LATITUDE_DEGREE: f64 = 111_320.0;
pub const MIN_LONGITUDE_SCALE: f64 = 0.01;

/// Native TerrainPack coordinates, not a checked geographic position. Offsets
/// can leave WGS84 or package bounds; the source lookup owns missing coverage.
#[derive(Clone, Copy, Debug)]
pub struct NativeTerrainCoordinate {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
}

/// Numerical-kernel output for a mesh or camera adapter, in local east/north
/// metres. Logical scene positions retain their existing checked frame types.
#[derive(Clone, Copy, Debug)]
pub struct NativeTerrainOffset {
    pub east_metres: f64,
    pub north_metres: f64,
}

impl From<Wgs84CoordinateE7> for NativeTerrainCoordinate {
    fn from(origin: Wgs84CoordinateE7) -> Self {
        Self {
            latitude_degrees: origin.latitude().degrees(),
            longitude_degrees: origin.longitude().degrees(),
        }
    }
}

impl NativeTerrainCoordinate {
    /// Terrain sampling kernel: signed east/north metres become continuous
    /// degrees. The operation order matches imported tactical terrain capture.
    pub fn at_offset(self, east_metres: f64, north_metres: f64) -> Self {
        let latitude_delta = north_metres / METRES_PER_LATITUDE_DEGREE;
        let longitude_scale = self
            .latitude_degrees
            .to_radians()
            .cos()
            .abs()
            .max(MIN_LONGITUDE_SCALE);
        let longitude_delta = east_metres / (METRES_PER_LATITUDE_DEGREE * longitude_scale);
        Self {
            latitude_degrees: self.latitude_degrees + latitude_delta,
            longitude_degrees: self.longitude_degrees + longitude_delta,
        }
    }

    /// Inverse local projection at this sampler origin. This does not choose a
    /// new origin or round a geographic position during mesh/camera arithmetic.
    pub fn offset_to(self, target: Wgs84CoordinateE7) -> NativeTerrainOffset {
        let longitude_scale = self
            .latitude_degrees
            .to_radians()
            .cos()
            .abs()
            .max(MIN_LONGITUDE_SCALE);
        NativeTerrainOffset {
            east_metres: (target.longitude().degrees() - self.longitude_degrees)
                * (METRES_PER_LATITUDE_DEGREE * longitude_scale),
            north_metres: (target.latitude().degrees() - self.latitude_degrees)
                * METRES_PER_LATITUDE_DEGREE,
        }
    }
}

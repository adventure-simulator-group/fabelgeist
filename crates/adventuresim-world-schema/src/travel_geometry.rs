//! Canonical offline edge geometry.
use crate::coordinates;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidTravelCoordinate;
impl std::fmt::Display for InvalidTravelCoordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid travel geometry coordinate")
    }
}
impl std::error::Error for InvalidTravelCoordinate {}

/// A bounded, deterministic WGS84 point in canonical offline edge geometry.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(crate = spacetimedb_lib))]
pub struct TravelGeometryPoint {
    pub longitude_e7: i32,
    pub latitude_e7: i32,
}

impl TravelGeometryPoint {
    pub fn new(longitude: f64, latitude: f64) -> Result<Self, InvalidTravelCoordinate> {
        let longitude =
            coordinates::LongitudeE7::from_degrees(longitude).ok_or(InvalidTravelCoordinate)?;
        let latitude =
            coordinates::LatitudeE7::from_degrees(latitude).ok_or(InvalidTravelCoordinate)?;
        Ok(Self {
            longitude_e7: longitude.get(),
            latitude_e7: latitude.get(),
        })
    }

    pub fn longitude(self) -> f64 {
        coordinates::LongitudeE7::new(self.longitude_e7)
            .expect("travel geometry longitude was validated at construction")
            .degrees()
    }
    pub fn latitude(self) -> f64 {
        coordinates::LatitudeE7::new(self.latitude_e7)
            .expect("travel geometry latitude was validated at construction")
            .degrees()
    }
}

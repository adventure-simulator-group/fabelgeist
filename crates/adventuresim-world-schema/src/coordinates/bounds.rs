//! Checked geographic rectangles shared by source and regional presentation.
use super::Wgs84CoordinateE7;

/// Inclusive WGS84 rectangle with checked E7 corners. Antimeridian-crossing
/// rectangles are outside the regional source contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Wgs84BoundsE7 {
    southwest: Wgs84CoordinateE7,
    northeast: Wgs84CoordinateE7,
}

impl Wgs84BoundsE7 {
    pub const fn new(southwest: Wgs84CoordinateE7, northeast: Wgs84CoordinateE7) -> Option<Self> {
        if southwest.latitude().get() > northeast.latitude().get()
            || southwest.longitude().get() > northeast.longitude().get()
        {
            return None;
        }
        Some(Self {
            southwest,
            northeast,
        })
    }

    /// Native source bounds enter in west/south/east/north WGS84 degrees.
    pub fn from_longitude_latitude_degrees([west, south, east, north]: [f64; 4]) -> Option<Self> {
        Self::new(
            Wgs84CoordinateE7::from_longitude_latitude_degrees(west, south)?,
            Wgs84CoordinateE7::from_longitude_latitude_degrees(east, north)?,
        )
    }

    /// Native numerical-kernel adapter in west/south/east/north degrees.
    pub fn longitude_latitude_degrees(self) -> [f64; 4] {
        [
            self.southwest.longitude().degrees(),
            self.southwest.latitude().degrees(),
            self.northeast.longitude().degrees(),
            self.northeast.latitude().degrees(),
        ]
    }

    pub fn intersects(self, other: Self) -> bool {
        self.southwest.longitude() <= other.northeast.longitude()
            && self.northeast.longitude() >= other.southwest.longitude()
            && self.southwest.latitude() <= other.northeast.latitude()
            && self.northeast.latitude() >= other.southwest.latitude()
    }
}

use super::{CoordinateAxis, GeometryError, GeometryFrame, GeometryRole, Position, SpatialBounds};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// A positive clearance volume in metres, including arbitrarily thin admitted
/// sections. General collision/bearing bounds intentionally permit degeneracy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[serde(transparent, bound = "")]
#[repr(transparent)]
#[reflect(opaque)]
pub struct ClearanceVolume<F: GeometryFrame>(SpatialBounds<F>);
impl<F: GeometryFrame> ClearanceVolume<F> {
    pub fn new(bounds: SpatialBounds<F>) -> Result<Self, GeometryError> {
        let extent = bounds.extent()?.metres();
        for (axis, value) in [CoordinateAxis::X, CoordinateAxis::Y, CoordinateAxis::Z]
            .into_iter()
            .zip(extent.to_array())
        {
            if value == 0.0 {
                return Err(GeometryError::ZeroExtent {
                    role: GeometryRole::ClearanceVolume,
                    axis,
                });
            }
        }
        Ok(Self(bounds))
    }
    pub fn from_metres(
        min: bevy::math::Vec3,
        max: bevy::math::Vec3,
    ) -> Result<Self, GeometryError> {
        Self::new(SpatialBounds::from_metres(min, max)?)
    }
    pub fn bounds(self) -> SpatialBounds<F> {
        self.0
    }
    pub fn min(self) -> Position<F> {
        self.0.min()
    }
    pub fn max(self) -> Position<F> {
        self.0.max()
    }
}
impl<'de, F: GeometryFrame> Deserialize<'de> for ClearanceVolume<F> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(SpatialBounds::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

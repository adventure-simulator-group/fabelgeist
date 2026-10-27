use super::{CoordinateAxis, CuboidDimensions, GeometryError, GeometryFrame, Position};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Finite ordered bounds in one metre frame. Zero extents are permitted.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[serde(bound = "")]
#[reflect(opaque)]
pub struct SpatialBounds<F: GeometryFrame> {
    min: Position<F>,
    max: Position<F>,
}
impl<F: GeometryFrame> SpatialBounds<F> {
    /// Checked admission at native-vector production boundaries.
    pub fn from_metres(
        min: bevy::math::Vec3,
        max: bevy::math::Vec3,
    ) -> Result<Self, GeometryError> {
        Self::new(Position::from_metres(min)?, Position::from_metres(max)?)
    }
    pub fn new(min: Position<F>, max: Position<F>) -> Result<Self, GeometryError> {
        for (axis, (lower, upper)) in [CoordinateAxis::X, CoordinateAxis::Y, CoordinateAxis::Z]
            .into_iter()
            .zip(
                min.metres()
                    .to_array()
                    .into_iter()
                    .zip(max.metres().to_array()),
            )
        {
            if lower > upper {
                return Err(GeometryError::ReversedBounds { axis });
            }
        }
        Ok(Self { min, max })
    }
    pub fn at(point: Position<F>) -> Self {
        Self {
            min: point,
            max: point,
        }
    }
    pub fn min(self) -> Position<F> {
        self.min
    }
    pub fn max(self) -> Position<F> {
        self.max
    }
    /// Original f32 midpoint arithmetic; overflow is a construction error.
    pub fn centre(self) -> Result<Position<F>, GeometryError> {
        let metres = (self.min.metres() + self.max.metres()) * 0.5;
        super::error::finite(metres, super::GeometryRole::BoundsCentre)?;
        Position::from_metres(metres)
    }
    pub fn extent(self) -> Result<CuboidDimensions, GeometryError> {
        let metres = self.max.metres() - self.min.metres();
        super::error::finite(metres, super::GeometryRole::BoundsExtent)?;
        CuboidDimensions::from_metres(metres)
    }
    pub fn plan_half_extents(self) -> Result<super::PlanExtents, GeometryError> {
        let min = self.min.metres();
        let max = self.max.metres();
        super::PlanExtents::from_metres(bevy::math::Vec2::new(max.x - min.x, max.z - min.z) * 0.5)
    }
    /// Inclusive overlap, including face, edge and point tangency.
    pub fn overlaps(self, other: Self) -> bool {
        self.min.metres().cmple(other.max.metres()).all()
            && other.min.metres().cmple(self.max.metres()).all()
    }
    pub fn union(self, other: Self) -> Self {
        // min/max select already-admitted coordinates without arithmetic.
        Self {
            min: Position {
                metres: self.min.metres().min(other.min.metres()),
                frame: std::marker::PhantomData,
            },
            max: Position {
                metres: self.max.metres().max(other.max.metres()),
                frame: std::marker::PhantomData,
            },
        }
    }
    pub fn including(self, point: Position<F>) -> Self {
        self.union(Self::at(point))
    }
}
impl<'de, F: GeometryFrame> Deserialize<'de> for SpatialBounds<F> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(bound = "")]
        struct Bounds<F: GeometryFrame> {
            min: Position<F>,
            max: Position<F>,
        }
        let value = Bounds::deserialize(deserializer)?;
        Self::new(value.min, value.max).map_err(serde::de::Error::custom)
    }
}

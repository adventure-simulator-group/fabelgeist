use super::{GeometryError, GeometryRole, error::finite};
use bevy::{
    math::Vec3,
    reflect::{Reflect, TypePath},
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Implemented by the module owning a coordinate frame, never by native vectors.
pub trait GeometryFrame:
    Copy + std::fmt::Debug + PartialEq + TypePath + Send + Sync + 'static
{
}

/// Building-local X/Y/Z, in metres, with architectural ground at Y=0.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
pub enum Architectural {}
impl GeometryFrame for Architectural {}

/// A finite point in one declared frame; it is not a translation or dimensions.
///
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::{Architectural, CuboidDimensions, Position};
/// let point: Position<Architectural> = Position::ORIGIN;
/// let dimensions: CuboidDimensions = point;
/// ```
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::{Architectural, Displacement, SpatialDirection};
/// let displacement: Displacement<Architectural> = Displacement::ZERO;
/// let direction: SpatialDirection<Architectural> = displacement;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent, bound = "")]
#[reflect(opaque)]
pub struct Position<F: GeometryFrame> {
    pub(super) metres: Vec3,
    #[serde(skip)]
    pub(super) frame: PhantomData<F>,
}

impl<F: GeometryFrame> Position<F> {
    pub const ORIGIN: Self = Self {
        metres: Vec3::ZERO,
        frame: PhantomData,
    };
    pub fn from_metres(metres: Vec3) -> Result<Self, GeometryError> {
        finite(metres, GeometryRole::Position)?;
        Ok(Self {
            metres,
            frame: PhantomData,
        })
    }
    /// Explicit native representation for owned arithmetic and framework ports.
    pub fn metres(self) -> Vec3 {
        self.metres
    }
    pub fn translated(self, displacement: Displacement<F>) -> Result<Self, GeometryError> {
        Self::from_metres(self.metres + displacement.metres)
    }
    pub fn displacement_to(self, other: Self) -> Result<Displacement<F>, GeometryError> {
        Displacement::from_metres(other.metres - self.metres)
    }
}
impl<'de, F: GeometryFrame> Deserialize<'de> for Position<F> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_metres(Vec3::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// A finite signed translation. Zero is valid and does not need a direction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent, bound = "")]
#[reflect(opaque)]
pub struct Displacement<F: GeometryFrame> {
    pub(super) metres: Vec3,
    #[serde(skip)]
    pub(super) frame: PhantomData<F>,
}
impl<F: GeometryFrame> Displacement<F> {
    pub const ZERO: Self = Self {
        metres: Vec3::ZERO,
        frame: PhantomData,
    };
    pub fn from_metres(metres: Vec3) -> Result<Self, GeometryError> {
        finite(metres, GeometryRole::Displacement)?;
        Ok(Self {
            metres,
            frame: PhantomData,
        })
    }
    pub fn metres(self) -> Vec3 {
        self.metres
    }
}
impl<'de, F: GeometryFrame> Deserialize<'de> for Displacement<F> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_metres(Vec3::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Signed finite elevation from the declared frame's datum, in metres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent, bound = "")]
#[reflect(opaque)]
pub struct Elevation<F: GeometryFrame> {
    metres: f32,
    #[serde(skip)]
    pub(super) frame: PhantomData<F>,
}
impl<F: GeometryFrame> Elevation<F> {
    pub const ZERO: Self = Self {
        metres: 0.0,
        frame: PhantomData,
    };
    pub fn from_metres(metres: f32) -> Result<Self, GeometryError> {
        finite(Vec3::new(0.0, metres, 0.0), GeometryRole::Elevation)?;
        Ok(Self {
            metres,
            frame: PhantomData,
        })
    }
    pub fn metres(self) -> f32 {
        self.metres
    }
}
impl<'de, F: GeometryFrame> Deserialize<'de> for Elevation<F> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

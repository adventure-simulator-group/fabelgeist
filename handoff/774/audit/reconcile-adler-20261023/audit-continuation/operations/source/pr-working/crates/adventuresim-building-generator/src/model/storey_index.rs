//! Ordinal into the occupied storeys of a building programme.
//!
//! ```compile_fail
//! use adventuresim_building_generator::{RoomIndex, StoreyIndex};
//! let storey = StoreyIndex::GROUND;
//! let room: RoomIndex = storey;
//! ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, bevy::reflect::Reflect)]
#[reflect(opaque)]
pub struct StoreyIndex(usize);
impl StoreyIndex {
    pub const GROUND: Self = Self(0);
    pub const FIRST_UPPER: Self = Self(1);
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
    pub const fn from_serialized(ordinal: u16) -> Self {
        Self(ordinal as usize)
    }
    pub fn serialized_ordinal(self) -> Result<u16, OrdinalError> {
        u16::try_from(self.0).map_err(|cause| OrdinalError {
            role: RecipeOrdinalRole::Storey,
            ordinal: self.0,
            cause,
        })
    }
    pub const fn index(self) -> usize {
        self.0
    }

    /// Classify an occupied elevation with the existing rounded station
    /// arithmetic. Admission rejects saturation of the packed storey ordinal.
    pub fn from_elevation(
        elevation: crate::spatial_geometry::Elevation<crate::Architectural>,
        storey_height: crate::spatial_geometry::PositiveLength,
    ) -> Result<Self, StoreyElevationError> {
        let ordinal = (elevation.metres() / storey_height.metres()).round();
        if !ordinal.is_finite() {
            return Err(StoreyElevationError::Overflow);
        }
        if ordinal < 0.0 {
            return Err(StoreyElevationError::BelowGround);
        }
        if ordinal > f32::from(u16::MAX) {
            return Err(StoreyElevationError::ExceedsRepresentation);
        }
        Ok(Self::new(ordinal as usize))
    }
}
impl serde::Serialize for StoreyIndex {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.serialized_ordinal()
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }
}
impl<'de> serde::Deserialize<'de> for StoreyIndex {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        u16::deserialize(d).map(Self::from_serialized)
    }
}

#[derive(Clone, Copy, Debug, thiserror::Error, Eq, PartialEq)]
pub enum StoreyElevationError {
    #[error("elevation divided by storey height overflowed")]
    Overflow,
    #[error("rounded occupied storey lies below ground")]
    BelowGround,
    #[error("rounded occupied storey exceeds its packed u16 representation")]
    ExceedsRepresentation,
}

impl std::fmt::Display for StoreyIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeOrdinalRole {
    Storey,
    Room,
}
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("{role:?} ordinal {ordinal} does not fit its packed representation: {cause}")]
pub struct OrdinalError {
    pub role: RecipeOrdinalRole,
    pub ordinal: usize,
    #[source]
    pub cause: std::num::TryFromIntError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_geometry::{Elevation, PositiveLength};
    #[test]
    fn storey_admission_preserves_packed_ordinals_without_saturating() {
        for native in [0, 1, u16::MAX] {
            let storey = StoreyIndex::from_serialized(native);
            assert_eq!(
                postcard::to_allocvec(&storey).unwrap(),
                postcard::to_allocvec(&native).unwrap()
            );
            assert_eq!(
                postcard::from_bytes::<StoreyIndex>(&postcard::to_allocvec(&native).unwrap())
                    .unwrap(),
                storey
            );
        }
        let oversized = StoreyIndex::new(usize::from(u16::MAX) + 1);
        assert!(oversized.serialized_ordinal().is_err());
        assert!(postcard::to_allocvec(&oversized).is_err());
        let height = PositiveLength::from_metres(3.36).unwrap();
        assert_eq!(
            StoreyIndex::from_elevation(Elevation::from_metres(3.36).unwrap(), height).unwrap(),
            StoreyIndex::FIRST_UPPER
        );
        assert_eq!(
            StoreyIndex::from_elevation(Elevation::from_metres(-3.36).unwrap(), height)
                .unwrap_err(),
            StoreyElevationError::BelowGround
        );
        assert_eq!(
            StoreyIndex::from_elevation(Elevation::from_metres(1_000_000.0).unwrap(), height)
                .unwrap_err(),
            StoreyElevationError::ExceedsRepresentation
        );
        assert_eq!(
            StoreyIndex::from_elevation(
                Elevation::from_metres(f32::MAX).unwrap(),
                PositiveLength::from_metres(f32::MIN_POSITIVE).unwrap()
            )
            .unwrap_err(),
            StoreyElevationError::Overflow
        );
    }
}

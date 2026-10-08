//! Collar engineering leaves retain the existing zero and variation bounds.
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct RuptureWander(f32);
impl RuptureWander {
    pub fn from_metres(value: f32) -> Option<Self> {
        (value.is_finite() && value >= 0.0).then_some(Self(value))
    }
    pub fn metres(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for RuptureWander {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(d)?).ok_or_else(|| {
            serde::de::Error::custom("rupture wander must be finite nonnegative metres")
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Reflect)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct CollarWidthVariation(u16);
impl CollarWidthVariation {
    pub fn from_basis_points(value: u16) -> Option<Self> {
        (value <= 5_000).then_some(Self(value))
    }
    pub fn basis_points(self) -> u16 {
        self.0
    }
}
impl<'de> Deserialize<'de> for CollarWidthVariation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_basis_points(u16::deserialize(d)?).ok_or_else(|| {
            serde::de::Error::custom("collar width variation exceeds 5000 basis points")
        })
    }
}

//! Shared numerical units at application and mathematical-kernel boundaries.
//!
//! These types distinguish units without adding numerical policy. Their
//! scalar getters are for actual arithmetic, serialization and native APIs;
//! producers and consumers otherwise retain the unit. JSON representations
//! remain the same scalar or fixed array as before.

use serde::{Deserialize, Serialize};

/// A signed distance in metres.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Metres(f32);

/// A dimensionless ratio; application records own policy bounds.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Ratio(f32);

/// Angular change per metre of a curve.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RadiansPerMetre(f32);

/// Complete turns per metre of a curve.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TurnsPerMetre(f32);

/// Spatial frequency in inverse metres.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PerMetre(f32);

/// Acceleration in metres per second squared.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Acceleration(f32);

/// Acceleration components in the model coordinate frame, in metres per second squared.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccelerationVector([f32; 3]);

impl Metres {
    pub const fn from_metres(value: f32) -> Self {
        Self(value)
    }
    /// Native representation for the numerical or external boundary.
    pub const fn metres(self) -> f32 {
        self.0
    }
}

impl Ratio {
    pub const fn from_ratio(value: f32) -> Self {
        Self(value)
    }
    /// Native representation for the numerical or external boundary.
    pub const fn ratio(self) -> f32 {
        self.0
    }
}

impl RadiansPerMetre {
    pub const fn from_radians_per_metre(value: f32) -> Self {
        Self(value)
    }
    /// Native representation for the numerical or external boundary.
    pub const fn radians_per_metre(self) -> f32 {
        self.0
    }
}

impl TurnsPerMetre {
    pub const fn from_turns_per_metre(value: f32) -> Self {
        Self(value)
    }
    /// Native representation for the numerical or external boundary.
    pub const fn turns_per_metre(self) -> f32 {
        self.0
    }
}

impl PerMetre {
    pub const fn from_per_metre(value: f32) -> Self {
        Self(value)
    }
    /// Native representation for the numerical or external boundary.
    pub const fn per_metre(self) -> f32 {
        self.0
    }
}

impl Acceleration {
    pub const fn from_metres_per_second_squared(value: f32) -> Self {
        Self(value)
    }
    /// Native representation for the numerical or external boundary.
    pub const fn metres_per_second_squared(self) -> f32 {
        self.0
    }
}

impl AccelerationVector {
    pub const fn from_components(value: [f32; 3]) -> Self {
        Self(value)
    }
    /// Native representation for the numerical or external boundary.
    pub const fn components(self) -> [f32; 3] {
        self.0
    }
}

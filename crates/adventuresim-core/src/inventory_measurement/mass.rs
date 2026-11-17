//! Checked nonnegative item and inventory mass in kilograms.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct InventoryMassKg(f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryMassError {
    #[error("inventory mass must be finite")]
    NonFinite,
    #[error("inventory mass must not be negative")]
    Negative,
}

impl InventoryMassKg {
    pub const ZERO: Self = Self(0.0);

    pub const fn kilograms(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for InventoryMassKg {
    type Error = InventoryMassError;

    fn try_from(kilograms: f32) -> Result<Self, Self::Error> {
        if !kilograms.is_finite() {
            return Err(InventoryMassError::NonFinite);
        }
        if kilograms < 0.0 {
            return Err(InventoryMassError::Negative);
        }
        Ok(Self(kilograms))
    }
}

impl From<InventoryMassKg> for f32 {
    fn from(mass: InventoryMassKg) -> Self {
        mass.kilograms()
    }
}

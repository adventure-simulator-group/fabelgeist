//! A fauld and its suspended tassets form one equipable waist defense.
use crate::{DesignError, GarmentArmorDesign, GarmentArmorKind};
use serde::{Deserialize, Serialize};

pub const TASSET_SUSPENSION_GAP_M: f32 = 0.005;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaistArmorDesign {
    pub fauld: GarmentArmorDesign,
    pub tassets: GarmentArmorDesign,
}

impl WaistArmorDesign {
    pub fn validate(&self) -> Result<(), crate::GenerateError> {
        if self.fauld.kind != GarmentArmorKind::Fauld
            || self.tassets.kind != GarmentArmorKind::Tassets
        {
            return Err(DesignError::ParametricParameters.into());
        }
        self.fauld.validate()?;
        self.tassets.validate()
    }
}

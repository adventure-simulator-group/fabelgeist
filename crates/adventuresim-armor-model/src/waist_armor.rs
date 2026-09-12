//! A fauld and its suspended tassets form one equipable waist defense.
use crate::{ArmorComponentRole, DesignError, GarmentArmorDesign, GarmentArmorKind, PartMesh};
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

/// Seat a horizontal suspension edge below the lowest fauld hem. Shaped
/// suspension edges are seated by their anatomical fitter instead.
pub fn suspend_horizontal_tassets(fauld: &PartMesh, mut tassets: PartMesh) -> PartMesh {
    let hem = fauld
        .positions
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);
    let top = tassets
        .positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let shift = (hem - TASSET_SUSPENSION_GAP_M - top).min(0.0);
    if shift < 0.0 {
        tassets = tassets.transformed(&crate::PartFrame {
            origin: [0.0, shift, 0.0],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [1.0; 3],
        });
    }
    tassets
}

/// Preserve fitted positions and distinct plate identities through export.
pub fn compose_waist(fauld: PartMesh, tassets: PartMesh) -> PartMesh {
    let mut mesh = fauld.with_component(ArmorComponentRole::Fauld, None);
    mesh.append(tassets.with_component(ArmorComponentRole::Tassets, None));
    mesh
}

//! Frozen anatomical cloth carriers and separately seated sewn patches.
//! Carrier positions are the innermost cloth face; material gauges add outward.
use crate::armor_frames::Wearer;
use crate::skin_transfer::SkinWeights;
use anyhow::Result;
use bevy::math::Vec3;

#[path = "cloth_panel_anatomical.rs"]
mod anatomical;
#[path = "cloth_panel_banks.rs"]
pub(crate) mod banks;
#[path = "cloth_panel_domain.rs"]
mod domain;
#[path = "cloth_panel_enclosure.rs"]
mod enclosure;
#[path = "cloth_panel_fit_plan.rs"]
mod fit_plan;
#[path = "cloth_panel_normals.rs"]
mod normal_field;
pub use fit_plan::PanelPlan;

/// Minimum authored resting separation, matching the bodice clearance range.
/// Canonical metres at the native fitting boundary. Sampled contacts and
/// winding remain checked; final intersections require an actual mesh audit.
pub(crate) const MINIMUM_CLOTH_CLEARANCE_M: f32 = 0.001;

pub struct PanelSurface {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    skin: SkinWeights,
}

impl PanelSurface {
    /// Preserve the already fitted source without applying another body offset.
    pub(super) fn from_fitted_carrier(
        positions: Vec<[f32; 3]>,
        faces: Vec<[u32; 3]>,
        skin: SkinWeights,
    ) -> Result<Self> {
        let normals = Self::fitted_normals(&positions, &faces)?;
        Ok(Self {
            positions,
            normals,
            faces,
            skin,
        })
    }

    /// Preserve a positive direction for every incident face after seating.
    /// Positions and connectivity use the canonical unposed metre frame.
    pub(crate) fn fitted_normals(
        positions: &[[f32; 3]],
        faces: &[[u32; 3]],
    ) -> Result<Vec<[f32; 3]>> {
        let points = positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect::<Vec<_>>();
        Ok(normal_field::outward(&points, faces)?
            .into_iter()
            .map(|v| v.to_array())
            .collect())
    }

    pub fn wearer<'a>(&'a self, body: &Wearer<'a>) -> Wearer<'a> {
        Wearer {
            positions: &self.positions,
            normals: &self.normals,
            faces: &self.faces,
            joint_indices: &self.skin.indices,
            joint_weights: &self.skin.weights,
            joint_names: body.joint_names,
            joints: body.joints,
        }
    }
}

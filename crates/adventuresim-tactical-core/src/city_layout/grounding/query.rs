//! Vertical support uses a finite plan point and an explicit ceiling role.
use super::SupportElevation;
use crate::scene_coordinates::ScenePlanPoint;
use adventuresim_building_generator::spatial_geometry::{
    CoordinateAxis, GeometryError, GeometryRole,
};
use bevy::math::{Vec3, Vec3Swizzles};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SupportCeiling {
    Bounded(SupportElevation),
    Unbounded,
}

/// An unbounded query never manufactures an infinite scene position.
/// ```compile_fail
/// use adventuresim_tactical_core::city_layout::grounding::SupportQuery;
/// use adventuresim_building_generator::spatial_geometry::{Architectural, Position};
/// fn wrong_frame(point: Position<Architectural>) -> SupportQuery { point.into() }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupportQuery {
    pub point: ScenePlanPoint,
    pub ceiling: SupportCeiling,
}
impl SupportQuery {
    pub fn bounded(point: ScenePlanPoint, ceiling: SupportElevation) -> Self {
        Self {
            point,
            ceiling: SupportCeiling::Bounded(ceiling),
        }
    }
    pub fn unbounded(point: ScenePlanPoint) -> Self {
        Self {
            point,
            ceiling: SupportCeiling::Unbounded,
        }
    }
    pub(crate) fn permits(self, elevation: SupportElevation, tolerance_metres: f32) -> bool {
        match self.ceiling {
            SupportCeiling::Unbounded => true,
            SupportCeiling::Bounded(ceiling) => {
                elevation.metres() <= ceiling.metres() + tolerance_metres
            }
        }
    }
}
/// Native physics/mesh positions admit a finite bounded query at their adapter.
impl TryFrom<Vec3> for SupportQuery {
    type Error = GeometryError;
    fn try_from(position: Vec3) -> Result<Self, Self::Error> {
        Ok(Self::bounded(
            ScenePlanPoint::try_from(position.xz())?,
            SupportElevation::from_metres(position.y).ok_or(GeometryError::NonFinite {
                role: GeometryRole::Elevation,
                axis: CoordinateAxis::Y,
            })?,
        ))
    }
}

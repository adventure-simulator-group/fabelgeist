//! Connect each physical scupper to the exact crown owner’s wall-walk datum.
use super::*;
pub(super) fn connect_outlets(
    crowns: &[CrownAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<(), GenerationError> {
    for (index, void) in geometry.voids.iter_mut().enumerate() {
        void.id = ResolvedItemId((3_u64 << 60) | (u64::from(void.owner.0) << 32) | index as u64);
        let crown = crowns
            .iter()
            .find(|crown| crown.owner == void.owner)
            .ok_or(GenerationError::MissingCrown { owner: void.owner })?;
        let centre = (void.bounds.min().metres() + void.bounds.max().metres()) * 0.5;
        let outward = match crown.path {
            CrownPath::Straight { outward, .. } => direction_vector(outward),
            CrownPath::Round { centre: tower, .. } => {
                (Vec2::new(centre.x, centre.z) - tower).normalize_or_zero()
            }
        };
        geometry.drainage_routes.push(DrainageRoute {
            id: ResolvedItemId((5_u64 << 60) | index as u64),
            owner: void.owner,
            outlet_void: void.id,
            inlet: Vec3::new(
                centre.x - outward.x * (crown.profile.thickness_metres * 0.5 + 0.01),
                crown.base_height_metres - 0.02,
                centre.z - outward.y * (crown.profile.thickness_metres * 0.5 + 0.01),
            ),
            outlet: Vec3::new(
                centre.x + outward.x * 0.35,
                crown.base_height_metres - 0.08,
                centre.z + outward.y * 0.35,
            ),
        });
    }
    Ok(())
}

pub(super) fn nearest_route(
    routes: &[DrainageRoute],
    centre: Vec2,
    outward: Vec2,
    owner: GeometryOwnerId,
) -> Result<DrainageRoute, GenerationError> {
    routes
        .iter()
        .min_by(|a, b| {
            let a_direction = Vec2::new(a.outlet.x, a.outlet.z) - centre;
            let b_direction = Vec2::new(b.outlet.x, b.outlet.z) - centre;
            let a_dot = a_direction.normalize_or_zero().dot(outward);
            let b_dot = b_direction.normalize_or_zero().dot(outward);
            b_dot.total_cmp(&a_dot)
        })
        .copied()
        .ok_or(GenerationError::MissingCrownDrainage { owner })
}

use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{PlanDirection, PositiveLength};
pub(super) enum ChannelOutletEnd {
    PositiveTangent,
    NegativeTangent,
}
impl ChannelOutletEnd {
    fn sign(&self) -> f32 {
        match self {
            Self::PositiveTangent => 1.0,
            Self::NegativeTangent => -1.0,
        }
    }
}
/// Admitted architectural inputs to the existing native channel sampling kernel.
pub(super) struct CrownChannelSetOut {
    toe: ArchitecturalPlanPoint,
    inlet: ArchitecturalPlanPoint,
    tangent: PlanDirection<Architectural>,
    span: PositiveLength,
    outlet: ChannelOutletEnd,
}
impl CrownChannelSetOut {
    pub(super) fn from_metres(
        owner: GeometryOwnerId,
        toe: Vec2,
        inlet: Vec2,
        tangent: Vec2,
        span: f32,
        outlet: ChannelOutletEnd,
    ) -> Result<Self, GenerationError> {
        let construct = || {
            Ok::<_, crate::spatial_geometry::GeometryError>(Self {
                toe: ArchitecturalPlanPoint::from_metres(toe)?,
                inlet: ArchitecturalPlanPoint::from_metres(inlet)?,
                tangent: PlanDirection::from_normalized(tangent)?,
                span: PositiveLength::from_metres(span)?,
                outlet,
            })
        };
        construct().map_err(|cause| GenerationError::CrownDrainageConstruction { owner, cause })
    }
    /// Native Vec2 values are scratch projections consumed immediately by the
    /// admitted collision-solid constructor, not stored architectural poses.
    pub(super) fn points(self, path: CrownPath) -> Vec<Vec2> {
        let toe_centre = self.toe.metres();
        let inlet = self.inlet.metres();
        let tangent = self.tangent.vector();
        let length_metres = self.span.metres();
        let outlet_sign = self.outlet.sign();
        let far_toe = toe_centre - tangent * outlet_sign * length_metres * 0.5;
        match path {
            CrownPath::Round {
                centre: tower_centre,
                ..
            } => {
                let near_toe = toe_centre + tangent * outlet_sign * length_metres * 0.5;
                let start_delta = near_toe - tower_centre;
                let end_delta = inlet - tower_centre;
                let start_angle = start_delta.y.atan2(start_delta.x);
                let angle_delta = (end_delta.y.atan2(end_delta.x) - start_angle
                    + std::f32::consts::PI)
                    .rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                let steps = (angle_delta.abs() / (std::f32::consts::PI / 48.0))
                    .ceil()
                    .max(1.0) as usize;
                let gutter_radius = (toe_centre - tower_centre).length();
                let mut points = vec![far_toe, near_toe];
                points.extend((0..=steps).map(|index| {
                    let progress = index as f32 / steps as f32;
                    let angle = start_angle + angle_delta * progress;
                    tower_centre + Vec2::new(angle.cos(), angle.sin()) * gutter_radius
                }));
                if points
                    .last()
                    .is_none_or(|point| point.distance(inlet) > 0.02)
                {
                    points.push(inlet);
                }
                points.dedup_by(|left, right| left.distance(*right) < 0.02);
                points
            }
            CrownPath::Straight { .. } => {
                let near_toe = toe_centre + tangent * outlet_sign * length_metres * 0.5;
                vec![far_toe, near_toe, inlet]
            }
        }
    }
}

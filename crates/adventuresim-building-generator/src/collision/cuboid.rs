//! Cuboid topology and pose arithmetic are owned together.
use super::*;
use crate::plan_geometry::PlanGeometryError;

pub(super) struct CuboidCorners {
    points: [Vec3; 8],
}
pub(super) struct CuboidEdge {
    pub start: Vec3,
    pub end: Vec3,
}
impl CuboidCorners {
    pub fn points(&self) -> &[Vec3; 8] {
        &self.points
    }
    pub fn edges(&self) -> impl Iterator<Item = CuboidEdge> + '_ {
        // The 12 edges of the x/y/z sign-coded corner order. Each undirected
        // edge occurs once; indexing stays inside this topology owner.
        const EDGES: [[usize; 2]; 12] = [
            [0, 1],
            [0, 2],
            [0, 4],
            [1, 3],
            [1, 5],
            [2, 3],
            [2, 6],
            [3, 7],
            [4, 5],
            [4, 6],
            [5, 7],
            [6, 7],
        ];
        EDGES.into_iter().map(|[start, end]| CuboidEdge {
            start: self.points[start],
            end: self.points[end],
        })
    }
}
impl CollisionCuboid {
    pub(super) fn corners(self) -> Result<CuboidCorners, PlanGeometryError> {
        if !self.centre.is_finite()
            || !self.size.is_finite()
            || !self.yaw_radians.is_finite()
            || !self.crossfall_radians.is_finite()
            || !self.longfall_radians.is_finite()
        {
            return Err(PlanGeometryError::NonFinite);
        }
        if self.size.min_element() < 0.0 {
            return Err(PlanGeometryError::NegativeSolidDimension);
        }
        let rotation = bevy::math::Quat::from_rotation_y(self.yaw_radians)
            * bevy::math::Quat::from_rotation_x(self.crossfall_radians)
            * bevy::math::Quat::from_rotation_z(self.longfall_radians);
        let points = std::array::from_fn(|i| {
            let sign = Vec3::new(
                if i & 1 == 0 { -1.0 } else { 1.0 },
                if i & 2 == 0 { -1.0 } else { 1.0 },
                if i & 4 == 0 { -1.0 } else { 1.0 },
            );
            self.centre + rotation * (self.size * sign * 0.5)
        });
        if points.iter().any(|p| !p.is_finite()) {
            return Err(PlanGeometryError::NonFinite);
        }
        Ok(CuboidCorners { points })
    }
}

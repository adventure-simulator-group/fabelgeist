//! Canonical sign-coded cuboid topology, independent of its coordinate frame.
use super::*;

pub struct CuboidCorners<F: GeometryFrame> {
    points: [Position<F>; 8],
}
pub struct CuboidEdge<F: GeometryFrame> {
    pub start: Position<F>,
    pub end: Position<F>,
}
impl<F: GeometryFrame> CuboidCorners<F> {
    /// Preserves the caller's rotation calculation and existing f32 corner order.
    pub fn from_pose(
        centre: Position<F>,
        size: CuboidDimensions,
        rotation: crate::spatial_geometry::RigidRotation,
    ) -> Result<Self, GeometryError> {
        let mut points = [centre; 8];
        for (i, point) in points.iter_mut().enumerate() {
            let sign = Vec3::new(
                if i & 1 == 0 { -1.0 } else { 1.0 },
                if i & 2 == 0 { -1.0 } else { 1.0 },
                if i & 4 == 0 { -1.0 } else { 1.0 },
            );
            *point = Position::from_metres(
                centre.metres() + rotation.quaternion() * (size.metres() * sign * 0.5),
            )?;
        }
        Ok(Self { points })
    }
    pub fn points(&self) -> &[Position<F>; 8] {
        &self.points
    }
    pub fn edges(&self) -> impl Iterator<Item = CuboidEdge<F>> + '_ {
        // Every undirected edge occurs exactly once. Indexes belong to topology.
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
impl<F: GeometryFrame> CollisionCuboid<F> {
    pub fn corners(self) -> Result<CuboidCorners<F>, CollisionError> {
        let rotation = bevy::math::Quat::from_rotation_y(self.yaw_radians.radians())
            * bevy::math::Quat::from_rotation_x(self.crossfall_radians.radians())
            * bevy::math::Quat::from_rotation_z(self.longfall_radians.radians());
        let construct = || {
            CuboidCorners::from_pose(
                self.centre,
                self.size,
                crate::spatial_geometry::RigidRotation::from_quaternion(rotation)?,
            )
        };
        construct().map_err(|cause| CollisionError {
            source_id: self.source,
            cause,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn traversal_covers_eight_corners_and_twelve_unique_edges() {
        let centre = Position::<Architectural>::from_metres(Vec3::new(-2.0, 3.0, 7.0)).unwrap();
        let dimensions = CuboidDimensions::from_metres(Vec3::new(2.0, 4.0, 6.0)).unwrap();
        let corners = CuboidCorners::from_pose(
            centre,
            dimensions,
            crate::spatial_geometry::RigidRotation::IDENTITY,
        )
        .unwrap();
        let mut degrees = [0; 8];
        let mut pairs = std::collections::BTreeSet::new();
        for edge in corners.edges() {
            let a = corners
                .points()
                .iter()
                .position(|p| *p == edge.start)
                .unwrap();
            let b = corners
                .points()
                .iter()
                .position(|p| *p == edge.end)
                .unwrap();
            assert_eq!((a ^ b).count_ones(), 1);
            degrees[a] += 1;
            degrees[b] += 1;
            assert!(pairs.insert((a.min(b), a.max(b))));
        }
        assert_eq!(pairs.len(), 12);
        assert_eq!(degrees, [3; 8]);
        assert_eq!(corners.points()[0].metres(), Vec3::new(-3.0, 1.0, 4.0));
        assert_eq!(corners.points()[7].metres(), Vec3::new(-1.0, 5.0, 10.0));
    }
}

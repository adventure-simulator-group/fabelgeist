//! Residual enclosure material derived from reciprocal inset-wall dimensions.
use bevy::math::{Vec2, Vec3};
use geo::{BooleanOps, Coord, LineString, MultiPolygon, Polygon};

use crate::{RoofEnclosureFace, WallAssembly, WallSourceId};

impl RoofEnclosureFace {
    pub(crate) fn new(
        id: crate::ResolvedItemId,
        polygon: Vec<Vec3>,
        material: crate::RoofMaterial,
        support_nodes: Vec<crate::StructuralNodeId>,
    ) -> Self {
        Self {
            id,
            polygon,
            material,
            support_nodes,
            inset_walls: Vec::new(),
        }
    }

    pub(crate) fn normal(&self) -> Vec3 {
        (self.polygon[1] - self.polygon[0])
            .cross(self.polygon[2] - self.polygon[0])
            .normalize_or_zero()
    }

    pub(crate) fn tangent(&self) -> Vec3 {
        let normal = self.normal();
        Vec3::new(normal.z, 0.0, -normal.x)
    }

    /// This is computed, never serialized as a second geometry authority.
    pub(crate) fn residual(&self, walls: &[WallAssembly]) -> MultiPolygon<f32> {
        let tangent = self.tangent();
        let project = |p: Vec3| Vec2::new(p.dot(tangent), p.y);
        let polygon = |points: Vec<Vec2>| {
            let mut coordinates = points
                .into_iter()
                .map(|p| Coord { x: p.x, y: p.y })
                .collect::<Vec<_>>();
            coordinates.push(coordinates[0]);
            Polygon::new(LineString(coordinates), Vec::new())
        };
        let mut residual = MultiPolygon(vec![polygon(
            self.polygon.iter().copied().map(project).collect(),
        )]);
        for wall in walls.iter().filter(|wall| {
            self.inset_walls.contains(&wall.id)
                && matches!(wall.source, WallSourceId::RoofGable { enclosure, .. } if enclosure == self.id)
        }) {
            let left = wall.frame.origin - wall.frame.tangent * wall.length_metres * 0.5;
            let right = wall.frame.origin + wall.frame.tangent * wall.length_metres * 0.5;
            let bottom = wall.base_elevation_metres;
            let top = bottom + wall.height_metres;
            let points = [Vec3::new(left.x, bottom, left.y), Vec3::new(right.x, bottom, right.y),
                Vec3::new(right.x, top, right.y), Vec3::new(left.x, top, left.y)];
            residual = residual.difference(&polygon(points.into_iter().map(project).collect()));
        }
        residual
    }
}

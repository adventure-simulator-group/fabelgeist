use super::*;
use bevy::math::Vec3Swizzles;
mod assembly;

#[derive(Clone, Copy)]
pub(super) enum SupportFaceRole {
    Bearing,
    Retaining,
}
mod union;

/// Indexed metre-space surfaces. Each retaining face has distinct vertices on
/// both levels; no averaging can lift an adjacent occupied floor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertySupportMesh {
    pub property_id: CityPropertyId,
    pub member_building_ids: Vec<u64>,
    pub positions: Vec<Vec3>,
    pub support_triangles: Vec<[u32; 3]>,
    pub retaining_triangles: Vec<[u32; 3]>,
    pub(super) contact_tolerance_metres: f32,
}

impl PropertySupportMesh {
    /// The same support and retaining triangles used by physical traversal.
    /// This surface collider does not establish a finite foundation volume.
    pub fn collider(&self) -> avian3d::prelude::Collider {
        avian3d::prelude::Collider::trimesh(
            self.positions.clone(),
            self.support_triangles
                .iter()
                .chain(&self.retaining_triangles)
                .copied()
                .collect(),
        )
    }

    pub(super) fn quad(
        &mut self,
        points: [Vec3; 4],
        role: SupportFaceRole,
    ) -> Result<(), SupportDiagnostic> {
        let start = self.vertex_start(points.len(), points[0].xz())?;
        self.positions.extend(points);
        let triangles = [[start, start + 2, start + 1], [start, start + 3, start + 2]];
        if matches!(role, SupportFaceRole::Retaining) {
            self.retaining_triangles.extend(triangles);
        } else {
            self.support_triangles.extend(triangles);
        }
        Ok(())
    }

    pub(super) fn convex_floor(
        &mut self,
        outline: &crate::scene_coordinates::ScenePlanPolygon,
        elevation: SupportElevation,
    ) -> Result<(), SupportDiagnostic> {
        let outline = outline.vertices();
        let start = self.vertex_start(outline.len(), outline[0].metres())?;
        self.positions.extend(outline.iter().map(|p| {
            let point = p.metres();
            Vec3::new(point.x, elevation.metres(), point.y)
        }));
        self.support_triangles.extend(
            (1..outline.len() - 1).map(|i| [start, start + i as u32 + 1, start + i as u32]),
        );
        Ok(())
    }

    pub(super) fn append(&mut self, other: &Self) -> Result<(), SupportDiagnostic> {
        let point = other.positions.first().map_or(Vec2::ZERO, |p| p.xz());
        let offset = self.vertex_start(other.positions.len(), point)?;
        self.positions.extend_from_slice(&other.positions);
        self.support_triangles.extend(
            other
                .support_triangles
                .iter()
                .map(|t| t.map(|i| i + offset)),
        );
        self.retaining_triangles.extend(
            other
                .retaining_triangles
                .iter()
                .map(|t| t.map(|i| i + offset)),
        );
        Ok(())
    }

    pub fn elevations_at(
        &self,
        scene_point: crate::scene_coordinates::ScenePlanPoint,
    ) -> SurfaceElevations {
        let point = scene_point.metres();
        let mut heights = self
            .support_triangles
            .iter()
            .filter_map(|indices| {
                let [a, b, c] = indices.map(|i| self.positions[i as usize]);
                let ab = b.xz() - a.xz();
                let ac = c.xz() - a.xz();
                let ap = point - a.xz();
                let determinant = ab.perp_dot(ac);
                if determinant.abs() <= f32::EPSILON {
                    return None;
                }
                let u = ap.perp_dot(ac) / determinant;
                let v = ab.perp_dot(ap) / determinant;
                let tolerance = self.contact_tolerance_metres * ab.length().max(ac.length())
                    / determinant.abs();
                (u >= -tolerance && v >= -tolerance && u + v <= 1.0 + tolerance)
                    .then_some(SupportElevation(a.y + (b.y - a.y) * u + (c.y - a.y) * v))
            })
            .collect::<Vec<_>>();
        heights.sort_by(|a, b| a.metres().total_cmp(&b.metres()));
        heights.dedup_by(|a, b| (a.metres() - b.metres()).abs() <= self.contact_tolerance_metres);
        SurfaceElevations(heights)
    }

    /// Maximum physical grade of any support triangle, excluding vertical
    /// retaining faces that are not an actor's walkable surface.
    pub fn maximum_grade(&self) -> f32 {
        self.support_triangles
            .iter()
            .map(|indices| {
                let [a, b, c] = indices.map(|i| self.positions[i as usize]);
                let normal = (b - a).cross(c - a);
                normal.xz().length() / normal.y.abs()
            })
            .fold(0.0, f32::max)
    }
}

impl PropertySupportMesh {
    pub(super) fn empty_for_compound(plan: &CompoundSupportPlan) -> Self {
        PropertySupportMesh {
            property_id: plan.property.id,
            member_building_ids: vec![
                plan.property.front_building_id,
                plan.property.rear_building_id,
            ],
            positions: Vec::new(),
            support_triangles: Vec::new(),
            retaining_triangles: Vec::new(),
            contact_tolerance_metres: plan.limits.contact_tolerance_metres,
        }
    }
}

mod grid;

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
#[serde(try_from = "SupportMeshWire")]
pub struct PropertySupportMesh {
    pub(super) property_id: CityPropertyId,
    pub(super) member_building_ids: Vec<crate::scene_input::SceneBuildingId>,
    pub(super) positions: Vec<Vec3>,
    pub(super) support_triangles: Vec<[u32; 3]>,
    pub(super) retaining_triangles: Vec<[u32; 3]>,
    pub(super) contact_tolerance_metres: f32,
}

#[derive(Deserialize)]
struct SupportMeshWire {
    property_id: CityPropertyId,
    member_building_ids: Vec<crate::scene_input::SceneBuildingId>,
    positions: Vec<Vec3>,
    support_triangles: Vec<[u32; 3]>,
    retaining_triangles: Vec<[u32; 3]>,
    contact_tolerance_metres: f32,
}

impl TryFrom<SupportMeshWire> for PropertySupportMesh {
    type Error = SupportSurfaceAdmissionError;
    fn try_from(wire: SupportMeshWire) -> Result<Self, Self::Error> {
        let mesh = Self {
            property_id: wire.property_id,
            member_building_ids: wire.member_building_ids,
            positions: wire.positions,
            support_triangles: wire.support_triangles,
            retaining_triangles: wire.retaining_triangles,
            contact_tolerance_metres: wire.contact_tolerance_metres,
        };
        mesh.validate_geometry()
            .map_err(|issue| SupportSurfaceAdmissionError::for_mesh(&mesh, issue))?;
        Ok(mesh)
    }
}

impl PropertySupportMesh {
    pub fn property_id(&self) -> CityPropertyId {
        self.property_id
    }
    pub fn member_building_ids(&self) -> &[crate::scene_input::SceneBuildingId] {
        &self.member_building_ids
    }
    /// Finite native scene-metre vertices at the mesh upload/physics boundary.
    pub fn positions(&self) -> &[Vec3] {
        &self.positions
    }
    pub fn support_triangles(&self) -> &[[u32; 3]] {
        &self.support_triangles
    }
    pub fn retaining_triangles(&self) -> &[[u32; 3]] {
        &self.retaining_triangles
    }
    fn validate_geometry(&self) -> Result<(), SupportSurfaceIssue> {
        if self.property_id.0 == 0
            || self.member_building_ids.is_empty()
            || self
                .member_building_ids
                .iter()
                .enumerate()
                .any(|(i, id)| id.0 == 0 || self.member_building_ids[..i].contains(id))
        {
            return Err(SupportSurfaceIssue::Members);
        }
        if !self.contact_tolerance_metres.is_finite()
            || self.contact_tolerance_metres <= 0.0
            || self.positions.iter().any(|p| !p.is_finite())
        {
            return Err(SupportSurfaceIssue::Bounds);
        }
        if self.positions.len() > u32::MAX as usize
            || self.support_triangles.is_empty()
            || self
                .support_triangles
                .iter()
                .chain(&self.retaining_triangles)
                .any(|t| {
                    t.iter().any(|i| *i as usize >= self.positions.len())
                        || t[0] == t[1]
                        || t[1] == t[2]
                        || t[2] == t[0]
                })
        {
            return Err(SupportSurfaceIssue::Topology);
        }
        Ok(())
    }

    /// The same support and retaining triangles used by physical traversal.
    /// This surface collider does not establish a finite foundation volume.
    pub fn collider(&self) -> Result<avian3d::prelude::Collider, SupportColliderError> {
        avian3d::prelude::Collider::try_trimesh(
            self.positions.clone(),
            self.support_triangles
                .iter()
                .chain(&self.retaining_triangles)
                .copied()
                .collect(),
        )
        .map_err(|cause| SupportColliderError::Surface {
            property: self.property_id,
            members: self.member_building_ids.clone(),
            cause,
        })
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
                if !(u >= -tolerance && v >= -tolerance && u + v <= 1.0 + tolerance) {
                    return None;
                }
                let height = a.y + (b.y - a.y) * u + (c.y - a.y) * v;
                SupportElevation::from_metres(height).or_else(|| {
                    let height = f64::from(a.y)
                        + (f64::from(b.y) - f64::from(a.y)) * f64::from(u)
                        + (f64::from(c.y) - f64::from(a.y)) * f64::from(v);
                    SupportElevation::from_metres(height as f32)
                })
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
            contact_tolerance_metres: plan.limits.contact_tolerance_metres.metres(),
        }
    }
}

pub(super) mod grid;

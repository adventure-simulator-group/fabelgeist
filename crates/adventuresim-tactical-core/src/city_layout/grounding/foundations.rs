//! Closed support cells retain exact source intersections and owned boundaries.
use super::*;
use bevy::math::Vec3Swizzles;
mod banks;
mod constraints;
mod geometry;
mod replacement;
mod settlement;
mod source;
mod transport;
pub(super) use geometry::GroundTriangle;
pub use replacement::BoundedPropertyTerrain;
pub use settlement::{BoundedSettlementTerrain, SettlementSupportError};
pub use source::{
    GeographicHeightControl, GeographicHeightRange, GeographicRegionMeasurement, GeographicSurface,
    GeographicSurfaceComparison, SurfaceDifferenceControl,
};

/// Closed topology of one six-vertex prism with positive plan winding.
const FOUNDATION_PRISM_TRIANGLES: [[u32; 3]; 8] = [
    [0, 2, 1],
    [3, 4, 5],
    [0, 1, 4],
    [0, 4, 3],
    [1, 2, 5],
    [1, 5, 4],
    [2, 0, 3],
    [2, 3, 5],
];

/// Positive foundation depth below both the selected surface and source ground.
/// The caller supplies an architectural value, not a movement tolerance.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, bevy::reflect::Reflect)]
#[reflect(opaque)]
#[serde(transparent)]
pub struct FoundationEmbedment(adventuresim_building_generator::spatial_geometry::PositiveLength);

impl FoundationEmbedment {
    pub const fn new(
        depth: adventuresim_building_generator::spatial_geometry::PositiveLength,
    ) -> Self {
        Self(depth)
    }
    pub fn metres(self) -> f32 {
        self.0.metres()
    }

    pub(crate) fn is_valid(self) -> bool {
        Self::from_metres(self.0.metres()).is_some()
    }
    pub fn from_metres(metres: f32) -> Option<Self> {
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(metres)
            .ok()
            .map(Self)
    }
}

impl CompoundSupportPlan {
    pub(super) fn bounded_floor_shift(
        &self,
        source: &GeographicSurface,
    ) -> Result<f32, SupportDiagnostic> {
        constraints::floor_shift(self, source)
    }
}

/// Each clipped triangular cell is a closed finite prism. Support, bottom and
/// side triangles share its six vertices; no foundation floats above the source.
/// This generated geometry does not establish structural strength or drainage.
#[derive(Clone, Debug, PartialEq, bevy::prelude::Reflect)]
#[reflect(opaque)]
pub struct PropertyFoundationMesh {
    pub(in crate::city_layout::grounding) property_id: CityPropertyId,
    pub(in crate::city_layout::grounding) member_building_ids:
        Vec<crate::scene_input::SceneBuildingId>,
    pub(in crate::city_layout::grounding) positions: Vec<Vec3>,
    pub(in crate::city_layout::grounding) solid_triangles: Vec<[u32; 3]>,
    pub(in crate::city_layout::grounding) support_triangles: Vec<[u32; 3]>,
    /// Vertical exposed source faces at owned cuts; these are not floor bearings.
    pub(in crate::city_layout::grounding) cut_faces: Vec<[Vec3; 3]>,
}

impl PropertyFoundationMesh {
    /// Admit exact scene-frame prism cells and ordered physical ownership.
    /// Triangle topology is derived once from these six-vertex cells.
    pub fn from_prisms(
        property_id: CityPropertyId,
        members: PropertyMembers,
        cells: Vec<
            [adventuresim_building_generator::spatial_geometry::Position<
                crate::scene_coordinates::Scene,
            >; 6],
        >,
        cut_faces: Vec<
            [adventuresim_building_generator::spatial_geometry::Position<
                crate::scene_coordinates::Scene,
            >; 3],
        >,
    ) -> Result<Self, SupportGeometryIssue> {
        let mut mesh = Self {
            property_id,
            member_building_ids: members.ids().to_vec(),
            positions: Vec::new(),
            solid_triangles: Vec::new(),
            support_triangles: Vec::new(),
            cut_faces: cut_faces
                .into_iter()
                .map(|t| t.map(|p| p.metres()))
                .collect(),
        };
        if cells.len() > u32::MAX as usize / 6 {
            return Err(SupportGeometryIssue::Topology);
        }
        for cell in cells {
            let points = cell.map(|p| p.metres());
            super::admission::prism(&points)?;
            mesh.append_cell(
                [points[0], points[1], points[2]],
                [points[3], points[4], points[5]],
            )?;
        }
        mesh.validate()?;
        Ok(mesh)
    }

    pub fn property_id(&self) -> CityPropertyId {
        self.property_id
    }
    pub fn member_building_ids(&self) -> &[crate::scene_input::SceneBuildingId] {
        &self.member_building_ids
    }
    /// Native scene-metre mesh storage, admitted once before indexing or upload.
    pub fn positions(&self) -> &[Vec3] {
        &self.positions
    }
    pub fn solid_triangles(&self) -> &[[u32; 3]] {
        &self.solid_triangles
    }
    pub fn support_triangles(&self) -> &[[u32; 3]] {
        &self.support_triangles
    }
    pub fn cut_faces(&self) -> &[[Vec3; 3]] {
        &self.cut_faces
    }
    pub(crate) fn validate(&self) -> Result<(), SupportGeometryIssue> {
        PropertyMembers::validate(&self.member_building_ids)?;
        if self.property_id.0 == 0 {
            return Err(SupportGeometryIssue::Members);
        }
        if !self.positions.len().is_multiple_of(6) || u32::try_from(self.positions.len()).is_err() {
            return Err(SupportGeometryIssue::Topology);
        }
        let cells = self.positions.len() / 6;
        if self.support_triangles.len() != cells
            || self.solid_triangles.len() != cells * FOUNDATION_PRISM_TRIANGLES.len()
        {
            return Err(SupportGeometryIssue::Topology);
        }
        for (cell, points) in self.positions.as_chunks::<6>().0.iter().enumerate() {
            super::admission::prism(points)?;
            let start = (cell * 6) as u32;
            if self.support_triangles[cell] != [start, start + 2, start + 1]
                || self.solid_triangles[cell * 8..(cell + 1) * 8]
                    != FOUNDATION_PRISM_TRIANGLES.map(|t| t.map(|i| i + start))
            {
                return Err(SupportGeometryIssue::Topology);
            }
        }
        if self.cut_faces.iter().flatten().any(|p| !p.is_finite()) {
            return Err(SupportGeometryIssue::NonFinite);
        }
        Ok(())
    }

    pub fn collider(&self) -> Result<SupportCollision, SupportColliderError> {
        let mut parts = Vec::new();
        for (cell, points) in self.positions.as_chunks::<6>().0.iter().enumerate() {
            let points: &[Vec3; 6] = points;
            let Some(faces) = super::admission::solid_faces(points, FOUNDATION_PRISM_TRIANGLES)
            else {
                continue;
            };
            let centre = points.iter().copied().sum::<Vec3>() / 6.0;
            let vertices = points.iter().map(|p| *p - centre).collect();
            // Explicit prism faces preserve finite thin cells without rebuilding
            // a hull or introducing a geometric proximity threshold.
            let shape = avian3d::parry::shape::SharedShape::convex_mesh(vertices, &faces);
            let Some(shape) = shape else {
                return Err(SupportColliderError::Foundation {
                    property: self.property_id,
                    members: self.member_building_ids.clone(),
                    cell,
                    issue: SupportGeometryIssue::Collider,
                });
            };
            parts.push((
                centre,
                bevy::math::Quat::IDENTITY,
                avian3d::prelude::Collider::from(shape),
            ));
        }
        Ok(SupportCollision::compound(parts))
    }

    pub fn volume_cubic_metres(&self) -> f64 {
        self.positions
            .as_chunks::<6>()
            .0
            .iter()
            .map(|cell| {
                let [a, b, c] = [cell[0].xz(), cell[1].xz(), cell[2].xz()];
                let area = (b.as_dvec2() - a.as_dvec2())
                    .perp_dot(c.as_dvec2() - a.as_dvec2())
                    .abs()
                    * 0.5;
                let depth = (0..3)
                    .map(|i| f64::from(cell[i].y - cell[i + 3].y))
                    .sum::<f64>();
                area * depth / 3.0
            })
            .sum()
    }

    fn append_cell(
        &mut self,
        top: [Vec3; 3],
        bottom: [Vec3; 3],
    ) -> Result<(), SupportGeometryIssue> {
        let total = self
            .positions
            .len()
            .checked_add(6)
            .ok_or(SupportGeometryIssue::Topology)?;
        u32::try_from(total).map_err(|_| SupportGeometryIssue::Topology)?;
        let start = self.positions.len() as u32;
        self.positions.extend(top);
        self.positions.extend(bottom);
        let support = [start, start + 2, start + 1];
        self.support_triangles.push(support);
        self.solid_triangles
            .extend(FOUNDATION_PRISM_TRIANGLES.map(|t| t.map(|i| i + start)));
        Ok(())
    }
}

pub(super) fn compile(
    plan: &PropertySupportSurface,
    geographic: &GeographicSurface,
    embedment: FoundationEmbedment,
) -> Result<PropertyFoundationMesh, SupportDiagnostic> {
    let surface = &plan.mesh;
    let mut mesh = PropertyFoundationMesh {
        property_id: surface.property_id,
        member_building_ids: surface.member_building_ids.clone(),
        positions: Vec::new(),
        solid_triangles: Vec::new(),
        support_triangles: Vec::new(),
        cut_faces: Vec::new(),
    };
    for indices in &surface.support_triangles {
        let support = GroundTriangle::new(indices.map(|i| surface.positions[i as usize]))
            .ok_or_else(|| geometry_rejection(plan, SupportGeometryIssue::Topology))?;
        visit_source_intersections(plan, geographic, &support, |polygon, source| {
            // The bottom is the minimum of two planes. Split at their zero line
            // so a source crossing cannot conceal a void.
            for section in geometry::height_sections(polygon, &support, source) {
                for i in 1..section.len().saturating_sub(1) {
                    let points = [section[0], section[i], section[i + 1]];
                    if geometry::area(&points) <= f64::EPSILON {
                        continue;
                    }
                    let top = points.map(|p| Vec3::new(p.x, support.height_at(p), p.y));
                    let bottom = points.map(|p| {
                        Vec3::new(
                            p.x,
                            support.height_at(p).min(source.height_at(p)) - embedment.metres(),
                            p.y,
                        )
                    });
                    mesh.append_cell(top, bottom)
                        .map_err(|cause| geometry_rejection(plan, cause))?;
                }
            }
            Ok(())
        })?;
    }
    mesh.cut_faces = banks::compile(plan, geographic)?;
    mesh.validate().map_err(|cause| {
        let mut error = plan.rejection(
            SupportConstraint::Reservation,
            SupportBoundary::PropertyReservation,
            plan.regions[0].centre_metres(),
            1.0,
            0.0,
        );
        error.construction_failure = Some(Box::new(SupportConstructionError::Geometry(cause)));
        error
    })?;
    Ok(mesh)
}

/// Selection checks every source intersection without materializing closed
/// prisms that compilation will generate once after all owners are accepted.
pub(super) fn validate_source_controls(
    plan: &PropertySupportSurface,
    geographic: &GeographicSurface,
) -> Result<(), SupportDiagnostic> {
    for indices in &plan.mesh.support_triangles {
        let support = GroundTriangle::new(indices.map(|i| plan.mesh.positions[i as usize]))
            .ok_or_else(|| geometry_rejection(plan, SupportGeometryIssue::Topology))?;
        visit_source_intersections(plan, geographic, &support, |_, _| Ok(()))?;
    }
    // Boundary cuts retain the same source-sample and matching-surface checks.
    banks::compile(plan, geographic)?;
    Ok(())
}

fn visit_source_intersections(
    plan: &PropertySupportSurface,
    geographic: &GeographicSurface,
    support: &GroundTriangle,
    mut visit: impl FnMut(&[Vec2], &GroundTriangle) -> Result<(), SupportDiagnostic>,
) -> Result<(), SupportDiagnostic> {
    let mut covered_area = 0.0;
    for source in geographic.intersecting(support) {
        let polygon = support.intersection(source);
        if polygon.len() < 3 {
            continue;
        }
        for point in &polygon {
            let displacement = (support.height_at(*point) - source.height_at(*point)).abs();
            if displacement > plan.limits.maximum_displacement_metres.metres() {
                let mut error = plan.rejection(
                    SupportConstraint::CutFill,
                    SupportBoundary::GeographicSurface,
                    *point,
                    displacement,
                    plan.limits.maximum_displacement_metres.metres(),
                );
                error.attempted_treatment = Box::new(plan.treatment);
                return Err(error);
            }
        }
        covered_area += geometry::area(&polygon);
        visit(&polygon, source)?;
    }
    validate_coverage(plan, support, covered_area)
}

fn validate_coverage(
    plan: &PropertySupportSurface,
    support: &GroundTriangle,
    covered_area: f64,
) -> Result<(), SupportDiagnostic> {
    let discrepancy = covered_area - support.area();
    let permitted = support.perimeter() * f64::from(plan.limits.contact_tolerance_metres.metres());
    if discrepancy.abs() > permitted {
        let constraint = if discrepancy < 0.0 {
            SupportConstraint::SurfaceCoverage
        } else {
            SupportConstraint::SurfaceOverlap
        };
        let mut error = plan.rejection(
            constraint,
            SupportBoundary::GeographicSurface,
            support.centre(),
            discrepancy.abs() as f32,
            permitted as f32,
        );
        error.attempted_treatment = Box::new(plan.treatment);
        return Err(error);
    }
    Ok(())
}

fn geometry_rejection(
    plan: &PropertySupportSurface,
    cause: SupportGeometryIssue,
) -> SupportDiagnostic {
    let mut error = plan.rejection(
        SupportConstraint::Reservation,
        SupportBoundary::PropertyReservation,
        plan.regions[0].centre_metres(),
        1.0,
        0.0,
    );
    error.construction_failure = Some(Box::new(SupportConstructionError::Geometry(cause)));
    error
}

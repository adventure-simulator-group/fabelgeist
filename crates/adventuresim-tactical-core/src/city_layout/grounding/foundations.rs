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
    GeographicHeightRange, GeographicRegionMeasurement, GeographicSurface,
    GeographicSurfaceComparison, SurfaceDifferenceControl,
};

/// Closed, outward-facing topology of one six-vertex triangular prism.
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FoundationEmbedment(f32);

impl FoundationEmbedment {
    pub fn metres(self) -> f32 {
        self.0
    }

    pub(crate) fn is_valid(self) -> bool {
        Self::from_metres(self.0).is_some()
    }
    pub fn from_metres(metres: f32) -> Option<Self> {
        (metres.is_finite() && metres > 0.0).then_some(Self(metres))
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
pub struct PropertyFoundationMesh {
    pub property_id: CityPropertyId,
    pub member_building_ids: Vec<u64>,
    pub positions: Vec<Vec3>,
    pub solid_triangles: Vec<[u32; 3]>,
    pub support_triangles: Vec<[u32; 3]>,
    /// Vertical exposed source faces at owned cuts; these are not floor bearings.
    pub cut_faces: Vec<[Vec3; 3]>,
}

impl PropertyFoundationMesh {
    pub fn collider(&self) -> avian3d::prelude::Collider {
        avian3d::prelude::Collider::compound(
            self.positions
                .as_chunks::<6>()
                .0
                .iter().enumerate()
                .map(|(index, cell)| {
                    let centre = cell.iter().copied().sum::<Vec3>() / 6.0;
                    let vertices: Vec<_> = cell.iter().map(|p| *p - centre).collect();
                    // The prism topology is already known. Reconstructing a
                    // hull can reject thin but finite source-intersection cells
                    // which remain valid in this explicit closed convex mesh.
                    let shape = avian3d::parry::shape::SharedShape::convex_mesh(
                        vertices, &FOUNDATION_PRISM_TRIANGLES,
                    ).unwrap_or_else(|| panic!(
                        "property {:?}, members {:?}, foundation cell {index} is not a closed convex prism: {cell:?}",
                        self.property_id, self.member_building_ids,
                    ));
                    let collider = avian3d::prelude::Collider::from(shape);
                    (centre, bevy::math::Quat::IDENTITY, collider)
                })
                .collect(),
        )
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

    fn append_cell(&mut self, top: [Vec3; 3], bottom: [Vec3; 3]) {
        let start = u32::try_from(self.positions.len())
            .expect("bounded foundation mesh fits in u32 vertex indices");
        self.positions.extend(top);
        self.positions.extend(bottom);
        let support = [start, start + 2, start + 1];
        self.support_triangles.push(support);
        self.solid_triangles
            .extend(FOUNDATION_PRISM_TRIANGLES.map(|t| t.map(|i| i + start)));
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
            .expect("validated nonvertical support triangle");
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
                            support.height_at(p).min(source.height_at(p)) - embedment.0,
                            p.y,
                        )
                    });
                    mesh.append_cell(top, bottom);
                }
            }
        })?;
    }
    mesh.cut_faces = banks::compile(plan, geographic)?;
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
            .expect("validated nonvertical support triangle");
        visit_source_intersections(plan, geographic, &support, |_, _| {})?;
    }
    // Boundary cuts retain the same source-sample and matching-surface checks.
    banks::compile(plan, geographic)?;
    Ok(())
}

fn visit_source_intersections(
    plan: &PropertySupportSurface,
    geographic: &GeographicSurface,
    support: &GroundTriangle,
    mut visit: impl FnMut(&[Vec2], &GroundTriangle),
) -> Result<(), SupportDiagnostic> {
    let mut covered_area = 0.0;
    for source in geographic.intersecting(support) {
        let polygon = support.intersection(source);
        if polygon.len() < 3 {
            continue;
        }
        for point in &polygon {
            let displacement = (support.height_at(*point) - source.height_at(*point)).abs();
            if displacement > plan.limits.maximum_displacement_metres {
                let mut error = plan.rejection(
                    SupportConstraint::CutFill,
                    SupportBoundary::GeographicSurface,
                    *point,
                    displacement,
                    plan.limits.maximum_displacement_metres,
                );
                error.attempted_treatment = plan.treatment;
                return Err(error);
            }
        }
        covered_area += geometry::area(&polygon);
        visit(&polygon, source);
    }
    validate_coverage(plan, support, covered_area)
}

fn validate_coverage(
    plan: &PropertySupportSurface,
    support: &GroundTriangle,
    covered_area: f64,
) -> Result<(), SupportDiagnostic> {
    let discrepancy = covered_area - support.area();
    let permitted = support.perimeter() * f64::from(plan.limits.contact_tolerance_metres);
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
        error.attempted_treatment = plan.treatment;
        return Err(error);
    }
    Ok(())
}

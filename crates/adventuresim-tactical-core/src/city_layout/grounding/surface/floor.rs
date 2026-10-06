//! Authored floor regions bind decoded support to each member datum.
use super::*;
mod region;
use crate::scene_coordinates::{ScenePlanPoint, ScenePlanPolygon};
use bevy::math::{DVec2, Vec3Swizzles};
pub use region::FloorRegion;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "FloorBearingWire")]
pub struct FloorBearing {
    pub(in crate::city_layout::grounding) building: crate::scene_input::SceneBuildingId,
    pub(in crate::city_layout::grounding) regions: Vec<FloorRegion>,
    pub(in crate::city_layout::grounding) elevation: SupportElevation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FloorBearingWire {
    building: crate::scene_input::SceneBuildingId,
    regions: Vec<FloorRegion>,
    elevation: SupportElevation,
}
impl TryFrom<FloorBearingWire> for FloorBearing {
    type Error = FloorBearingConstructionError;
    fn try_from(wire: FloorBearingWire) -> Result<Self, Self::Error> {
        Self::admit(wire.building, wire.regions, wire.elevation)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, thiserror::Error)]
pub enum FloorBearingConstructionError {
    #[error("floor binding has invalid building {0}")]
    InvalidBuilding(crate::scene_input::SceneBuildingId),
    #[error("floor regions must form a nonempty disjoint partition")]
    InvalidPartition,
    #[error(transparent)]
    Geometry(#[from] adventuresim_building_generator::spatial_geometry::GeometryError),
    #[error(transparent)]
    Plan(#[from] adventuresim_building_generator::plan_geometry::PlanGeometryError),
}
impl FloorBearing {
    fn admit(
        building: crate::scene_input::SceneBuildingId,
        regions: Vec<FloorRegion>,
        elevation: SupportElevation,
    ) -> Result<Self, FloorBearingConstructionError> {
        if building.0 == 0 {
            return Err(FloorBearingConstructionError::InvalidBuilding(building));
        }
        if regions.is_empty() {
            return Err(FloorBearingConstructionError::InvalidPartition);
        }
        let outlines: Vec<Vec<DVec2>> = regions
            .iter()
            .map(|region| {
                region
                    .outline()
                    .vertices()
                    .iter()
                    .map(|p| p.metres().as_dvec2())
                    .collect()
            })
            .collect();
        for (index, region) in outlines.iter().enumerate() {
            if outlines[..index].iter().any(|other| {
                has_positive_overlap(region, other) || has_partial_shared_edge(region, other)
            }) {
                return Err(FloorBearingConstructionError::InvalidPartition);
            }
        }
        Ok(Self {
            building,
            regions,
            elevation,
        })
    }
    // Exact shared partition edges cancel; internal cuts add no tolerance.
    fn exterior_perimeter(&self) -> f64 {
        let mut edges = std::collections::BTreeMap::new();
        for region in &self.regions {
            let vertices = region.outline().vertices();
            for (a, b) in vertices
                .iter()
                .zip(vertices.iter().cycle().skip(1))
                .take(vertices.len())
            {
                let mut endpoints = [
                    a.metres().to_array().map(edge_coordinate_key),
                    b.metres().to_array().map(edge_coordinate_key),
                ];
                endpoints.sort();
                let edge = edges
                    .entry(endpoints)
                    .or_insert((0_u32, a.metres().as_dvec2().distance(b.metres().as_dvec2())));
                edge.0 += 1;
            }
        }
        edges
            .values()
            .filter(|(count, _)| *count == 1)
            .map(|(_, length)| length)
            .sum()
    }
    pub fn building(&self) -> crate::scene_input::SceneBuildingId {
        self.building
    }
    pub fn regions(&self) -> &[FloorRegion] {
        &self.regions
    }
    pub fn elevation(&self) -> SupportElevation {
        self.elevation
    }
    pub(in crate::city_layout::grounding) fn from_member(
        plan: &CompoundSupportPlan,
        member: MemberSupport,
    ) -> Result<Self, FloorBearingConstructionError> {
        Self::admit(
            member.building_id,
            mesh::grid::floor_regions(plan, member)?,
            member.elevation,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum FloorBearingIssue {
    #[error("floor binding has invalid ordered physical membership")]
    Members,
    #[error("floor at {location:?} is {actual:?} m, expected {expected:?}")]
    Datum {
        location: ScenePlanPoint,
        actual: DiagnosticMetres,
        expected: SupportElevation,
    },
    #[error("floor bearing lacks {missing} m² of support, permitted {permitted} m²")]
    Coverage {
        missing: DiagnosticArea,
        permitted: DiagnosticArea,
    },
    #[error("floor bearing cannot be represented by admitted geometry")]
    Geometry,
}
impl PropertySupportSurface {
    pub fn floor_bearings(&self) -> &[FloorBearing] {
        &self.floor_bearings
    }
    pub(super) fn validate_floors(&self) -> Result<(), SupportSurfaceIssue> {
        if self.floor_bearings.len() != self.member_building_ids().len() {
            return Err(SupportSurfaceIssue::Members);
        }
        for (bearing, member) in self.floor_bearings.iter().zip(self.member_building_ids()) {
            if bearing.building != *member {
                return Err(SupportSurfaceIssue::Members);
            }
            self.validate_floor(bearing)
                .map_err(|issue| SupportSurfaceIssue::Floor {
                    building: *member,
                    issue,
                })?;
        }
        Ok(())
    }
    fn validate_floor(&self, bearing: &FloorBearing) -> Result<(), FloorBearingIssue> {
        let mut absent = 0.0;
        let mut wrong_area = 0.0;
        let mut wrong_control = None;
        for region in &bearing.regions {
            let coverage = self.floor_region_coverage(bearing, region.outline())?;
            absent += coverage.absent;
            wrong_area += coverage.wrong_area;
            wrong_control = coverage.wrong_control.or(wrong_control);
        }
        let permitted =
            bearing.exterior_perimeter() * f64::from(self.limits.contact_tolerance_metres.metres());
        if wrong_area > permitted {
            let (point, actual) = wrong_control.ok_or(FloorBearingIssue::Geometry)?;
            return Err(FloorBearingIssue::Datum {
                location: ScenePlanPoint::try_from(point.as_vec2())
                    .map_err(|_| FloorBearingIssue::Geometry)?,
                actual: DiagnosticMetres::from_metres(actual).ok_or(FloorBearingIssue::Geometry)?,
                expected: bearing.elevation,
            });
        }
        if absent > permitted {
            return Err(FloorBearingIssue::Coverage {
                missing: DiagnosticArea::from_square_metres(absent)
                    .ok_or(FloorBearingIssue::Geometry)?,
                permitted: DiagnosticArea::from_square_metres(permitted)
                    .ok_or(FloorBearingIssue::Geometry)?,
            });
        }
        Ok(())
    }
    fn floor_region_coverage(
        &self,
        bearing: &FloorBearing,
        region: &ScenePlanPolygon,
    ) -> Result<FloorRegionCoverage, FloorBearingIssue> {
        let outline: Vec<_> = region
            .vertices()
            .iter()
            .map(|p| p.metres().as_dvec2())
            .collect();
        let mut missing = vec![outline.clone()];
        let mut without_wrong_level = vec![outline.clone()];
        let mut wrong_control = None;
        let tolerance = f64::from(self.limits.contact_tolerance_metres.metres());
        for indices in self.mesh.support_triangles() {
            let points = indices.map(|i| self.mesh.positions()[i as usize]);
            let triangle =
                foundations::GroundTriangle::new(points).ok_or(FloorBearingIssue::Geometry)?;
            let mut face = points.map(|p| p.xz().as_dvec2()).to_vec();
            if planar::signed_area(&face) < 0.0 {
                face.reverse();
            }
            let intersection = intersection_with_outline(face.clone(), &outline);
            // Retaining-edge contact at another level has no bearing area.
            if planar::signed_area(&intersection).abs() <= f64::EPSILON {
                continue;
            }
            if let Some(point) = intersection.iter().find(|point| {
                (triangle.height_f64(**point) - f64::from(bearing.elevation.metres())).abs()
                    > tolerance
            }) {
                // Another retaining/contact level is ineligible coverage. Its
                // uncovered edge sliver uses the existing perimeter allowance.
                wrong_control = Some((*point, triangle.height_f64(*point)));
                without_wrong_level = without_wrong_level
                    .into_iter()
                    .flat_map(|polygon| planar::subtract(polygon, &face))
                    .collect();
                continue;
            }
            // Subtract original convex faces, not their rounded clipped outline.
            // Clipping may add tiny signed bends or redundant edges; treating
            // those fragments as convex cutters can inflate the missing union.
            // Duplicate original faces still cannot mask a hole.
            missing = missing
                .into_iter()
                .flat_map(|polygon| planar::subtract(polygon, &face))
                .collect();
        }
        let absent = missing
            .iter()
            .map(|p| planar::signed_area(p).abs())
            .sum::<f64>();
        let wrong_area = planar::signed_area(&outline).abs()
            - without_wrong_level
                .iter()
                .map(|p| planar::signed_area(p).abs())
                .sum::<f64>();
        Ok(FloorRegionCoverage {
            absent,
            wrong_area,
            wrong_control,
        })
    }
}
// Native f64 scene-metre planar clipping results, consumed by admission only.
struct FloorRegionCoverage {
    absent: f64,
    wrong_area: f64,
    wrong_control: Option<(DVec2, f64)>,
}
fn intersection_with_outline(mut polygon: Vec<DVec2>, outline: &[DVec2]) -> Vec<DVec2> {
    for index in 0..outline.len() {
        let a = outline[index];
        let edge = outline[(index + 1) % outline.len()] - a;
        polygon = planar::clip(polygon, |p| edge.perp_dot(p - a));
    }
    polygon
}

// Conforming authored partitions share complete edges. No proximity threshold
// can turn a partial edge into a complete one during collection admission.
fn has_partial_shared_edge(first: &[DVec2], second: &[DVec2]) -> bool {
    let splits = |vertices: &[DVec2], edges: &[DVec2]| {
        vertices.iter().any(|point| {
            edges
                .iter()
                .zip(edges.iter().cycle().skip(1))
                .take(edges.len())
                .any(|(a, b)| {
                    let edge = *b - *a;
                    let offset = *point - *a;
                    edge.perp_dot(offset) == 0.0
                        && offset.dot(edge) > 0.0
                        && offset.dot(edge) < edge.length_squared()
                })
        })
    };
    splits(first, second) || splits(second, first)
}

#[cfg(test)]
mod tests;

// Edge equality follows numeric admission equality; signed zero is one point.
// This key does not alter transport vertices or geographic hash framing.
fn edge_coordinate_key(coordinate: f32) -> u32 {
    if coordinate == 0.0 {
        0
    } else {
        coordinate.to_bits()
    }
}

// Exact separating half-planes distinguish positive overlap from edge contact.
fn has_positive_overlap(first: &[DVec2], second: &[DVec2]) -> bool {
    let separated = |edges: &[DVec2], vertices: &[DVec2]| {
        edges
            .iter()
            .zip(edges.iter().cycle().skip(1))
            .take(edges.len())
            .any(|(a, b)| vertices.iter().all(|p| (*b - *a).perp_dot(*p - *a) <= 0.0))
    };
    !separated(first, second) && !separated(second, first)
}

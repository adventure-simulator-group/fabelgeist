//! Compose disjoint owned support against one unchanged geographic surface.
use super::*;
use std::collections::BTreeSet;
mod query;
use query::SupportQueryIndex;

#[cfg(test)]
mod tests;

/// A settlement surface owns each foundation once and clips source triangles
/// once. Keeping every property's uncut source collider would put natural
/// terrain back through the neighbouring property's graded floors.
#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize, bevy::prelude::Reflect)]
pub struct BoundedSettlementTerrain {
    pub foundations: Vec<PropertyFoundationMesh>,
    #[serde(with = "crate::geometry_transport")]
    pub natural_triangles: Vec<[Vec3; 3]>,
    contact_tolerance_metres: f32,
    query: SupportQueryIndex,
}

#[derive(Clone, Debug, PartialEq, Serialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum SettlementSupportError {
    #[error("property {property:?} occurs twice in the support projection")]
    DuplicateProperty { property: CityPropertyId },
    #[error("building {building} has support owned by both {first:?} and {second:?}")]
    DuplicateMember {
        building: u64,
        first: CityPropertyId,
        second: CityPropertyId,
    },
    #[error(
        "property {first:?} members {first_members:?} and property {second:?} members {second_members:?} overlap at {location_metres:?}: {area_square_metres} m² of support has two owners"
    )]
    OwnershipOverlap {
        first: CityPropertyId,
        first_members: Vec<u64>,
        second: CityPropertyId,
        second_members: Vec<u64>,
        location_metres: Vec2,
        area_square_metres: f64,
        first_region: CityPlotBounds,
        second_region: CityPlotBounds,
    },
    #[error("property support rejected: {0:?}")]
    Property(SupportDiagnostic),
}

impl BoundedSettlementTerrain {
    /// Canonical property order controls generated products, never their input
    /// iteration order. Overlapping owners are rejected before clipping; this
    /// operation does not choose the higher floor or merge nearby terraces.
    pub fn compile(
        plans: &[PropertySupportSurface],
        geographic: &GeographicSurface,
        embedment: FoundationEmbedment,
    ) -> Result<Self, SettlementSupportError> {
        let mut ordered: Vec<_> = plans.iter().collect();
        ordered.sort_by_key(|plan| plan.property_id());
        validate_owners(&ordered)?;
        let mut foundations = Vec::with_capacity(ordered.len());
        let mut outlines = Vec::new();
        let mut tolerance = 0.0_f32;
        for plan in ordered {
            foundations.push(
                plan.foundations(geographic, embedment)
                    .map_err(SettlementSupportError::Property)?,
            );
            outlines.extend(plan.clipping_outlines.clone());
            tolerance = tolerance.max(plan.limits.contact_tolerance_metres);
        }
        let cuts = geometry::SourceCutRegions::from_outlines(outlines);
        let natural_triangles = geographic
            .triangles
            .iter()
            .flat_map(|triangle| triangle.outside_regions(&cuts))
            .map(|triangle| triangle.points())
            .collect();
        let mut surface = Self {
            foundations,
            natural_triangles,
            contact_tolerance_metres: tolerance,
            query: SupportQueryIndex::default(),
        };
        surface.query = SupportQueryIndex::compile(&surface);
        Ok(surface)
    }

    /// Sorted support elevations, coalesced within the contact tolerance.
    /// Architectural consumers select their bound floor rather than averaging
    /// levels.
    pub fn elevations_at(
        &self,
        scene_point: crate::scene_coordinates::ScenePlanPoint,
    ) -> SurfaceElevations {
        let point = scene_point.metres();
        let mut heights: Vec<_> = self
            .query
            .triangles_at(self, point, self.contact_tolerance_metres)
            .map(|triangle| SupportElevation(triangle.height_at(point)))
            .collect();
        heights.sort_by(|a, b| a.metres().total_cmp(&b.metres()));
        heights.dedup_by(|a, b| (a.metres() - b.metres()).abs() <= self.contact_tolerance_metres);
        SurfaceElevations(heights)
    }

    /// Convex foundation compounds and the clipped source trimesh are separate
    /// static bodies. Avian/Parry does not support nesting composite colliders.
    pub fn colliders(&self) -> Vec<avian3d::prelude::Collider> {
        let mut colliders: Vec<_> = self
            .foundations
            .iter()
            .map(PropertyFoundationMesh::collider)
            .collect();
        if !self.natural_triangles.is_empty() {
            let triangles: Vec<_> = self
                .natural_triangles
                .iter()
                .chain(self.foundations.iter().flat_map(|f| f.cut_faces.iter()))
                .collect();
            let positions: Vec<_> = triangles.iter().flat_map(|t| t.iter()).copied().collect();
            let indices = (0..triangles.len())
                .map(|i| {
                    let start =
                        u32::try_from(i * 3).expect("bounded settlement source fits u32 indices");
                    if i < self.natural_triangles.len() {
                        [start, start + 2, start + 1]
                    } else {
                        [start, start + 1, start + 2]
                    }
                })
                .collect();
            colliders.push(avian3d::prelude::Collider::trimesh(positions, indices));
        }
        colliders
    }

    /// Source, bearings and intentionally buried foundation faces use the
    /// same vertices in presentation and collision. No render-only elevation
    /// offset hides unsupported construction.
    pub fn presentation_triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.natural_triangles
            .iter()
            .map(|[a, b, c]| [*a, *c, *b])
            .chain(
                self.foundations
                    .iter()
                    .flat_map(|f| f.cut_faces.iter().copied()),
            )
            .chain(self.foundations.iter().flat_map(|foundation| {
                foundation
                    .solid_triangles
                    .iter()
                    .map(|indices| indices.map(|i| foundation.positions[i as usize]))
            }))
    }

    /// Highest exterior support is appropriate for an unbound exterior query.
    /// At retaining edges, callers with an actor or architectural datum use
    /// `surface_below` or `elevations_at` instead.
    pub fn highest_surface_at(
        &self,
        scene_point: crate::scene_coordinates::ScenePlanPoint,
    ) -> Option<super::super::SurfaceHit> {
        let point = scene_point.metres();
        self.surface_below(Vec3::new(point.x, f32::INFINITY, point.y))
    }

    /// Select physical support below an explicit vertical query ceiling. The
    /// caller supplies its actor/threshold clearance; no movement limit is
    /// invented here. Buried prism bottoms and vertical faces are not bearings.
    pub fn surface_below(&self, position: Vec3) -> Option<super::super::SurfaceHit> {
        crate::scene_coordinates::ScenePlanPoint::from_metres(position.xz())?;
        if position.y.is_nan() {
            return None;
        }
        self.query
            .triangles_at(self, position.xz(), self.contact_tolerance_metres)
            .filter_map(|triangle| {
                let height = triangle.height_at(position.xz());
                if height > position.y + self.contact_tolerance_metres {
                    return None;
                }
                let [a, b, c] = triangle.points();
                let normal = (c - a).cross(b - a);
                super::super::SurfaceHit::from_geometry(height, normal)
            })
            .max_by(|a, b| a.elevation.metres().total_cmp(&b.elevation.metres()))
    }

    pub(crate) fn support_heights(&self) -> impl Iterator<Item = f32> + '_ {
        self.support_triangles()
            .flat_map(|points| points.map(|p| p.y))
    }

    pub(crate) fn support_triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.natural_triangles
            .iter()
            .copied()
            .chain(self.foundations.iter().flat_map(|foundation| {
                foundation
                    .support_triangles
                    .iter()
                    .map(|indices| indices.map(|i| foundation.positions[i as usize]))
            }))
    }
}

fn validate_owners(plans: &[&PropertySupportSurface]) -> Result<(), SettlementSupportError> {
    use crate::city_layout::grounding::planar::query::{PlanarBounds, PlanarQueryIndex};
    // Include complete source-clipping outlines, especially entry aprons
    // outside a plot. The broad phase only avoids disjoint owner pairs; exact
    // clipping below retains its canonical conflict order and diagnostics.
    let bounds: Vec<_> = plans
        .iter()
        .map(|plan| {
            PlanarBounds::from_points(plan.clipping_outlines.iter().flatten().copied())
                .expect("validated property support has nonempty clipping outlines")
        })
        .collect();
    let index = PlanarQueryIndex::from_bounds(bounds.clone());
    let mut properties = BTreeSet::new();
    let mut members = std::collections::BTreeMap::new();
    for (ordinal, plan) in plans.iter().enumerate() {
        let property = plan.property_id();
        if !properties.insert(property) {
            return Err(SettlementSupportError::DuplicateProperty { property });
        }
        for building in &plan.mesh.member_building_ids {
            if let Some(first) = members.insert(*building, property) {
                return Err(SettlementSupportError::DuplicateMember {
                    building: *building,
                    first,
                    second: property,
                });
            }
        }
        for earlier in index
            .intersections(bounds[ordinal])
            .into_iter()
            .filter(|earlier| *earlier < ordinal)
            .map(|earlier| plans[earlier])
        {
            for (first, first_outline) in earlier
                .support_regions()
                .iter()
                .zip(&earlier.clipping_outlines)
            {
                for (second, second_outline) in
                    plan.support_regions().iter().zip(&plan.clipping_outlines)
                {
                    if let Some((location_metres, area_square_metres)) =
                        overlap(*first, *second, first_outline, second_outline)
                    {
                        return Err(SettlementSupportError::OwnershipOverlap {
                            first: earlier.property_id(),
                            first_members: earlier.mesh.member_building_ids.clone(),
                            second: property,
                            second_members: plan.mesh.member_building_ids.clone(),
                            location_metres,
                            area_square_metres,
                            first_region: *first,
                            second_region: *second,
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

fn overlap(
    first: CityPlotBounds,
    second: CityPlotBounds,
    first_outline: &[bevy::math::DVec2],
    second_outline: &[bevy::math::DVec2],
) -> Option<(Vec2, f64)> {
    if !first.intersects(second) {
        return None;
    }
    let mut intersection = first_outline.to_vec();
    for i in 0..second_outline.len() {
        let a = second_outline[i];
        let edge = second_outline[(i + 1) % second_outline.len()] - a;
        intersection =
            crate::city_layout::grounding::planar::clip(intersection, |p| edge.perp_dot(p - a));
    }
    let area = crate::city_layout::grounding::planar::signed_area(&intersection).abs();
    (area > f64::EPSILON).then(|| {
        let location =
            intersection.iter().copied().sum::<bevy::math::DVec2>() / intersection.len() as f64;
        (location.as_vec2(), area)
    })
}

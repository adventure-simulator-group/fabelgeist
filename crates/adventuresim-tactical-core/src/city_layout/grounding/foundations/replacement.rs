//! Source triangles are replaced only inside the complete declared support cores.
use super::*;
#[cfg(test)]
mod tests;

/// The clipped natural surface and finite foundation share geometry between
/// spatial queries, collision and eventual presentation. Interior prism faces
/// remain construction geometry; collision uses solid convex cells.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BoundedPropertyTerrain {
    pub foundations: PropertyFoundationMesh,
    pub natural_triangles: Vec<[Vec3; 3]>,
    pub support_regions: Vec<CityPlotBounds>,
    contact_tolerance_metres: f32,
}

impl BoundedPropertyTerrain {
    pub fn compile(
        plan: &CompoundSupportPlan,
        geographic: &GeographicSurface,
        embedment: FoundationEmbedment,
    ) -> Result<Self, SupportDiagnostic> {
        let foundations = plan.foundations(geographic, embedment)?;
        let regions = plan.support_regions();
        let outlines = plan.source_clipping_outlines();
        let cuts = geometry::SourceCutRegions::from_outlines(outlines);
        let natural_triangles = geographic
            .triangles
            .iter()
            .flat_map(|triangle| triangle.outside_regions(&cuts))
            .map(|triangle| triangle.points())
            .collect();
        Ok(Self {
            foundations,
            natural_triangles,
            support_regions: regions,
            contact_tolerance_metres: plan.limits.contact_tolerance_metres,
        })
    }

    pub fn elevations_at(
        &self,
        scene_point: crate::scene_coordinates::ScenePlanPoint,
    ) -> SurfaceElevations {
        let point = scene_point.metres();
        let support = self
            .foundations
            .support_triangles
            .iter()
            .map(|indices| indices.map(|i| self.foundations.positions[i as usize]));
        let mut heights: Vec<_> = support
            .chain(self.natural_triangles.iter().copied())
            .filter_map(|points| {
                let triangle = GroundTriangle::new(points)?;
                triangle
                    .contains(point, self.contact_tolerance_metres)
                    .then(|| SupportElevation(triangle.height_at(point)))
            })
            .collect();
        heights.sort_by(|a, b| a.metres().total_cmp(&b.metres()));
        heights.dedup_by(|a, b| (a.metres() - b.metres()).abs() <= self.contact_tolerance_metres);
        SurfaceElevations(heights)
    }

    /// Avian composite colliders cannot be nested. Install these immutable
    /// shapes on separate static bodies with the same scene-space origin.
    pub fn colliders(&self) -> Vec<avian3d::prelude::Collider> {
        let foundation = self.foundations.collider();
        if self.natural_triangles.is_empty() {
            return vec![foundation];
        }
        let triangles: Vec<_> = self
            .natural_triangles
            .iter()
            .chain(&self.foundations.cut_faces)
            .collect();
        let positions: Vec<_> = triangles.iter().flat_map(|t| t.iter()).copied().collect();
        let indices = (0..triangles.len())
            .map(|i| {
                let start =
                    u32::try_from(i * 3).expect("bounded geographic mesh fits in u32 indices");
                if i < self.natural_triangles.len() {
                    [start, start + 2, start + 1]
                } else {
                    [start, start + 1, start + 2]
                }
            })
            .collect();
        let natural = avian3d::prelude::Collider::trimesh(positions, indices);
        vec![foundation, natural]
    }
}

impl CompoundSupportPlan {
    /// An explicitly bound apron shares the plot's front half-plane. Computing
    /// the same line independently through rounded rectangle centres can leave
    /// a micrometre-wide source triangle blocking the full doorway route.
    /// Only this property's declared join is welded; unrelated terraces and
    /// source heights are untouched. Double precision keeps the shared line
    /// until final presentation/collision vertices are emitted.
    pub(in crate::city_layout::grounding) fn source_clipping_outlines(
        &self,
    ) -> Vec<Vec<bevy::math::DVec2>> {
        let mut regions: Vec<_> = self
            .support_regions()
            .into_iter()
            .map(|region| region.corners().map(|point| point.as_dvec2()).to_vec())
            .collect();
        if self.street_entry.is_some() {
            let (a, b) = (regions[0][0], regions[0][1]);
            let edge = b - a;
            let apron = regions
                .last_mut()
                .expect("bound entry supplies its clipping region");
            for point in &mut apron[2..] {
                *point = a + edge * ((*point - a).dot(edge) / edge.length_squared());
            }
        }
        regions
    }
}

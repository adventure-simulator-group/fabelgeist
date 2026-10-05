//! Terrain queries, render meshes and colliders share owned support topology.
use super::*;
use crate::city_layout::grounding::{
    BoundedSettlementTerrain, GeographicSurface, SurfaceElevations,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Reflect)]
pub(super) enum TerrainGeometry {
    #[default]
    Sampled,
    Owned(BoundedSettlementTerrain),
}

impl SceneTerrain {
    /// Complete accepted support and stitched geographic rings. Presentation
    /// partitions this surface; raw vista samples must not be layered over it.
    pub fn property_surface(&self) -> Option<&BoundedSettlementTerrain> {
        match &self.geometry {
            TerrainGeometry::Sampled => None,
            TerrainGeometry::Owned(surface) => Some(surface),
        }
    }

    /// Install the producer's accepted topology without reconstructing its
    /// source from samples with a potentially different subdivision diagonal.
    /// The grid retains its extent and material sampling metadata. All physical
    /// queries, meshes and colliders use the compiled surface thereafter.
    /// Repairs/refinement precede installation; later grid rewrites are rejected.
    pub fn with_property_surface(mut self, surface: BoundedSettlementTerrain) -> Self {
        self.geometry = TerrainGeometry::Owned(surface);
        self
    }

    /// Exact triangle source, including the heightfield's subdivision diagonal.
    /// This deliberately does not use the vista grid's different triangulation.
    pub fn sampled_geographic_surface(&self) -> Option<GeographicSurface> {
        if matches!(self.geometry, TerrainGeometry::Owned(_)) {
            return None;
        }
        let (positions, indices, _) = self.mesh_components_with_stride_filtered(1, |_| true);
        let triangles = indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|indices| indices.map(|i| Vec3::from_array(positions[i as usize])));
        GeographicSurface::from_triangles(triangles)
    }

    /// Exact occupied support and clipped natural terrain, excluding buried
    /// foundation bottoms and vertical retaining faces from height comparisons.
    pub fn physical_geographic_surface(&self) -> Option<GeographicSurface> {
        match &self.geometry {
            TerrainGeometry::Sampled => self.sampled_geographic_surface(),
            TerrainGeometry::Owned(surface) => {
                GeographicSurface::from_triangles(surface.support_triangles())
            }
        }
    }

    /// Physical candidates at a retaining boundary remain distinct. A bound
    /// building uses its placement elevation; route/actor queries select their
    /// support using an explicit vertical ceiling.
    pub fn support_elevations_at(&self, point: Vec2) -> SurfaceElevations {
        match &self.geometry {
            TerrainGeometry::Sampled => self
                .height_at(point)
                .and_then(crate::city_layout::grounding::SupportElevation::from_metres)
                .map(SurfaceElevations::from_elevation)
                .unwrap_or_default(),
            TerrainGeometry::Owned(surface) => surface.elevations_at(point),
        }
    }

    pub fn surface_below(
        &self,
        position: Vec3,
    ) -> Option<crate::city_layout::grounding::SurfaceHit> {
        match &self.geometry {
            TerrainGeometry::Sampled => {
                let sample = self.surface_at_stride(Vec2::new(position.x, position.z), 1)?;
                (sample.elevation.metres() <= position.y).then_some(sample)
            }
            TerrainGeometry::Owned(surface) => surface.surface_below(position),
        }
    }

    pub(super) fn owned_mesh_components(
        &self,
        surface: &BoundedSettlementTerrain,
        keep_triangle: impl Fn(Vec2) -> bool,
    ) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 2]>) {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut uvs = Vec::new();
        for triangle in surface.presentation_triangles() {
            let centre = triangle.iter().copied().sum::<Vec3>() / 3.0;
            if !keep_triangle(Vec2::new(centre.x, centre.z)) {
                continue;
            }
            let start = u32::try_from(positions.len())
                .expect("bounded owned terrain fits u32 presentation indices");
            positions.extend(triangle.map(|point| point.to_array()));
            indices.extend([start, start + 1, start + 2]);
            uvs.extend(triangle.map(|p| [p.x / self.width() + 0.5, p.z / self.depth() + 0.5]));
        }
        (positions, indices, uvs)
    }

    /// Exact accepted owner. Enclosures must not select a neighbouring terrace
    /// or substitute a member building's floor for the property's gate landing.
    pub fn property_foundation(
        &self,
        property: crate::city_layout::CityPropertyId,
    ) -> Option<&crate::city_layout::grounding::PropertyFoundationMesh> {
        let TerrainGeometry::Owned(surface) = &self.geometry else {
            return None;
        };
        surface
            .foundations
            .iter()
            .find(|foundation| foundation.property_id == property)
    }

    pub(crate) fn foundation_colliders(&self) -> Vec<Collider> {
        match &self.geometry {
            TerrainGeometry::Sampled => Vec::new(),
            TerrainGeometry::Owned(surface) => surface
                .foundations
                .iter()
                .map(crate::city_layout::grounding::PropertyFoundationMesh::collider)
                .collect(),
        }
    }

    pub(crate) fn natural_collision_mesh_with_transition(
        &self,
        collar: crate::terrain_transition::TerrainTransitionCollar,
    ) -> (Vec<Vec3>, Vec<[u32; 3]>) {
        match &self.geometry {
            TerrainGeometry::Sampled => self.collider_mesh_with_transition(collar),
            TerrainGeometry::Owned(surface) => {
                let mut positions = Vec::new();
                let mut indices = Vec::new();
                for (triangle, winding) in surface
                    .natural_triangles
                    .iter()
                    .map(|t| (t, [0, 2, 1]))
                    .chain(
                        surface
                            .foundations
                            .iter()
                            .flat_map(|f| f.cut_faces.iter())
                            .map(|t| (t, [0, 1, 2])),
                    )
                {
                    let centre = triangle.iter().copied().sum::<Vec3>() / 3.0;
                    if collar.cuts_out(Vec2::new(centre.x, centre.z)) {
                        continue;
                    }
                    let start = u32::try_from(positions.len())
                        .expect("bounded natural terrain fits u32 collision indices");
                    positions.extend(triangle);
                    indices.push(winding.map(|i| start + i));
                }
                (positions, indices)
            }
        }
    }
}

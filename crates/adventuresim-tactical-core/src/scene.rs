use adventuresim_world_schema::BASIS_POINTS_PER_WHOLE;
use avian3d::prelude::*;
use bevy::prelude::*;
use noiz::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{city_layout::grounding::SupportElevation, scene_coordinates::ScenePlanPoint};
#[cfg(test)]
use crate::terrain_transition::TerrainTransitionCollar;

pub use admission::TerrainAdmissionError;
pub use grade::{TerrainGradeError, TerrainGradeLimit};
pub use ground::*;
use property_support::TerrainGeometry;
pub use queries::{SceneElevationRange, TerrainSupportError, TerrainSupportResult};

mod admission;
pub(crate) mod grade;
mod ground;
mod property_support;
mod queries;

/// Terrain generator shared by the authoritative server and deterministic
/// presentation fixtures. Keeping the implementation here prevents animation
/// captures from approximating a different surface than gameplay uses.
#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    pub seed: fabelgeist_determinism::Seed,
    pub period: f32,
    pub grid_scale: f32,
}

/// Id of the scene in which the game takes place.
#[derive(Component, Serialize, Deserialize, Default, Debug, Reflect, Clone, PartialEq, Eq)]
#[reflect(Component)]
#[component(immutable)]
pub struct SceneId(pub String);

#[derive(Component, Serialize, Deserialize, Debug, Reflect, Clone, PartialEq)]
#[reflect(opaque, Component)]
#[serde(try_from = "admission::TerrainWire")]
pub struct SceneTerrain {
    heightmap: Vec<f32>,
    width: usize,
    scale: f32,
    /// Number of authoritative sample intervals represented by one vertex in
    /// the coarse render LOD. Collision and spatial queries always use every
    /// sample; this stride only avoids submitting the full-resolution field
    /// outside the camera-local detail patch.
    coarse_stride: usize,
    /// Sampled ground or an explicitly compiled owned surface. This is a
    /// generated geometry contract, not a second tactical simulation state.
    geometry: TerrainGeometry,
}

impl TerrainGenerator {
    pub fn new(seed: fabelgeist_determinism::Seed) -> Self {
        Self {
            seed,
            period: 30.0,
            grid_scale: 1.0,
        }
    }

    pub fn generate(
        self,
        width: usize,
        height: usize,
        depth: usize,
    ) -> Result<SceneTerrain, TerrainAdmissionError> {
        let mut noise = Noise::from(LayeredNoise::new(
            Normed::<f32>::default(),
            Persistence(0.5),
            FractalLayers {
                layer: Octave::<MixCellGradients<OrthoGrid, Smoothstep, QuickGradients>>::default(),
                lacunarity: 2.0,
                amount: 8,
            },
        ));
        // Noiz owns a 32-bit native seed port; preserve the original low word.
        noise.set_seed(self.seed.to_u64() as u32);
        noise.set_period(self.period);

        SceneTerrain::new(width, depth, self.grid_scale, move |location| {
            let normal: f32 = noise.sample(location);
            normal * height as f32
        })
    }
}

impl SceneTerrain {
    /// Builds terrain from an already-authoritative row-major height sample.
    ///
    /// The dimensions count grid vertices, while `SceneTerrain::new` counts
    /// cells. This constructor is the scene-input boundary used by both the
    /// server collider and the client's replicated playable mesh.
    pub fn from_heightmap(
        grid_width: usize,
        grid_depth: usize,
        scale: f32,
        heightmap: Vec<f32>,
    ) -> Option<Self> {
        let terrain = Self {
            heightmap,
            width: grid_width,
            scale,
            coarse_stride: 1,
            geometry: TerrainGeometry::Sampled,
        };
        if terrain.grid_depth_checked()? != grid_depth {
            return None;
        }
        terrain.validate().ok()?;
        Some(terrain)
    }

    /// Native row-major sampling boundary; dimensions count cells and the
    /// callback returns metres. Admission completes before any mesh/query use.
    pub fn new(
        width: usize,
        depth: usize,
        scale: f32,
        op: impl Fn(Vec2) -> f32,
    ) -> Result<Self, TerrainAdmissionError> {
        let width = width
            .checked_add(1)
            .ok_or(TerrainAdmissionError::Dimensions)?;
        let depth = depth
            .checked_add(1)
            .ok_or(TerrainAdmissionError::Dimensions)?;
        let count = width
            .checked_mul(depth)
            .ok_or(TerrainAdmissionError::Dimensions)?;
        admission::validate_dimensions(width, depth, scale, 1)?;
        let heightmap = (0..count)
            .map(|i| op(Vec2::new((i % width) as f32, (i / width) as f32)))
            .collect();
        let terrain = Self {
            width,
            heightmap,
            scale,
            coarse_stride: 1,
            geometry: TerrainGeometry::Sampled,
        };
        terrain.validate()?;
        Ok(terrain)
    }

    /// Refines every coarse cell by an integer factor and evaluates one
    /// authoritative height function at the resulting vertices. The original
    /// grid remains implicit as the coarse render LOD through `coarse_stride`.
    pub fn refined(
        &self,
        target_spacing: f32,
        height_at: impl Fn(ScenePlanPoint, SupportElevation) -> Option<SupportElevation>,
    ) -> Option<Self> {
        if !matches!(self.geometry, TerrainGeometry::Sampled)
            || !target_spacing.is_finite()
            || target_spacing <= 0.0
        {
            return None;
        }
        let subdivisions = (self.scale / target_spacing).ceil().max(1.0) as usize;
        let cells_x = self
            .grid_width()
            .checked_sub(1)?
            .checked_mul(subdivisions)?;
        let cells_z = self
            .grid_depth()
            .checked_sub(1)?
            .checked_mul(subdivisions)?;
        let width = cells_x.checked_add(1)?;
        let depth = cells_z.checked_add(1)?;
        let scale = self.scale / subdivisions as f32;
        admission::validate_dimensions(width, depth, scale, subdivisions).ok()?;
        let half_extent = Vec2::new(self.width(), self.depth()) * 0.5;
        let mut heights = Vec::with_capacity(width.checked_mul(depth)?);
        for z in 0..depth {
            for x in 0..width {
                let point = ScenePlanPoint::from_metres(
                    Vec2::new(x as f32 * scale, z as f32 * scale) - half_extent,
                )?;
                let base_height = self.height_at(point)?;
                let height = height_at(point, base_height)?;
                heights.push(height.metres());
            }
        }
        let terrain = Self {
            heightmap: heights,
            width,
            scale,
            coarse_stride: subdivisions,
            geometry: TerrainGeometry::Sampled,
        };
        terrain.validate().ok()?;
        Some(terrain)
    }

    pub fn grid_width(&self) -> usize {
        self.width
    }

    pub fn grid_depth(&self) -> usize {
        self.heightmap.len() / self.width
    }

    pub fn width(&self) -> f32 {
        self.grid_width().saturating_sub(1) as f32 * self.scale
    }

    pub fn depth(&self) -> f32 {
        self.grid_depth().saturating_sub(1) as f32 * self.scale
    }

    pub fn grid_scale(&self) -> f32 {
        self.scale
    }

    pub fn coarse_grid_scale(&self) -> f32 {
        self.scale * self.coarse_stride.max(1) as f32
    }

    /// Constrain the actual rendered/collision triangle gradients. Already valid
    /// relief retains its bits; owned support is never rewritten by this repair.
    pub fn constrain_max_grade(
        &mut self,
        maximum_grade: grade::TerrainGradeLimit,
    ) -> Result<(), TerrainGradeError> {
        if !matches!(self.geometry, TerrainGeometry::Sampled) {
            return Err(TerrainGradeError::OwnedSurface);
        }
        let depth = self.grid_depth();
        grade::constrain(
            &mut self.heightmap,
            self.width,
            depth,
            self.scale,
            maximum_grade.ratio(),
        )
    }

    /// Rewrites authoritative samples in centered world coordinates while
    /// preserving the terrain dimensions and render/collider correspondence.
    /// Publish the replacement only after every sample is admitted.
    pub fn rewrite_heights(
        &mut self,
        mut rewrite: impl FnMut(ScenePlanPoint, SupportElevation) -> Option<SupportElevation>,
    ) -> bool {
        if !matches!(self.geometry, TerrainGeometry::Sampled) {
            return false;
        }
        let half_extent = Vec2::new(self.width(), self.depth()) * 0.5;
        let mut replacement = Vec::with_capacity(self.heightmap.len());
        for z in 0..self.grid_depth() {
            for x in 0..self.grid_width() {
                let index = z * self.grid_width() + x;
                let Some(point) = ScenePlanPoint::from_metres(
                    Vec2::new(x as f32, z as f32) * self.scale - half_extent,
                ) else {
                    return false;
                };
                let Some(height) = SupportElevation::from_metres(self.heightmap[index])
                    .and_then(|height| rewrite(point, height))
                else {
                    return false;
                };
                replacement.push(height.metres());
            }
        }
        self.heightmap = replacement;
        true
    }

    /// Install each shape on its own static body. Owned foundation compounds
    /// and the natural trimesh cannot be nested in one composite collider.
    pub fn colliders(
        &self,
    ) -> Result<Vec<Collider>, crate::city_layout::grounding::SupportColliderError> {
        match &self.geometry {
            TerrainGeometry::Sampled => Ok(vec![Collider::heightfield(
                self.collider_height_matrix(),
                Vec3::new(self.width(), 1.0, self.depth()),
            )]),
            TerrainGeometry::Owned(surface) => surface.colliders(),
        }
    }

    fn collider_height_matrix(&self) -> Vec<Vec<f32>> {
        // Avian flattens this nested Vec directly into Parry's column-major
        // Array2. Supply X-major values while retaining Z rows and X columns
        // as the matrix dimensions expected by Parry's heightfield.
        let column_major = (0..self.grid_width())
            .flat_map(|x| {
                (0..self.grid_depth()).map(move |z| self.heightmap[x + z * self.grid_width()])
            })
            .collect::<Vec<_>>();
        column_major
            .chunks_exact(self.grid_width())
            .map(|row| row.to_vec())
            .collect()
    }

    #[cfg(feature = "meshgen")]
    pub fn mesh(&self) -> Mesh {
        self.mesh_with_stride(1)
    }

    #[cfg(feature = "meshgen")]
    pub fn coarse_mesh(&self) -> Mesh {
        self.mesh_with_stride(self.coarse_stride.max(1))
    }

    #[cfg(feature = "meshgen")]
    fn mesh_with_stride(&self, stride: usize) -> Mesh {
        let (positions, indices, uvs) = self.mesh_components_with_stride(stride);

        self.mesh_from_components(positions, indices, uvs)
    }

    #[cfg(test)]
    fn mesh_components(&self) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 2]>) {
        self.mesh_components_with_stride(1)
    }

    #[cfg(any(feature = "meshgen", test))]
    fn mesh_components_with_stride(
        &self,
        stride: usize,
    ) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 2]>) {
        self.mesh_components_with_stride_filtered(stride, |_| true)
    }

    pub(crate) fn mesh_components_with_stride_filtered(
        &self,
        stride: usize,
        keep_cell: impl Fn(Vec2) -> bool,
    ) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 2]>) {
        if let TerrainGeometry::Owned(surface) = &self.geometry {
            return self.owned_mesh_components(surface, keep_cell);
        }
        let stride = stride.max(1);
        let xs = (0..self.grid_width()).step_by(stride).collect::<Vec<_>>();
        let zs = (0..self.grid_depth()).step_by(stride).collect::<Vec<_>>();
        debug_assert_eq!(xs.last().copied(), Some(self.grid_width() - 1));
        debug_assert_eq!(zs.last().copied(), Some(self.grid_depth() - 1));
        let mesh_offset = Vec2::new(
            self.grid_width().saturating_sub(1) as f32,
            self.grid_depth().saturating_sub(1) as f32,
        ) * -0.5;

        let mut positions = Vec::with_capacity(xs.len() * zs.len());
        let mut uvs = Vec::with_capacity(xs.len() * zs.len());
        // Keep vertices in the same row-major (z * width + x) order used by
        // height sampling and the triangle indices below. Iterating x first
        // transposed the storage without transposing the indices, which made
        // otherwise smooth terrain render as long shredded triangles.
        for &z in &zs {
            for &x in &xs {
                let i = z * self.grid_width() + x;
                let y = self.heightmap[i];

                uvs.push([
                    x as f32 / (self.grid_width() - 1) as f32,
                    z as f32 / (self.grid_depth() - 1) as f32,
                ]);

                let x = (x as f32 + mesh_offset.x) * self.scale;
                let z = (z as f32 + mesh_offset.y) * self.scale;
                positions.push([x, y, z]);
            }
        }

        let mut indices = Vec::with_capacity((xs.len() - 1) * (zs.len() - 1) * 6);
        for x in 0..xs.len() - 1 {
            for z in 0..zs.len() - 1 {
                let i = (z * xs.len() + x) as u32;
                let cell_triangles = [
                    [i, i + xs.len() as u32, i + 1],
                    [i + 1, i + xs.len() as u32, i + xs.len() as u32 + 1],
                ];
                for triangle in cell_triangles {
                    let centre = triangle
                        .iter()
                        .map(|&index| {
                            let position = positions[index as usize];
                            Vec2::new(position[0], position[2])
                        })
                        .sum::<Vec2>()
                        / 3.0;
                    if keep_cell(centre) {
                        indices.extend_from_slice(&triangle);
                    }
                }
            }
        }

        (positions, indices, uvs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_collar_removes_only_the_heightfield_interior() {
        let terrain = SceneTerrain::new(10, 10, 1.0, |_| 0.0).unwrap();
        let collar =
            TerrainTransitionCollar::irregular_ellipse(crate::prelude::TerrainCollarParameters {
                origin: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::ZERO).unwrap(),
                tangent:
                    adventuresim_building_generator::spatial_geometry::PlanDirection::from_vector(
                        Vec2::X,
                    )
                    .unwrap(),
                half_length:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        3.0,
                    )
                    .unwrap(),
                half_width:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        3.0,
                    )
                    .unwrap(),
                width:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        1.0,
                    )
                    .unwrap(),
                seed: 0.into(),
                wander: crate::prelude::RuptureWander::from_metres(0.0).unwrap(),
                width_variation: crate::prelude::CollarWidthVariation::from_basis_points(0)
                    .unwrap(),
            })
            .unwrap();
        let (_, full_indices, _) = terrain.mesh_components_with_stride(1);
        let (positions, cut_indices, _) =
            terrain.mesh_components_with_stride_and_transition(1, collar);

        assert!(cut_indices.len() < full_indices.len());
        for triangle in cut_indices.as_chunks::<3>().0 {
            let centre = triangle
                .iter()
                .map(|&index| Vec3::from_array(positions[index as usize]))
                .sum::<Vec3>()
                / 3.0;
            assert!(
                !crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(centre.x, centre.z))
                    .is_ok_and(|point| collar.cuts_out(point))
            );
        }
        assert!(
            crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::ZERO)
                .is_ok_and(|point| collar.cuts_out(point))
        );
        assert!(
            crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(2.5, 0.0))
                .is_ok_and(|point| collar.cuts_out(point))
        );
        assert!(
            !crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(3.0, 0.0))
                .is_ok_and(|point| collar.cuts_out(point))
        );
        assert!(
            crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(2.5, 0.0))
                .is_ok_and(|point| collar.contains(point))
        );
        assert!(
            !crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::splat(2.5))
                .is_ok_and(|point| collar.contains(point))
        );
    }


    #[test]
    fn refined_surface_keeps_a_coarse_lod_without_splitting_query_authority() {
        let coarse = SceneTerrain::new(4, 4, 1.0, |_| 0.0).unwrap();
        let refined = coarse
            .refined(0.5, |point, base| {
                let point = point.metres();
                SupportElevation::from_metres(
                    base.metres() + (point.x * core::f32::consts::PI).sin()
                        * (point.y * core::f32::consts::PI).sin()
                        * 0.04,
                )
            })
            .unwrap();

        assert_eq!(refined.grid_scale(), 0.5);
        assert_eq!(refined.coarse_grid_scale(), 1.0);
        let point = ScenePlanPoint::from_metres(Vec2::splat(0.5)).unwrap();
        assert!((refined.height_at(point).unwrap().metres() - 0.04).abs() < 0.000_01);
        assert!(refined.coarse_height_at(point).unwrap().metres().abs() < 0.000_01);
    }

    #[test]
    fn mesh_vertices_follow_heightmap_row_major_order() {
        let terrain = SceneTerrain::new(2, 2, 1.0, |point| point.x + point.y * 10.0).unwrap();
        let (positions, indices, _) = terrain.mesh_components();

        assert_eq!(positions[0][1], 0.0);
        assert_eq!(positions[1][1], 1.0);
        assert_eq!(positions[2][1], 2.0);
        assert_eq!(positions[3][1], 10.0);

        let a = Vec3::from_array(positions[indices[0] as usize]);
        let b = Vec3::from_array(positions[indices[1] as usize]);
        let c = Vec3::from_array(positions[indices[2] as usize]);
        assert!((b - a).cross(c - a).y > 0.0);
    }

    #[test]
    fn collider_heightfield_matches_the_rendered_surface_without_transposing_axes() {
        let terrain =
            SceneTerrain::from_heightmap(3, 2, 1.0, vec![0.0, 1.0, 4.0, 10.0, 13.0, 20.0]).unwrap();
        assert_eq!(
            terrain.collider_height_matrix(),
            vec![vec![0.0, 10.0, 1.0], vec![13.0, 4.0, 20.0]]
        );

        let collider = terrain.colliders().unwrap().pop().unwrap();
        for point in [
            Vec2::new(-0.6, -0.2),
            Vec2::new(0.6, -0.2),
            Vec2::new(-0.6, 0.2),
            Vec2::new(0.6, 0.2),
            Vec2::ZERO,
        ] {
            let expected_height = terrain.height_at(ScenePlanPoint::from_metres(point).unwrap()).unwrap().metres();
            let ray_origin = Vec3::new(point.x, 30.0, point.y);
            let (distance, _) = collider
                .cast_ray(
                    Vec3::ZERO,
                    Rotation::default(),
                    ray_origin,
                    Vec3::NEG_Y,
                    60.0,
                    false,
                )
                .expect("the downward ray should hit the terrain collider");
            let collider_height = ray_origin.y - distance;

            assert!(
                (collider_height - expected_height).abs() < 0.0001,
                "collider {collider_height} != sampled surface {expected_height} at {point}"
            );
        }
    }


}

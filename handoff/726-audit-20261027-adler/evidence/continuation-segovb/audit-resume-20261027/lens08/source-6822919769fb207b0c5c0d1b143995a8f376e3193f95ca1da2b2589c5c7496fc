//! One source retains fine playable triangles and the existing vista seams.
use super::*;
use crate::{
    scene::SceneTerrain,
    scene_input::{VistaLod, VistaSample},
    vista_surface::{
        cell_rectangles_outside_inner_rectangle, presented_vista_vertex_height,
        subdivide_playable_boundary_rectangle,
    },
};

impl GeographicSurface {
    /// Exact current presentation surface before owned grading. Fine playable
    /// triangles are retained, not resampled onto a vista grid. Each centered
    /// vista ring excludes the preceding rectangle and uses the renderer's
    /// subdivision, height morph and playable-edge stitching policy.
    /// Malformed, offset or nonexpanding rings are rejected explicitly.
    pub fn from_presented_scene(terrain: &SceneTerrain, vista: &VistaSample) -> Option<Self> {
        let playable = terrain.sampled_geographic_surface()?;
        let mut triangles: Vec<_> = playable.triangles().collect();
        let mut inner_half_extent = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        for lod in &vista.lods {
            let half_extent = Vec2::new(
                f32::from(lod.width.checked_sub(1)?),
                f32::from(lod.depth.checked_sub(1)?),
            ) * lod.spacing_metres
                * 0.5;
            if lod.width < 2
                || lod.depth < 2
                || lod.heights_metres.len() != usize::from(lod.width) * usize::from(lod.depth)
                || !lod.spacing_metres.is_finite()
                || lod.spacing_metres <= 0.0
                || lod.origin_east_metres != 0.0
                || lod.origin_north_metres != 0.0
                || half_extent.cmple(inner_half_extent).any()
            {
                return None;
            }
            inner_half_extent = half_extent;
        }
        inner_half_extent = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        for (index, lod) in vista.lods.iter().enumerate() {
            append_ring(
                &mut triangles,
                lod,
                vista.lods.get(index + 1),
                (index == 0).then_some(terrain),
                inner_half_extent,
            )?;
            inner_half_extent = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        Self::from_triangles(triangles)
    }
}

fn append_ring(
    triangles: &mut Vec<[Vec3; 3]>,
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    terrain: Option<&SceneTerrain>,
    inner_half_extent: Vec2,
) -> Option<()> {
    let size = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1));
    for z in 0..usize::from(lod.depth - 1) {
        for x in 0..usize::from(lod.width - 1) {
            let minimum = (Vec2::new(x as f32, z as f32) - size * 0.5) * lod.spacing_metres;
            let maximum = minimum + Vec2::splat(lod.spacing_metres);
            for rectangle in
                cell_rectangles_outside_inner_rectangle(minimum, maximum, inner_half_extent)
            {
                for [x0, x1, z0, z1] in
                    subdivide_playable_boundary_rectangle(rectangle, inner_half_extent, terrain)
                {
                    let vertex = |x, z| {
                        let point = Vec2::new(x, z);
                        Some(Vec3::new(
                            x,
                            presented_vista_vertex_height(
                                lod,
                                coarser,
                                terrain,
                                point,
                                inner_half_extent,
                            )?,
                            z,
                        ))
                    };
                    let [a, b, c, d] = [
                        vertex(x0, z0)?,
                        vertex(x1, z0)?,
                        vertex(x1, z1)?,
                        vertex(x0, z1)?,
                    ];
                    triangles.extend([[a, b, c], [a, c, d]]);
                }
            }
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vista_surface::vista_triangle_height;

    fn vista() -> VistaSample {
        VistaSample {
            lods: [(0, 2.0, 4.0), (1, 4.0, 8.0)]
                .map(|(level, spacing, elevation)| VistaLod {
                    level: crate::scene_input::VistaLevelIndex::new(level),
                    spacing_metres: spacing,
                    width: 5,
                    depth: 5,
                    origin_east_metres: 0.0,
                    origin_north_metres: 0.0,
                    heights_metres: vec![elevation; 25],
                    environment: vec![],
                })
                .to_vec(),
        }
    }

    #[test]
    fn composed_source_retains_fine_mesh_and_exact_presented_seam_with_complete_coverage() {
        let terrain = SceneTerrain::from_heightmap(
            3,
            3,
            1.0,
            vec![0.0, 1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 1.0, 0.0],
        )
        .unwrap();
        let vista = vista();
        let source = GeographicSurface::from_presented_scene(&terrain, &vista).unwrap();
        let fine = terrain.sampled_geographic_surface().unwrap();
        let outline = [
            Vec2::splat(-1.0),
            Vec2::new(1.0, -1.0),
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
        ];
        let comparison = source
            .compare_in_outline(
                &fine,
                &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                    (outline)
                        .iter()
                        .copied()
                        .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(comparison.minimum.difference_metres.metres(), 0.0);
        assert_eq!(comparison.maximum.difference_metres.metres(), 0.0);
        assert_eq!(comparison.covered_area_square_metres.square_metres(), 4.0);
        for point in [
            Vec2::new(1.0, 0.37),
            Vec2::new(1.2, -0.3),
            Vec2::new(-0.4, 1.6),
            Vec2::new(3.1, 2.4),
        ] {
            let expected =
                vista_triangle_height(&vista.lods[0], Some(&vista.lods[1]), &terrain, point)
                    .unwrap();
            assert!(
                (source
                    .elevation_at(
                        crate::scene_coordinates::ScenePlanPoint::try_from(point).unwrap()
                    )
                    .unwrap()
                    .metres()
                    - expected)
                    .abs()
                    < 1e-6
            );
        }
        let constant = GeographicSurface::from_triangles([
            [
                Vec3::new(-1.4, 0.0, -1.4),
                Vec3::new(1.4, 0.0, -1.4),
                Vec3::new(1.4, 0.0, 1.4),
            ],
            [
                Vec3::new(-1.4, 0.0, -1.4),
                Vec3::new(1.4, 0.0, 1.4),
                Vec3::new(-1.4, 0.0, 1.4),
            ],
        ])
        .unwrap();
        let across = [
            Vec2::splat(-1.4),
            Vec2::new(1.4, -1.4),
            Vec2::splat(1.4),
            Vec2::new(-1.4, 1.4),
        ];
        let comparison = source
            .compare_in_outline(
                &constant,
                &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                    (across)
                        .iter()
                        .copied()
                        .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert!(
            (comparison.covered_area_square_metres.square_metres()
                - comparison.required_area_square_metres.square_metres())
            .abs()
                < 1e-8
        );
        assert_eq!(
            source
                .elevation_at(
                    crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::splat(7.0)).unwrap()
                )
                .unwrap()
                .metres(),
            8.0
        );
    }

    #[test]
    fn source_rejects_offset_and_nonexpanding_rings_instead_of_substituting_another_surface() {
        let terrain = SceneTerrain::from_heightmap(3, 3, 1.0, vec![0.0; 9]).unwrap();
        let mut input = vista();
        input.lods[0].origin_east_metres = 1.0;
        assert!(GeographicSurface::from_presented_scene(&terrain, &input).is_none());
        input = vista();
        input.lods[1].spacing_metres = input.lods[0].spacing_metres;
        assert!(GeographicSurface::from_presented_scene(&terrain, &input).is_none());
    }
}

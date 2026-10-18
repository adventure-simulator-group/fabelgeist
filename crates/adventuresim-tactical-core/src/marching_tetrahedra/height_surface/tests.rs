use super::*;
use crate::scene_coordinates::ScenePlanPoint;
use crate::terrain_transition::{CollarWidthVariation, RuptureWander, TerrainCollarParameters};
use adventuresim_building_generator::spatial_geometry::{PlanDirection, PositiveLength};
use bevy::prelude::Vec2;

fn collar() -> TerrainTransitionCollar {
    TerrainTransitionCollar::irregular_ellipse(TerrainCollarParameters {
        origin: ScenePlanPoint::try_from(Vec2::ZERO).unwrap(),
        tangent: PlanDirection::from_vector(Vec2::X).unwrap(),
        half_length: PositiveLength::from_metres(4.0).unwrap(),
        half_width: PositiveLength::from_metres(3.0).unwrap(),
        width: PositiveLength::from_metres(1.0).unwrap(),
        seed: 17.into(),
        wander: RuptureWander::from_metres(0.0).unwrap(),
        width_variation: CollarWidthVariation::from_basis_points(0).unwrap(),
    })
    .unwrap()
}

#[test]
fn trimmed_height_extraction_preserves_ordered_mesh_buffers() {
    let dimensions = [8, 13, 7];
    let position_at = |[x, y, z]: [usize; 3]| {
        Vec3::new(
            x as f32 * 0.5 - 2.0,
            y as f32 * 0.25 - 1.0,
            z as f32 * 0.5 - 2.0,
        )
    };
    for fixture in 0..6 {
        let heights = (0..dimensions[0] * dimensions[2])
            .map(|column| {
                let x = column % dimensions[0];
                let z = column / dimensions[0];
                match fixture {
                    0 => 0.0,
                    1 => x as f32 * 0.25 - 0.5,
                    2 => {
                        if x < 4 {
                            -2.0
                        } else {
                            3.0
                        }
                    }
                    3 => (x as f32 * 0.7).sin() + (z as f32 * 0.4).cos() * 0.3,
                    4 => {
                        if (x + z) % 2 == 0 {
                            0.000009
                        } else {
                            -0.000011
                        }
                    }
                    _ => x as f32 * 0.25 + z as f32 * 0.125 - 0.5,
                }
            })
            .collect::<Vec<_>>();
        let dense = marching_tetrahedra(
            dimensions,
            position_at,
            |point| {
                let x = ((point.x + 2.0) * 2.0) as usize;
                let z = ((point.z + 2.0) * 2.0) as usize;
                point.y - heights[z * dimensions[0] + x]
            },
            collar(),
        )
        .unwrap();
        let trimmed = marching_height_surface(dimensions, position_at, &heights, collar()).unwrap();
        assert_eq!(trimmed, dense, "height fixture {fixture}");
    }
}

#[test]
fn trimmed_height_extraction_rejects_nonfinite_columns() {
    let result = marching_height_surface(
        [2, 2, 2],
        |[x, y, z]| Vec3::new(x as f32, y as f32, z as f32),
        &[0.5, f32::NAN, 0.5, 0.5],
        collar(),
    );
    assert!(matches!(result, Err(TerrainRecipeError::NonFiniteField)));
    let overflow = marching_height_surface(
        [2, 2, 2],
        |[x, y, z]| Vec3::new(x as f32, y as f32 * f32::MAX, z as f32),
        &[-f32::MAX; 4],
        collar(),
    );
    assert!(matches!(overflow, Err(TerrainRecipeError::NonFiniteField)));
}

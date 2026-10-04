use fabelgeist_determinism::StreamId;

use crate::{
    city_layout::{CityStreetPatch, CityYardPatch},
    scene::{GroundCover, GroundSubstrate, SceneGround, SceneTerrain},
    scene_input::{
        EnvironmentalSample, GeneratedBuilding, GeneratedObstacle, SceneInputError,
        TREE_CANOPY_GROUND_RADIUS_METRES, TREE_DENSE_LEAF_LITTER_RADIUS_METRES,
        base_ground_surface,
    },
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_scene_ground(
    width: usize,
    depth: usize,
    spacing: f32,
    environment: &[EnvironmentalSample],
    terrain: &SceneTerrain,
    obstacles: &[GeneratedObstacle],
    obstacle_spacing: f32,
    buildings: &[GeneratedBuilding],
    streets: &[CityStreetPatch],
    yards: &[CityYardPatch],
) -> Result<SceneGround, SceneInputError> {
    let mut samples = environment
        .iter()
        .copied()
        .map(base_ground_surface)
        .collect::<Vec<_>>();
    let half_width = terrain.width() * 0.5;
    let half_depth = terrain.depth() * 0.5;
    for obstacle in obstacles {
        let GeneratedObstacle::Tree { x, z } = *obstacle else {
            continue;
        };
        let tree = bevy::math::Vec2::new(
            f32::from(x) * obstacle_spacing - half_width,
            f32::from(z) * obstacle_spacing - half_depth,
        );
        for sample_z in 0..depth {
            for sample_x in 0..width {
                let position = bevy::math::Vec2::new(
                    sample_x as f32 * spacing - half_width,
                    sample_z as f32 * spacing - half_depth,
                );
                let distance = position.distance(tree);
                if distance > TREE_CANOPY_GROUND_RADIUS_METRES {
                    continue;
                }
                let sample = &mut samples[sample_z * width + sample_x];
                if matches!(
                    sample.substrate,
                    GroundSubstrate::Water | GroundSubstrate::Road
                ) {
                    continue;
                }
                let litter_roll = StreamId::new("terrain.tree-leaf-litter")
                    .rng(
                        0,
                        &[u64::from(x), u64::from(z), sample_x as u64, sample_z as u64],
                    )
                    .unit_f32();
                if distance <= TREE_DENSE_LEAF_LITTER_RADIUS_METRES
                    || litter_roll < tree_leaf_litter_probability(distance)
                {
                    sample.cover = GroundCover::LeafLitter;
                    sample.cover_density_bps = 9_200;
                    sample.cover_height_cm = 6;
                }
            }
        }
    }
    let mut ground =
        SceneGround::from_samples(width, depth, spacing, samples).ok_or_else(|| {
            SceneInputError::Validation("generated ground-surface grid is invalid".into())
        })?;
    ground.urban = crate::scene::UrbanGroundSurfaces::new(streets, yards, buildings);
    Ok(ground)
}

pub(crate) fn tree_leaf_litter_probability(distance_metres: f32) -> f32 {
    if distance_metres <= TREE_DENSE_LEAF_LITTER_RADIUS_METRES {
        return 1.0;
    }
    let crown_fraction = ((distance_metres - TREE_DENSE_LEAF_LITTER_RADIUS_METRES)
        / (TREE_CANOPY_GROUND_RADIUS_METRES - TREE_DENSE_LEAF_LITTER_RADIUS_METRES))
        .clamp(0.0, 1.0);
    0.12 + (1.0 - crown_fraction).powf(1.5) * 0.60
}

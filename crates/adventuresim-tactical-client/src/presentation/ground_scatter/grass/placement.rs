//! Terrain cover, slope and deterministic jitter admission for grass cells.
use super::streams;
use adventuresim_tactical_core::{
    prelude::{GroundCover, SceneGround, SceneTerrain},
    scene_coordinates::ScenePlanPoint,
};
use bevy::prelude::{Quat, Transform, Vec2, Vec3};

/// Applies the canonical render-centre cover and slope gates for one placement
/// cell in the instanced renderer.
pub(in crate::presentation::ground_scatter) fn cell_allows_grass(
    terrain: &SceneTerrain,
    ground: &SceneGround,
    cell_hash: fabelgeist_determinism::Seed,
    x: i32,
    z: i32,
    cell_spacing: f32,
    jitter_fraction: f32,
) -> bool {
    let jitter_x = streams::JITTER_X.rng(cell_hash, &[]).inclusive_unit_f32() - 0.5;
    let jitter_z = streams::JITTER_Z.rng(cell_hash, &[]).inclusive_unit_f32() - 0.5;
    let render_centre = Vec2::new(
        (x as f32 + jitter_x * jitter_fraction) * cell_spacing,
        (z as f32 + jitter_z * jitter_fraction) * cell_spacing,
    );
    ScenePlanPoint::from_metres(render_centre)
        .and_then(|centre| {
            grass_patch_placement_with_spacing(terrain, ground, centre, cell_spacing)
        })
        .is_some()
}

fn ground_allows_grass_patch_with_spacing(
    ground: &SceneGround,
    centre: ScenePlanPoint,
    patch_spacing: f32,
) -> bool {
    let half_extent = patch_spacing * 0.58;
    [-1.0, 0.0, 1.0].into_iter().any(|z| {
        [-1.0, 0.0, 1.0].into_iter().any(|x| {
            ScenePlanPoint::from_metres(centre.metres() + Vec2::new(x, z) * half_extent)
                .and_then(|point| ground.ground_at(point))
                .is_some_and(|sample| sample.cover == GroundCover::TallGrass)
        })
    })
}

fn grass_patch_transform(terrain: &SceneTerrain, point: ScenePlanPoint) -> Option<Transform> {
    let height = terrain.height_at(point)?;
    let normal = terrain.normal_at(point)?;
    if normal.y < 0.72 {
        return None;
    }
    Some(
        Transform::from_xyz(point.metres().x, height.metres(), point.metres().y)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, *normal)),
    )
}

fn grass_patch_placement_with_spacing(
    terrain: &SceneTerrain,
    ground: &SceneGround,
    render_centre: ScenePlanPoint,
    patch_spacing: f32,
) -> Option<Transform> {
    if !ground_allows_grass_patch_with_spacing(ground, render_centre, patch_spacing) {
        return None;
    }
    grass_patch_transform(terrain, render_centre)
}

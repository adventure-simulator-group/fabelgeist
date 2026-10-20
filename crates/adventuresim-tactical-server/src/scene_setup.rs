use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::prelude::SceneVistaBundle;
use bevy::prelude::*;

use super::{furniture, openings, scene_obstacles, terrain_collision};

pub(crate) struct SceneGeometryPlugin;

impl Plugin for SceneGeometryPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "grown-trees")]
        app.add_plugins(super::scene_tree_assets::SceneTreeAssetsPlugin);
        app.add_observer(super::terrain_collision::on_scene_terrain_added)
            .add_observer(super::openings::on_scene_building_added)
            .add_observer(super::openings::on_scene_boundary_added)
            .add_observer(super::furniture::on_furniture_added)
            .add_observer(super::furniture::on_group_added)
            .add_observer(super::furniture::on_vista_furniture_added);
    }
}

pub(crate) fn vista_bundle(input: &TacticalSceneInput) -> Option<SceneVistaBundle> {
    Some(SceneVistaBundle {
        properties: input.properties.clone(),
        scene_digest: input.digest().expect("loaded scene input was validated"),
        playable_half_extent_metres: Vec2::new(
            f32::from(input.playable.width.saturating_sub(1)) * input.playable.spacing_metres * 0.5,
            f32::from(input.playable.depth.saturating_sub(1)) * input.playable.spacing_metres * 0.5,
        ),
        distant_buildings: input.distant_buildings.clone(),
        establishments: input.establishments.clone(),
        streets: input.streets.clone(),
        yards: input.yards.clone(),
        parishes: input.parishes.clone(),
        compounds: input.compounds.clone(),
        gardens: input.gardens.clone(),
        furniture_groups: Vec::new(),
        distant_furniture: Vec::new(),
        lods: input.vista.lods.clone(),
    })
}

/// One server-owned scene entity binds the unchanged physical obstacle batch.
pub(crate) fn spawn_generated(
    commands: &mut Commands,
    input: &TacticalSceneInput,
    content: SceneTreeContent,
) -> Result {
    info!("Creating a game scene for {}", input.scene_key);
    #[cfg(feature = "grown-trees")]
    let growth = if content == SceneTreeContent::Grown {
        let expected = input.environment_snapshot(input.digest()?);
        Some(super::scene_tree_assets::SceneTreeAssets::new(
            input, &expected,
        )?)
    } else {
        None
    };
    let generated = input.generate()?;
    info!(
        scene_digest = %generated.digest,
        schema_version = input.schema_version,
        generation_version = input.generation_version,
        source = ?input.source,
        obstacles = generated.obstacles.len(),
        upsampled_height_samples = generated.repairs.upsampled_height_samples,
        microrelief_adjusted_samples = generated.repairs.microrelief_adjusted_samples,
        adjusted_height_samples = generated.repairs.adjusted_height_samples,
        repaired_water_samples = generated.repairs.repaired_water_samples,
        removed_corridor_obstacles = generated.repairs.removed_corridor_obstacles,
        levelled_building_samples = generated.repairs.levelled_building_samples,
        removed_building_obstacles = generated.repairs.removed_building_obstacles,
        "Loaded deterministic tactical scene input"
    );
    let census = SceneObstacleCensus::from_generated(&generated.obstacles)?;
    let environment = input.environment_snapshot(generated.digest);
    content.validate_environment(&environment)?;
    #[cfg(feature = "grown-trees")]
    if let Some(assets) = &growth {
        assets.verify_owner(&SceneId(input.scene_key.clone()), &environment)?;
    }
    let scene = commands.spawn((content, census)).id();
    #[cfg(feature = "grown-trees")]
    if let Some(assets) = growth {
        commands.entity(scene).insert(assets);
    }
    scene_obstacles::spawn(
        commands,
        scene,
        generated.obstacles,
        &generated.terrain,
        input.playable.spacing_metres,
    );
    openings::spawn_generated_buildings(commands, generated.buildings, &input.establishments)?;
    openings::spawn_generated_boundaries(commands, generated.boundaries);
    for garden in generated.gardens {
        commands.spawn((garden, Transform::default()));
    }
    furniture::spawn(commands, generated.furniture);
    terrain_collision::populate_scene(
        commands.entity(scene),
        input.scene_key.clone(),
        generated.terrain,
        generated.ground,
        environment,
        generated.terrain_patch.as_ref(),
        input.landform,
    );
    Ok(())
}

pub(crate) fn spawn_world_bounds(commands: &mut Commands, width: f32, depth: f32) {
    commands.spawn((
        RigidBody::Static,
        CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
        Transform::default(),
        children![
            (
                Collider::half_space(Vec3::X),
                Transform::from_xyz(-width * 0.5, 0.0, 0.0)
            ),
            (
                Collider::half_space(Vec3::NEG_X),
                Transform::from_xyz(width * 0.5, 0.0, 0.0)
            ),
            (
                Collider::half_space(Vec3::Z),
                Transform::from_xyz(0.0, 0.0, -depth * 0.5)
            ),
            (
                Collider::half_space(Vec3::NEG_Z),
                Transform::from_xyz(0.0, 0.0, depth * 0.5)
            )
        ],
    ));
}

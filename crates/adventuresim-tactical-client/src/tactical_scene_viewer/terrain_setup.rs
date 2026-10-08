use super::*;

pub(super) fn spawn(
    commands: &mut Commands,
    input: &TacticalSceneInput,
    environment: SceneEnvironment,
    ground: SceneGround,
    terrain: SceneTerrain,
    terrain_patch: Option<&SceneTerrainPatch>,
) -> Result {
    let colliders = terrain_patch.map_or_else(
        || terrain.colliders(),
        |patch| patch.colliders_with_terrain(&terrain),
    )?;
    let mut terrain_entity = commands.spawn((
        Name::new("Captured tactical terrain"),
        SceneId(input.scene_key.clone()),
        environment,
        ground,
        terrain,
        Transform::default(),
    ));
    if let Some(landform) = input.landform {
        terrain_entity.insert(landform);
    }
    terrain_entity.with_children(|parent| {
        for collider in colliders {
            parent.spawn((
                RigidBody::Static,
                CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
                collider,
                Transform::IDENTITY,
            ));
        }
    });
    Ok(())
}

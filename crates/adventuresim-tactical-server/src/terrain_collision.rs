use super::*;
#[cfg(test)]
mod tests;

pub(super) fn colliders(
    terrain: &SceneTerrain,
    recipe: Option<&TerrainLandformRecipe>,
    prepared: Option<&SceneTerrainPatch>,
) -> Result<Vec<Collider>> {
    if let Some(patch) = prepared {
        return Ok(patch.colliders_with_terrain(terrain));
    }
    recipe.map_or_else(
        || Ok(terrain.colliders()),
        |recipe| {
            terrain_landform_patch(terrain, *recipe)
                .map(|patch| patch.colliders_with_terrain(terrain))
                .map_err(|reason| BevyError::from(reason.to_owned()))
        },
    )
}

pub(super) fn spawn_scene(
    commands: &mut Commands,
    scene_id: String,
    terrain: SceneTerrain,
    ground: SceneGround,
    environment: SceneEnvironment,
    terrain_patch: Option<&SceneTerrainPatch>,
    landform: Option<TerrainLandformRecipe>,
) {
    let mut scene = commands.spawn((Replicated, SceneId(scene_id), ground, Transform::default()));
    scene.insert(environment);
    if let Some(recipe) = landform {
        scene.insert(recipe);
    }
    if let Some(patch) = terrain_patch {
        scene.insert(patch.clone());
    }
    // The observer receives the prepared patch and its recipe before it builds
    // independent collision bodies. Dump restoration regenerates that same
    // immutable product when no prepared patch is present.
    scene.insert(terrain);
}

/// Fires whenever `SceneTerrain` lands on any entity - via fresh procedural
/// generation in `on_server_started` or a loaded world dump
/// (`load_world_dump`). Derives independent static collision bodies from the
/// immutable terrain and adds the replication marker. Colliders aren't reflectable
/// and so never survives a dump on its own; a dump only needs to carry the
/// "core" `SceneId`/`SceneTerrain`/`Transform`.
pub(crate) fn on_scene_terrain_added(
    event: On<Add, SceneTerrain>,
    mut commands: Commands,
    query: Query<(
        &SceneTerrain,
        Option<&TerrainLandformRecipe>,
        Option<&SceneTerrainPatch>,
    )>,
) -> Result {
    let (terrain, recipe, prepared) = query.get(event.entity)?;
    let colliders = terrain_collision::colliders(terrain, recipe, prepared)?;
    commands
        .entity(event.entity)
        .insert(Replicated)
        .remove::<SceneTerrainPatch>()
        .with_children(|parent| {
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

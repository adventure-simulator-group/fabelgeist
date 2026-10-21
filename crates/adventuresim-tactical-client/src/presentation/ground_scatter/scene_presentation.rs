//! Present one scene and publish its completed native scatter workload.
use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects independently borrowed scene, asset, cache, and procedural-resource state"
)]
pub(in crate::presentation) fn present_ground_scatter(
    scenes: GroundScatterSceneQuery,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut foliage_materials: ResMut<Assets<TacticalFoliageMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut pebble_materials: ResMut<Assets<TacticalPebbleMaterial>>,
    mut pebble_billboard_materials: ResMut<Assets<TacticalPebbleBillboardMaterial>>,
    mut leaf_materials: ResMut<Assets<TacticalTreeLeafCardMaterial>>,
    mut understory_cache: ResMut<WoodyUnderstoryPresentationCache>,
    mut ground_foliage_cache: ResMut<GroundFoliagePresentationCache>,
    procedural_assets: Res<ProceduralTextureAssets>,
    mut shrub_bark_materials: ResMut<Assets<TacticalShrubBarkInstancedMaterial>>,
    mut shrub_leaf_materials: ResMut<Assets<TacticalShrubLeafInstancedMaterial>>,
) -> bevy::prelude::Result {
    for (entity, scene_id, terrain, ground, environment, fault_scarp) in &scenes {
        let started = web_time::Instant::now();
        tracing::info!("Generating tactical ground scatter");
        let masked_ground = fault_scarp
            .map(|recipe| scatter_ground_without_patch(ground, recipe.transition_collar()));
        let ground = masked_ground.as_ref().unwrap_or(ground);
        let litter_work = spawn_ground_foliage(
            &mut commands,
            &mut meshes,
            &mut foliage_materials,
            &mut standard_materials,
            &mut pebble_materials,
            &mut pebble_billboard_materials,
            &mut leaf_materials,
            &mut understory_cache,
            &mut ground_foliage_cache,
            &procedural_assets,
            scene_id,
            terrain,
            ground,
            environment,
            &mut shrub_bark_materials,
            &mut shrub_leaf_materials,
        )?;
        tracing::info!(
            elapsed_ms = started.elapsed().as_millis(),
            "Generated tactical ground scatter"
        );
        commands
            .entity(entity)
            .insert((GroundScatterPresented, litter_work));
    }
    Ok(())
}

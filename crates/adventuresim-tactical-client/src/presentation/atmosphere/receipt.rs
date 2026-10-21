//! Read-only cache ownership, separate from actual GPU radiance consumption.
use super::*;
use serde_json::{Value, json};

pub(in crate::presentation) fn snapshot(world: &World, cameras: &[Entity]) -> Value {
    let cache = world.get_resource::<AtmosphereIblCache>().map(|cache| {
        let phase = match cache.phase {
            AtmosphereIblPhase::WaitingForScene => json!({"state": "waiting-for-scene"}),
            AtmosphereIblPhase::Baking { scene } => {
                json!({"state": "baking", "scene_entity": scene.to_bits()})
            }
            AtmosphereIblPhase::Cached => json!({"state": "cached"}),
        };
        let key = cache.key.as_ref().map(|key| {
            json!({
                "scene_entity": key.scene.to_bits(), "environment": key.environment,
                "cubemap_size": key.size, "environment_light": key.environment_light,
                "atmosphere_entity": key.atmosphere.to_bits(),
                "atmosphere_transform_bits": key.atmosphere_transform,
                "atmosphere_tick": key.atmosphere_tick,
                "medium_revision": key.medium_revision, "shader_revision": key.shader_revision,
            })
        });
        json!({"phase": phase, "key": key, "medium_revision": cache.medium_revision,
            "completed_bakes": cache.completed_bakes})
    });
    let cached: Vec<Value> = cameras
        .iter()
        .map(|entity| {
            let assets = world
                .get::<CachedAtmosphereProbeAssets>(*entity)
                .map(|assets| {
                    json!({
                        "generated_map": format!("{:?}", assets._environment_map.id()),
                        "diffuse_map": format!("{:?}", assets._diffuse_map.id()),
                        "specular_map": format!("{:?}", assets._specular_map.id()),
                    })
                });
            json!({"camera_entity": entity.to_bits(), "retained_assets": assets})
        })
        .collect();
    let corrections = world
        .get_resource::<AtmosphereShaderCorrections>()
        .map(|corrections| {
            json!({
                "installed": corrections.installed, "revision": corrections.revision,
            })
        });
    json!({"cache": cache, "cached_camera_assets": cached, "shader_corrections": corrections,
        "gpu_bake_history": world.get_resource::<AtmosphereBakeGpu>().map(AtmosphereBakeGpu::receipt)})
}

/// Return only the current scene's already fenced and installed probe maps.
pub(in crate::presentation) fn completed_probe(
    world: &World,
    scene: Entity,
    camera: Entity,
) -> Option<Value> {
    let settings = world.resource::<TacticalGraphicsSettings>();
    if !settings.config.rendering.atmosphere.enabled {
        return Some(json!({"state":"disabled"}));
    }
    let cache = world.get_resource::<AtmosphereIblCache>()?;
    if !matches!(cache.phase, AtmosphereIblPhase::Cached) {
        return None;
    }
    let key = cache.key.as_ref()?;
    if key.scene != scene || key.environment != *world.get::<SceneEnvironment>(scene)? {
        return None;
    }
    let retained = world.get::<CachedAtmosphereProbeAssets>(camera)?;
    if key.environment_light {
        let installed = world.get::<EnvironmentMapLight>(camera)?;
        if installed.diffuse_map.id() != retained._diffuse_map.id()
            || installed.specular_map.id() != retained._specular_map.id()
        {
            return None;
        }
    }
    Some(
        json!({"state":"cached","scene_entity":scene.to_bits().to_string(),
        "camera_entity":camera.to_bits().to_string(),"completed_bakes":cache.completed_bakes,
        "generated_map":format!("{:?}",retained._environment_map.id()),
        "diffuse_map":format!("{:?}",retained._diffuse_map.id()),
        "specular_map":format!("{:?}",retained._specular_map.id()),
        "scope":"existing scene-keyed GPU bake fence and current camera map installation; no universal material-pipeline readiness claim"}),
    )
}

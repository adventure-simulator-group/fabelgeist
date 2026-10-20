//! Explicit, one-shot observation of final main-world tactical lighting.
//!
//! These values describe components after PostUpdate, not GPU bindings, drawn
//! fragments, arrival readiness, or the exact frame of a later screenshot.
use super::{
    ActiveTacticalScene, PresentedCelestialLighting, SceneEnvironment, TacticalGameplayCamera,
    TacticalMoonlight, TacticalSunlight, atmosphere, interior_lighting::InteriorExposure,
};
use bevy::{
    camera::{Exposure, Projection, RenderTarget},
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::FrameCount,
    light::{CascadeShadowConfig, DirectionalLightShadowMap, EnvironmentMapLight},
    pbr::AtmosphereSettings,
    prelude::*,
    render::view::ColorGrading,
};
use serde_json::{Value, json};

#[derive(Resource, Default)]
enum FrameReceiptRequest {
    #[default]
    Idle,
    Armed,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<FrameReceiptRequest>()
        .add_systems(Last, capture);
}

pub(crate) fn arm(world: &mut World) {
    world.insert_resource(FrameReceiptRequest::Armed);
}

fn capture(world: &mut World) {
    if !matches!(
        world.resource::<FrameReceiptRequest>(),
        FrameReceiptRequest::Armed
    ) {
        return;
    }
    world.insert_resource(FrameReceiptRequest::Idle);
    let scene = world.resource::<ActiveTacticalScene>().entity;
    let environment = scene.and_then(|entity| world.get::<SceneEnvironment>(entity));
    let identity = json!({
        "scene_entity": scene.map(Entity::to_bits),
        "environment": environment,
        "main_frame": world.resource::<FrameCount>().0,
    });
    let celestial = world
        .resource::<PresentedCelestialLighting>()
        .snapshot
        .as_ref()
        .map(|sky| {
            json!({
                "scene_entity": sky.scene.to_bits(),
                "matches_active_scene": Some(sky.scene) == scene,
                "sun_direction": sky.sun_direction.to_array(),
                "moon_direction": sky.moon_direction.to_array(),
                "sun_altitude_degrees": sky.sun_altitude_degrees,
                "moon_altitude_degrees": sky.moon_altitude_degrees,
                "lunar_illumination": sky.lunar_illumination,
                "weather_transmission": sky.weather_transmission,
                "base_exposure_ev100": sky.exposure_ev100,
            })
        });
    let cameras: Vec<Entity> = world
        .query_filtered::<Entity, With<TacticalGameplayCamera>>()
        .iter(world)
        .collect();
    let camera_records: Vec<Value> = cameras
        .iter()
        .map(|entity| camera_snapshot(world, *entity))
        .collect();
    let atmosphere = atmosphere::receipt::snapshot(world, &cameras);
    let exposure = world
        .get_resource::<InteriorExposure>()
        .map(InteriorExposure::receipt);
    let lights = directional_light_snapshot(world);
    let ambient = world.get_resource::<GlobalAmbientLight>().map(|ambient| {
        json!({
            "brightness": ambient.brightness,
            "color_linear": ambient.color.to_linear().to_f32_array(),
        })
    });
    let receipt = json!({
        "schema": "tactical-main-lighting-1", "identity": identity,
        "scope": "final main-world components only; GPU bindings, visible fragments, exact screenshot association and arrival readiness unestablished",
        "camera_count": cameras.len(), "cameras": camera_records,
        "celestial": celestial, "interior_exposure": exposure,
        "directional_lights": lights, "ambient": ambient,
        "directional_shadow_map_size": world.get_resource::<DirectionalLightShadowMap>().map(|map| map.size),
        "atmosphere": atmosphere,
    });
    info!("[lighting-receipt] {receipt}");
}

fn camera_snapshot(world: &World, entity: Entity) -> Value {
    let camera = world.get::<Camera>(entity);
    let environment = world.get::<EnvironmentMapLight>(entity).map(|light| {
        json!({
            "diffuse_map": format!("{:?}", light.diffuse_map.id()),
            "specular_map": format!("{:?}", light.specular_map.id()),
            "intensity": light.intensity,
            "rotation": light.rotation.to_array(),
            "affects_lightmapped_mesh_diffuse": light.affects_lightmapped_mesh_diffuse,
        })
    });
    json!({
        "entity": entity.to_bits(),
        "camera": camera.map(|camera| json!({
            "is_active": camera.is_active, "order": camera.order,
            "target": world.get::<RenderTarget>(entity).map(|target| format!("{target:?}")),
            "physical_viewport": camera.physical_viewport_rect().map(|rect| [rect.min.to_array(), rect.max.to_array()]),
            "physical_target_size": camera.physical_target_size().map(|size| size.to_array()),
        })),
        "world_from_camera": world.get::<GlobalTransform>(entity).map(|transform| transform.to_matrix().to_cols_array()),
        "projection": world.get::<Projection>(entity).map(|value| format!("{value:?}")),
        "exposure_ev100": world.get::<Exposure>(entity).map(|exposure| exposure.ev100),
        "tonemapping": world.get::<Tonemapping>(entity).map(|value| format!("{value:?}")),
        "msaa": world.get::<Msaa>(entity).map(|value| format!("{value:?}")),
        "color_grading": world.get::<ColorGrading>(entity).map(|value| format!("{value:?}")),
        "atmosphere_settings": world.get::<AtmosphereSettings>(entity).map(atmosphere_settings_snapshot),
        "environment_map": environment,
    })
}

fn directional_light_snapshot(world: &mut World) -> Vec<Value> {
    world
        .query::<(
            Entity,
            &DirectionalLight,
            &GlobalTransform,
            Has<TacticalSunlight>,
            Has<TacticalMoonlight>,
            Option<&CascadeShadowConfig>,
        )>()
        .iter(world)
        .map(|(entity, light, transform, sun, moon, cascades)| {
            json!({
                "entity": entity.to_bits(), "sun": sun, "moon": moon,
                "illuminance_lux": light.illuminance,
                "color_linear": light.color.to_linear().to_f32_array(),
                "world_from_light": transform.to_matrix().to_cols_array(),
                "shadow_maps_enabled": light.shadow_maps_enabled,
                "contact_shadows_enabled": light.contact_shadows_enabled,
                "shadow_depth_bias": light.shadow_depth_bias,
                "shadow_normal_bias": light.shadow_normal_bias,
                "cascades": cascades.map(|value| format!("{value:?}")),
            })
        })
        .collect()
}

fn atmosphere_settings_snapshot(settings: &AtmosphereSettings) -> Value {
    json!({
        "transmittance_lut_size": settings.transmittance_lut_size.to_array(),
        "multiscattering_lut_size": settings.multiscattering_lut_size.to_array(),
        "sky_view_lut_size": settings.sky_view_lut_size.to_array(),
        "aerial_view_lut_size": settings.aerial_view_lut_size.to_array(),
        "transmittance_lut_samples": settings.transmittance_lut_samples,
        "multiscattering_lut_dirs": settings.multiscattering_lut_dirs,
        "multiscattering_lut_samples": settings.multiscattering_lut_samples,
        "sky_view_lut_samples": settings.sky_view_lut_samples,
        "aerial_view_lut_samples": settings.aerial_view_lut_samples,
        "aerial_view_lut_max_distance_metres": settings.aerial_view_lut_max_distance,
        "sky_max_samples": settings.sky_max_samples,
        "rendering_method": settings.rendering_method as u32,
    })
}

//! Shared Bevy preview scene for interactive authoring and native review captures.
mod backdrop;
mod geometry;
mod maps;
mod special;
mod terrain;
#[cfg(test)]
mod tests;
use crate::{
    app::Studio,
    document::{Document, Environment},
};
use bevy::{
    camera::{Exposure, ScalingMode, Viewport},
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
};

#[derive(Component)]
struct PreviewCamera;
#[derive(Component)]
struct KeyLight;
#[derive(Component)]
struct FillLight;
#[derive(Resource, Default)]
struct SceneAssets {
    entities: Vec<Entity>,
    images: Vec<Handle<Image>>,
    materials: Vec<Handle<StandardMaterial>>,
    bark: Vec<Handle<adventuresim_procedural_materials::TacticalTreeBarkMaterial>>,
    leaves: Vec<Handle<adventuresim_procedural_materials::TacticalTreeLeafCardMaterial>>,
    meshes: Vec<Handle<Mesh>>,
    signature: String,
}

pub(crate) fn setup(world: &mut World) {
    world.insert_resource(SceneAssets::default());
    world.insert_resource(GlobalAmbientLight {
        brightness: 120.0,
        ..default()
    });
    world.spawn((
        PreviewCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 3.2).looking_at(Vec3::ZERO, Vec3::Y),
        Tonemapping::TonyMcMapface,
        Exposure::default(),
    ));
    if let Some(mut settings) = world.get_resource_mut::<bevy_egui::EguiGlobalSettings>() {
        settings.auto_create_primary_context = false;
        world.spawn((
            Camera2d,
            bevy_egui::PrimaryEguiContext,
            bevy::camera::visibility::RenderLayers::none(),
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                output_mode: bevy::camera::CameraOutputMode::Write {
                    blend_state: Some(bevy::render::render_resource::BlendState::ALPHA_BLENDING),
                    clear_color: ClearColorConfig::None,
                },
                ..default()
            },
        ));
    }
    world.spawn((KeyLight, DirectionalLight::default(), Transform::default()));
    world.spawn((FillLight, DirectionalLight::default(), Transform::default()));
}

pub(crate) fn update(world: &mut World) {
    let (document, rect, dirty, comparison) = {
        let studio = world.resource::<Studio>();
        (
            studio.document.clone(),
            studio.viewport,
            studio.scene_dirty,
            studio.pinned.is_some(),
        )
    };
    lighting(world, &document.environment);
    camera(world, &document, rect, comparison);
    let signature = serde_json::to_string(&(
        document.view.shape,
        document.view.repeats,
        document.view.offset,
        document.view.displacement,
        &document.surface,
    ))
    .unwrap();
    if dirty || signature != world.resource::<SceneAssets>().signature {
        rebuild(world);
        world.resource_mut::<SceneAssets>().signature = signature;
        world.resource_mut::<Studio>().scene_dirty = false;
    }
    if document.view.turntable {
        let delta = world.resource::<Time>().delta_secs();
        world.resource_mut::<Studio>().document.view.yaw += delta * 0.3;
    }
}

fn lighting(world: &mut World, environment: &Environment) {
    *world.resource_mut::<ClearColor>() =
        ClearColor(Color::srgb_from_array(environment.background));
    world.resource_mut::<GlobalAmbientLight>().brightness = environment.ambient;
    let azimuth = environment.azimuth_degrees.to_radians();
    let incidence = environment.incidence_degrees.to_radians();
    let direction = Vec3::new(
        azimuth.sin() * incidence.cos(),
        azimuth.cos() * incidence.cos(),
        incidence.sin(),
    );
    for (mut light, mut transform) in world
        .query_filtered::<(&mut DirectionalLight, &mut Transform), With<KeyLight>>()
        .iter_mut(world)
    {
        light.illuminance = environment.key_lux;
        light.color = Color::srgb_from_array(environment.key_color);
        light.shadow_maps_enabled = environment.shadows;
        *transform = Transform::from_translation(direction).looking_at(Vec3::ZERO, Vec3::Y);
    }
    for (_, material) in world
        .resource_mut::<Assets<adventuresim_procedural_materials::TacticalTreeBarkMaterial>>()
        .iter_mut()
    {
        material.extension.lighting = direction.extend((environment.key_lux / 7000.0).min(1.0));
    }
    for (mut light, mut transform) in world
        .query_filtered::<(&mut DirectionalLight, &mut Transform), With<FillLight>>()
        .iter_mut(world)
    {
        light.illuminance = environment.fill_lux;
        light.color = Color::srgb_from_array(environment.fill_color);
        *transform = Transform::from_xyz(0.8, 0.4, 0.7).looking_at(Vec3::ZERO, Vec3::Y);
    }
}

fn camera(world: &mut World, document: &Document, rect: bevy_egui::egui::Rect, comparison: bool) {
    let window = world.query::<&Window>().iter(world).next();
    let scale = window.map(|w| w.scale_factor()).unwrap_or(1.0);
    let extent = window.map(|w| UVec2::new(w.physical_width(), w.physical_height()));
    let view = &document.view;
    for (mut camera, mut projection, mut transform, mut exposure) in world.query_filtered::<(&mut Camera, &mut Projection, &mut Transform, &mut Exposure), With<PreviewCamera>>().iter_mut(world) {
        if let Some(extent) = extent
            && rect.is_positive() {
                let origin = UVec2::new((rect.min.x * scale) as u32, (rect.min.y * scale) as u32).min(extent.saturating_sub(UVec2::ONE));
                let size = UVec2::new((rect.width() * scale) as u32, (rect.height() * scale) as u32).max(UVec2::ONE).min(extent - origin);
                camera.viewport = Some(Viewport { physical_position: origin, physical_size: size, ..default() });
            }
        let distance = view.distance * if comparison {1.7} else {1.0};
        let position = Quat::from_euler(EulerRot::YXZ, view.yaw, view.pitch, 0.0) * Vec3::Z * distance;
        *transform = Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y);
        *projection = if view.orthographic {
            Projection::Orthographic(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: distance * 0.6625 }, ..OrthographicProjection::default_3d() })
        } else { Projection::Perspective(PerspectiveProjection::default()) };
        exposure.ev100 = Exposure::default().ev100 - document.environment.exposure_ev;
    }
}

fn rebuild(world: &mut World) {
    let (current, pinned, document, channel) = {
        let studio = world.resource::<Studio>();
        (
            studio.current.clone(),
            studio.pinned.clone(),
            studio.document.clone(),
            studio.channel,
        )
    };
    let Some(current) = current else {
        return;
    };
    let mut assets = world.remove_resource::<SceneAssets>().unwrap();
    for entity in assets.entities.drain(..) {
        world.despawn(entity);
    }
    for image in assets.images.drain(..) {
        world.resource_mut::<Assets<Image>>().remove(image.id());
    }
    for material in assets.materials.drain(..) {
        world
            .resource_mut::<Assets<StandardMaterial>>()
            .remove(material.id());
    }
    for material in assets.bark.drain(..) {
        world
            .resource_mut::<Assets<adventuresim_procedural_materials::TacticalTreeBarkMaterial>>()
            .remove(material.id());
    }
    for material in assets.leaves.drain(..) {
        world.resource_mut::<Assets<adventuresim_procedural_materials::TacticalTreeLeafCardMaterial>>().remove(material.id());
    }
    for mesh in assets.meshes.drain(..) {
        world.resource_mut::<Assets<Mesh>>().remove(mesh.id());
    }
    let x = if pinned.is_some() { 0.82 } else { 0.0 };
    spawn_material(world, &mut assets, &current, &document, channel, x);
    if let Some((pinned, mut pinned_document)) = pinned {
        pinned_document.view = document.view.clone();
        spawn_material(
            world,
            &mut assets,
            &pinned,
            &pinned_document,
            channel,
            -0.82,
        );
    }
    world.insert_resource(assets);
}

fn spawn_material(
    world: &mut World,
    assets: &mut SceneAssets,
    bake: &adventuresim_procedural_textures::BakedRecipe,
    document: &Document,
    channel: Option<adventuresim_procedural_textures::MapChannel>,
    x: f32,
) {
    if bake.recipe == adventuresim_procedural_textures::TextureRecipeId::WindowGlass
        && channel.is_none()
    {
        backdrop::spawn(world, assets, x);
    }
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(geometry::mesh(bake, &document.view));
    assets.meshes.push(mesh.clone());
    if channel.is_none() && special::spawn(world, assets, bake, document, &mesh, x) {
        return;
    }
    let material = maps::material(world, assets, bake, document, channel);
    assets.materials.push(material.clone());
    assets.entities.push(
        world
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(x, 0.0, 0.0),
            ))
            .id(),
    );
}

//! The interactive editor and CLI captures use this identical scene.
mod environment;
mod maps;
use crate::app::{Channel, Studio};
use adventuresim_heraldry::{bake::Baked, document::Document};
use bevy::{
    camera::{Exposure, ScalingMode, Viewport},
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
};
#[derive(Component)]
pub(crate) struct PreviewCamera;
#[derive(Component)]
struct KeyLight;
#[derive(Resource, Default)]
struct SceneAssets {
    entities: Vec<Entity>,
    images: Vec<Handle<Image>>,
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
}
pub(crate) fn setup(world: &mut World) {
    world.insert_resource(SceneAssets::default());
    world.insert_resource(ClearColor(Color::srgb(0.055, 0.067, 0.079)));
    world.insert_resource(GlobalAmbientLight {
        brightness: 120.0,
        ..default()
    });
    let reflection = environment::studio_map(world);
    world.spawn((
        PreviewCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        Tonemapping::TonyMcMapface,
        Exposure::default(),
        reflection,
    ));
    world.spawn((
        KeyLight,
        DirectionalLight {
            illuminance: 5500.0,
            ..default()
        },
        Transform::from_xyz(-1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    world.spawn((
        DirectionalLight {
            illuminance: 1700.0,
            color: Color::srgb(0.82, 0.89, 1.0),
            ..default()
        },
        Transform::from_xyz(1.0, -0.4, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
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
}
pub(crate) fn update(world: &mut World) {
    let Some(mut studio) = world.remove_resource::<Studio>() else {
        return;
    };
    camera(world, &studio);
    if studio.scene_dirty
        && let Some(baked) = &studio.current
    {
        let mut assets = world.remove_resource::<SceneAssets>().unwrap();
        for entity in assets.entities.drain(..) {
            world.despawn(entity);
        }
        for h in assets.images.drain(..) {
            world.resource_mut::<Assets<Image>>().remove(h.id());
        }
        for h in assets.materials.drain(..) {
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .remove(h.id());
        }
        for h in assets.meshes.drain(..) {
            world.resource_mut::<Assets<Mesh>>().remove(h.id());
        }
        let current = studio.current_document.as_ref().unwrap_or(&studio.document);
        let gap = current.surface.width.metres() * 0.15;
        let offset = studio
            .pinned
            .as_ref()
            .map_or(0.0, |(d, _)| (d.surface.width.metres() + gap) * 0.5);
        spawn(
            world,
            &mut assets,
            studio.current_document.as_ref().unwrap_or(&studio.document),
            baked,
            studio.channel,
            offset,
        );
        if let Some((d, b)) = &studio.pinned {
            let left = -(current.surface.width.metres() + gap) * 0.5;
            spawn(world, &mut assets, d, b, studio.channel, left);
        }
        world.insert_resource(assets);
        studio.scene_dirty = false;
    }
    world.insert_resource(studio);
}
fn camera(world: &mut World, studio: &Studio) {
    let d = &studio.document;
    let v = &d.view;
    let flat = studio.channel != Channel::Physical;
    let window = world.query::<&Window>().iter(world).next();
    let scale = window.map_or(1.0, Window::scale_factor);
    let extent = window.map(|w| UVec2::new(w.physical_width(), w.physical_height()));
    let current = studio.current_document.as_ref().unwrap_or(d);
    let width = current.surface.width.metres()
        + studio.pinned.as_ref().map_or(0.0, |(p, _)| {
            p.surface.width.metres() + current.surface.width.metres() * 0.15
        });
    let object_height = current.surface.height.metres().max(
        studio
            .pinned
            .as_ref()
            .map_or(0.0, |(p, _)| p.surface.height.metres()),
    );
    let aspect = if studio.viewport.is_positive() {
        studio.viewport.aspect_ratio()
    } else {
        1.6
    };
    let height = object_height.max(width / aspect) * 1.25 / v.zoom.0;
    let rotation = if flat {
        Quat::IDENTITY
    } else {
        Quat::from_euler(
            EulerRot::YXZ,
            v.yaw.0.to_radians(),
            v.pitch.0.to_radians(),
            0.0,
        )
    };
    for (mut camera, mut projection, mut transform, mut exposure, mut tonemapping) in world
        .query_filtered::<(
            &mut Camera,
            &mut Projection,
            &mut Transform,
            &mut Exposure,
            &mut Tonemapping,
        ), With<PreviewCamera>>()
        .iter_mut(world)
    {
        if let Some(extent) = extent
            && studio.viewport.is_positive()
        {
            let r = studio.viewport;
            let origin = UVec2::new((r.min.x * scale) as u32, (r.min.y * scale) as u32)
                .min(extent.saturating_sub(UVec2::ONE));
            let size = UVec2::new((r.width() * scale) as u32, (r.height() * scale) as u32)
                .max(UVec2::ONE)
                .min(extent - origin);
            camera.viewport = Some(Viewport {
                physical_position: origin,
                physical_size: size,
                ..default()
            });
        }
        *projection = Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: height,
            },
            ..OrthographicProjection::default_3d()
        });
        *transform =
            Transform::from_translation(rotation * Vec3::Z * 4.0).looking_at(Vec3::ZERO, Vec3::Y);
        exposure.ev100 = Exposure::default().ev100 - v.exposure;
        *tonemapping = if flat {
            Tonemapping::None
        } else {
            Tonemapping::TonyMcMapface
        };
    }
    let angle = v.light.0.to_radians();
    for mut t in world
        .query_filtered::<&mut Transform, With<KeyLight>>()
        .iter_mut(world)
    {
        *t = Transform::from_xyz(angle.sin(), 0.7, angle.cos()).looking_at(Vec3::ZERO, Vec3::Y);
    }
    for mut e in world.query::<&mut EnvironmentMapLight>().iter_mut(world) {
        e.rotation = Quat::from_rotation_y(angle);
    }
}

fn spawn(
    world: &mut World,
    assets: &mut SceneAssets,
    d: &Document,
    b: &Baked,
    channel: Channel,
    x: f32,
) {
    let mut d = d.clone();
    if channel != Channel::Physical {
        d.surface.curvature.0 = 0.0;
    }
    let geometry = adventuresim_heraldry::geometry::Mesh::generate(&d).unwrap();
    let front = maps::material(world, assets, b, channel);
    let support = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::linear_rgb(0.16, 0.08, 0.035),
            perceptual_roughness: 0.8,
            ..default()
        });
    assets.materials.extend([front.clone(), support.clone()]);
    for (indices, material) in [(&geometry.front, front), (&geometry.support, support)] {
        let mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, geometry.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, geometry.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, geometry.uv.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, geometry.tangents.clone())
        .with_inserted_indices(bevy::mesh::Indices::U32(indices.clone()));
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        assets.meshes.push(mesh.clone());
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_inspection_bypasses_display_tonemapping() {
        let mut world = World::new();
        let entity = world
            .spawn((
                PreviewCamera,
                Camera3d::default(),
                Transform::default(),
                Exposure::default(),
                Tonemapping::TonyMcMapface,
            ))
            .id();
        let mut studio = Studio::new(Document::default());
        studio.channel = Channel::Flat;
        camera(&mut world, &studio);
        assert_eq!(world.get::<Tonemapping>(entity), Some(&Tonemapping::None));
        studio.channel = Channel::Physical;
        camera(&mut world, &studio);
        assert_eq!(
            world.get::<Tonemapping>(entity),
            Some(&Tonemapping::TonyMcMapface)
        );
    }
}

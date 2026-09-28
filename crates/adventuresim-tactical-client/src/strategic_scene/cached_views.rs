//! Retained camera targets avoid redrawing the street and portraits every frame.
//! The live room, snapshots and HTML-clipped images still share one window canvas.
use super::{
    RetainedScene,
    protocol::{CanvasRect, PortraitId, StrategicView},
    views::{StrategicCamera, ViewSpec},
};
use adventuresim_core::equipment_presentation::EquipmentAppearance;
use bevy::{
    camera::{CameraOutputMode, RenderTarget, visibility::RenderLayers},
    ecs::system::SystemParam,
    prelude::*,
    render::render_resource::{BlendState, TextureFormat},
};
use std::collections::HashMap;

#[cfg(test)]
mod environment_tests;
mod preload;
mod travel;

// A newly active view needs visibility, material specialization and render-world
// preparation before its image contains geometry, even with resident assets.
const CAPTURE_SETTLED_FRAMES: usize = 4;
const CONCURRENT_CAPTURES: usize = 2;
const COMPOSITOR_ORDER: isize = 256;

fn compositor_camera(active: bool) -> Camera {
    Camera {
        order: COMPOSITOR_ORDER,
        is_active: active,
        clear_color: ClearColorConfig::Custom(Color::NONE),
        // 2D and HDR 3D views have separate intermediate textures. Preserve the
        // live view already written to the window when compositing captured images.
        output_mode: CameraOutputMode::Write {
            blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            clear_color: ClearColorConfig::None,
        },
        ..default()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ViewKey {
    Street,
    Portrait { person: PortraitId, slot: usize },
}

#[derive(PartialEq)]
struct Signature {
    rect: CanvasRect,
    transform: Transform,
    equipment: Vec<EquipmentAppearance>,
}

impl Signature {
    fn new(spec: &ViewSpec, view: Option<&StrategicView>) -> Self {
        let key = spec.cache.as_ref().expect("cached view");
        Self {
            rect: match key {
                ViewKey::Street => spec.rect.uncropped(),
                ViewKey::Portrait { .. } => CanvasRect {
                    x: 0,
                    y: 0,
                    ..spec.rect
                },
            },
            transform: spec.transform,
            equipment: match key {
                ViewKey::Street => Vec::new(),
                ViewKey::Portrait { person: id, .. } => view
                    .and_then(|view| view.people.iter().find(|person| person.id == *id))
                    .map_or_else(Vec::new, |person| person.equipment.clone()),
            },
        }
    }
}
struct Frame {
    camera: Option<Entity>,
    image: Handle<Image>,
    projection: Projection,
    layers: RenderLayers,
    sprite: Entity,
    signature: Signature,
    remaining: usize,
}
#[derive(Resource, Default)]
pub(super) struct CachedViews {
    scene: Option<Entity>,
    identity: (String, String),
    previous: std::collections::VecDeque<travel::SavedViews>,
    compositor: Option<Entity>,
    frames: HashMap<ViewKey, Frame>,
    free_cameras: Vec<Entity>,
}
impl CachedViews {
    pub(super) fn pending_captures(&self) -> usize {
        self.frames
            .values()
            .filter(|frame| frame.remaining > 0)
            .count()
    }

    pub(super) fn is_ready(&self) -> bool {
        self.frames.values().all(|frame| frame.remaining == 0)
    }
}

#[derive(Component)]
struct SnapshotCamera;

pub(super) fn install(app: &mut App) {
    app.init_resource::<CachedViews>().add_systems(
        PostUpdate,
        // Capture cameras are spawned here. Apply their final environment before
        // camera preparation/extraction can specialize transient default pipelines.
        (settle, super::views::sync_environment)
            .chain()
            .before(bevy::camera::CameraUpdateSystems)
            .before(bevy::transform::TransformSystems::Propagate),
    );
}

#[derive(SystemParam)]
pub(super) struct CacheWorld<'w> {
    retained: ResMut<'w, CachedViews>,
    images: ResMut<'w, Assets<Image>>,
}
impl CacheWorld<'_> {
    pub(super) fn sync(
        &mut self,
        commands: &mut Commands,
        specs: &[ViewSpec],
        scene: &RetainedScene,
        view: Option<&StrategicView>,
        window: &Window,
    ) {
        let compositor = *self.retained.compositor.get_or_insert_with(|| {
            commands
                .spawn((
                    Camera2d,
                    compositor_camera(view.is_some()),
                    Msaa::Off,
                    RenderLayers::layer(super::protocol::COMPOSITOR_LAYER),
                ))
                .id()
        });
        if self.retained.scene != scene.root {
            self.retained.change_scene(commands, scene);
        }
        for frame in self.retained.frames.values_mut() {
            commands.entity(frame.sprite).insert(Visibility::Hidden);
        }
        for spec in specs {
            let Some(key) = &spec.cache else {
                continue;
            };
            let signature = Signature::new(spec, view);
            let changed = self
                .retained
                .frames
                .get(key)
                .is_none_or(|frame| frame.signature != signature);
            if changed {
                if let Some(previous) = self.retained.frames.remove(key) {
                    if let Some(camera) = previous.camera {
                        deactivate(commands, camera);
                        self.retained.free_cameras.push(camera);
                    }
                    commands.entity(previous.sprite).despawn();
                }
                let frame = self.create(commands, spec, signature);
                self.retained.frames.insert(key.clone(), frame);
            }
            let frame = self.retained.frames.get_mut(key).expect("created snapshot");
            frame.show(commands, spec, window);
        }
        if let Some(view) = view {
            for spec in preload::specs(view, scene) {
                let key = spec.cache.clone().expect("portrait preload");
                let signature = Signature::new(&spec, Some(view));
                if self
                    .retained
                    .frames
                    .get(&key)
                    .is_none_or(|frame| frame.signature != signature)
                {
                    if let Some(previous) = self.retained.frames.remove(&key) {
                        if let Some(camera) = previous.camera {
                            deactivate(commands, camera);
                            self.retained.free_cameras.push(camera);
                        }
                        commands.entity(previous.sprite).despawn();
                    }
                    let frame = self.create(commands, &spec, signature);
                    self.retained.frames.insert(key, frame);
                }
            }
        }
        let active = view.is_some();
        // Replacing Camera would erase its computed render-target metadata.
        // Bevy only recomputes that metadata on target/projection changes.
        commands.queue(move |world: &mut World| {
            if let Some(mut camera) = world.get_mut::<Camera>(compositor) {
                camera.is_active = active;
            }
        });
    }

    fn create(&mut self, commands: &mut Commands, spec: &ViewSpec, signature: Signature) -> Frame {
        let rect = signature.rect;
        let image = self.images.add(Image::new_target_texture(
            rect.width.max(1),
            rect.height.max(1),
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        let sprite = commands
            .spawn((
                Sprite {
                    image: image.clone(),
                    custom_size: Some(Vec2::new(rect.width as f32, rect.height as f32)),
                    ..default()
                },
                RenderLayers::layer(super::protocol::COMPOSITOR_LAYER),
                Visibility::Hidden,
            ))
            .id();
        Frame {
            camera: None,
            image,
            projection: spec.projection.clone(),
            layers: spec.layers.clone(),
            sprite,
            signature,
            remaining: CAPTURE_SETTLED_FRAMES,
        }
    }
}

impl Frame {
    fn start_capture(&mut self, commands: &mut Commands, reusable: Option<Entity>) {
        let rect = self.signature.rect;
        let mut camera = Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.13, 0.16, 0.19)),
            ..default()
        };
        rect.apply(&mut camera, UVec2::new(rect.width, rect.height));
        let mut entity = match reusable {
            Some(camera) => commands.entity(camera),
            None => commands.spawn_empty(),
        };
        self.camera = Some(
            entity
                .insert((
                    StrategicCamera,
                    crate::presentation::interior_lighting::FixedViewExposure,
                    SnapshotCamera,
                    Camera3d::default(),
                    camera,
                    RenderTarget::Image(self.image.clone().into()),
                    self.signature.transform,
                    self.projection.clone(),
                    self.layers.clone(),
                    Msaa::Off,
                    bevy::camera::Exposure::SUNLIGHT,
                ))
                .id(),
        );
    }

    fn show(&mut self, commands: &mut Commands, spec: &ViewSpec, window: &Window) {
        let scale = window.scale_factor();
        let rect = spec.rect;
        commands.entity(self.sprite).insert((
            Visibility::Inherited,
            Transform::from_xyz(
                (rect.x as f32 + rect.width as f32 * 0.5) / scale - window.width() * 0.5,
                window.height() * 0.5 - (rect.y as f32 + rect.height as f32 * 0.5) / scale,
                0.0,
            )
            .with_scale(Vec3::splat(scale.recip())),
        ));
        if matches!(spec.cache, Some(ViewKey::Street)) {
            let sprite = self.sprite;
            commands.queue(move |world: &mut World| {
                if let Some(mut sprite) = world.get_mut::<Sprite>(sprite) {
                    sprite.rect = Some(Rect::new(
                        rect.offset_x,
                        rect.offset_y,
                        rect.offset_x + rect.width as f32,
                        rect.offset_y + rect.height as f32,
                    ));
                    sprite.custom_size = Some(Vec2::new(rect.width as f32, rect.height as f32));
                }
            });
        }
    }
}

fn deactivate(commands: &mut Commands, camera: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(mut camera) = world.get_mut::<Camera>(camera) {
            camera.is_active = false;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolling_crops_the_same_street_target_without_recapturing() {
        let mut app = App::new();
        app.init_resource::<CachedViews>()
            .init_resource::<Assets<Image>>()
            .add_systems(
                Update,
                |mut commands: Commands, mut cache: CacheWorld, mut offset: Local<f32>| {
                    let spec = ViewSpec {
                        cache: Some(ViewKey::Street),
                        rect: CanvasRect {
                            x: 20,
                            y: 50,
                            width: 400,
                            height: 200,
                            full_width: 2000,
                            full_height: 200,
                            offset_x: *offset,
                            offset_y: 0.0,
                        },
                        transform: Transform::IDENTITY,
                        projection: PerspectiveProjection::default().into(),
                        layers: RenderLayers::default(),
                    };
                    cache.sync(
                        &mut commands,
                        &[spec],
                        &RetainedScene::default(),
                        None,
                        &Window::default(),
                    );
                    *offset += 200.0;
                },
            );
        app.update();
        let (image, sprite) = {
            let mut cache = app.world_mut().resource_mut::<CachedViews>();
            let frame = cache.frames.get_mut(&ViewKey::Street).unwrap();
            frame.remaining = 0;
            (frame.image.clone(), frame.sprite)
        };
        app.update();
        let cache = app.world().resource::<CachedViews>();
        let frame = &cache.frames[&ViewKey::Street];
        assert_eq!(frame.image, image);
        assert_eq!(frame.remaining, 0);
        assert_eq!(
            app.world()
                .resource::<Assets<Image>>()
                .get(&image)
                .unwrap()
                .size(),
            UVec2::new(2000, 200)
        );
        assert_eq!(
            app.world().get::<Sprite>(sprite).unwrap().rect,
            Some(Rect::new(200.0, 0.0, 600.0, 200.0))
        );
    }

    #[test]
    fn new_pipeline_work_does_not_invalidate_completed_images() {
        let mut app = App::new();
        let mut cached = CachedViews::default();
        let mut entities = Vec::new();
        for (id, remaining) in [(1, 0), (2, CAPTURE_SETTLED_FRAMES)] {
            let camera = app
                .world_mut()
                .spawn((
                    Camera::default(),
                    bevy::camera::visibility::VisibleEntities::default(),
                    SnapshotCamera,
                ))
                .id();
            entities.push(camera);
            cached.frames.insert(
                ViewKey::Portrait {
                    person: PortraitId(id),
                    slot: 0,
                },
                Frame {
                    camera: Some(camera),
                    image: Handle::default(),
                    projection: PerspectiveProjection::default().into(),
                    layers: RenderLayers::default(),
                    sprite: Entity::PLACEHOLDER,
                    remaining,
                    signature: Signature {
                        rect: CanvasRect {
                            x: 0,
                            y: 0,
                            width: 64,
                            height: 64,
                            full_width: 64,
                            full_height: 64,
                            offset_x: 0.0,
                            offset_y: 0.0,
                        },
                        transform: Transform::IDENTITY,
                        equipment: Vec::new(),
                    },
                },
            );
        }
        app.insert_resource(cached)
            .insert_resource(super::super::status::SceneAssetsReady(false))
            .add_systems(Update, settle);
        app.update();
        app.world_mut()
            .resource_mut::<super::super::status::SceneAssetsReady>()
            .0 = true;
        app.update();
        assert!(!app.world().get::<Camera>(entities[0]).unwrap().is_active);
        assert!(app.world().get::<Camera>(entities[1]).unwrap().is_active);
        assert!(!app.world().resource::<CachedViews>().is_ready());
    }

    #[test]
    fn layout_sync_preserves_the_compositors_resolved_render_target() {
        let mut app = App::new();
        let mut camera = compositor_camera(true);
        let size = UVec2::new(1440, 1000);
        camera.computed.target_info = Some(bevy::camera::RenderTargetInfo {
            physical_size: size,
            scale_factor: 1.0,
        });
        let compositor = app.world_mut().spawn(camera).id();
        app.insert_resource(CachedViews {
            compositor: Some(compositor),
            ..default()
        })
        .init_resource::<Assets<Image>>()
        .add_systems(Update, |mut commands: Commands, mut cache: CacheWorld| {
            cache.sync(
                &mut commands,
                &[],
                &RetainedScene::default(),
                None,
                &Window::default(),
            );
        });
        app.update();
        let camera = app.world().get::<Camera>(compositor).unwrap();
        assert_eq!(camera.physical_target_size(), Some(size));
        assert!(!camera.is_active);
    }
}

fn settle(
    mut commands: Commands,
    mut cached: ResMut<CachedViews>,
    ready: Res<super::status::SceneAssetsReady>,
    mut cameras: Query<
        (&mut Camera, &bevy::camera::visibility::VisibleEntities),
        With<SnapshotCamera>,
    >,
) {
    // Inactive Camera entities still consume Bevy's limited distance-visibility
    // slots. Reuse a bounded pair for captures, retaining completed images.
    let cached = &mut *cached;
    let mut active = 0;
    for frame in cached.frames.values_mut() {
        if frame.remaining == 0 {
            if let Some(camera) = frame.camera.take() {
                if let Ok((mut view, _)) = cameras.get_mut(camera) {
                    view.is_active = false;
                }
                cached.free_cameras.push(camera);
            }
            continue;
        }
        let Some(camera) = frame.camera else {
            continue;
        };
        active += 1;
        if !ready.0 && frame.remaining > 0 {
            frame.remaining = CAPTURE_SETTLED_FRAMES;
        }
        if let Ok((mut camera, visible)) = cameras.get_mut(camera) {
            // Do not redraw every snapshot throughout city generation. The live
            // view warms shared assets; snapshot pipelines settle when first used.
            camera.is_active = ready.0;
            if camera.is_active {
                if frame.remaining == 1 {
                    info!(
                        meshes = visible.len(std::any::TypeId::of::<Mesh3d>()),
                        "strategic snapshot captured"
                    );
                }
                frame.remaining = frame.remaining.saturating_sub(1);
            }
        }
    }
    if ready.0 {
        for frame in cached
            .frames
            .values_mut()
            .filter(|frame| frame.remaining > 0 && frame.camera.is_none())
            .take(CONCURRENT_CAPTURES.saturating_sub(active))
        {
            frame.start_capture(&mut commands, cached.free_cameras.pop());
        }
    }
}

//! Bounded return-trip images; scene identity includes the environment digest.
use super::*;

const RETAINED_CITY_VIEWS: usize = 2;
const RETAINED_IMAGE_BYTES: u64 = 64 * 1024 * 1024;

pub(super) struct SavedViews {
    identity: (String, String),
    frames: HashMap<ViewKey, Frame>,
}

impl SavedViews {
    fn bytes(&self) -> u64 {
        self.frames
            .values()
            .map(|f| u64::from(f.signature.rect.width) * u64::from(f.signature.rect.height) * 4)
            .sum()
    }

    fn discard(self, commands: &mut Commands) {
        for frame in self.frames.into_values() {
            commands.entity(frame.sprite).despawn();
        }
    }
}

impl CachedViews {
    pub(super) fn change_scene(&mut self, commands: &mut Commands, scene: &RetainedScene) {
        let mut frames = std::mem::take(&mut self.frames);
        for frame in frames.values_mut() {
            if let Some(camera) = frame.camera.take() {
                deactivate(commands, camera);
                self.free_cameras.push(camera);
            }
            commands.entity(frame.sprite).insert(Visibility::Hidden);
        }
        frames.retain(|_, frame| {
            if frame.remaining == 0 {
                true
            } else {
                commands.entity(frame.sprite).despawn();
                false
            }
        });
        if !self.identity.1.is_empty() && !frames.is_empty() {
            self.previous.push_back(SavedViews {
                identity: self.identity.clone(),
                frames,
            });
        } else {
            SavedViews {
                identity: self.identity.clone(),
                frames,
            }
            .discard(commands);
        }
        self.identity = (scene.location.clone(), scene.digest.clone());
        if let Some(index) = self
            .previous
            .iter()
            .position(|saved| saved.identity == self.identity)
        {
            self.frames = self.previous.remove(index).expect("retained city").frames;
        }
        while self.previous.len() > RETAINED_CITY_VIEWS
            || self.previous.iter().map(SavedViews::bytes).sum::<u64>() > RETAINED_IMAGE_BYTES
        {
            if let Some(old) = self.previous.pop_front() {
                old.discard(commands);
            }
        }
        self.scene = scene.root;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_trip_reuses_completed_images_but_changed_scene_or_dimensions_recapture() {
        let mut app = App::new();
        app.init_resource::<CachedViews>()
            .init_resource::<Assets<Image>>()
            .init_resource::<RetainedScene>()
            .add_systems(
                Update,
                |mut commands: Commands, mut cache: CacheWorld, scene: Res<RetainedScene>| {
                    let spec = ViewSpec {
                        cache: Some(ViewKey::Street),
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
                        projection: Projection::default(),
                        layers: RenderLayers::default(),
                    };
                    cache.sync(&mut commands, &[spec], &scene, None, &Window::default());
                },
            );
        let visit = |app: &mut App, name: &str, digest: &str| {
            let root = app.world_mut().spawn_empty().id();
            app.insert_resource(RetainedScene {
                location: name.into(),
                digest: digest.into(),
                root: Some(root),
                ..default()
            });
            app.update();
        };
        visit(&mut app, "a", "daylight-a");
        let image = {
            let mut cache = app.world_mut().resource_mut::<CachedViews>();
            let frame = cache.frames.get_mut(&ViewKey::Street).unwrap();
            frame.remaining = 0;
            frame.image.clone()
        };
        visit(&mut app, "b", "daylight-b");
        visit(&mut app, "a", "daylight-a");
        {
            let cache = app.world().resource::<CachedViews>();
            assert_eq!(cache.frames[&ViewKey::Street].image, image);
            assert!(cache.is_ready());
        }
        // Reused images still pass the ordinary viewport/equipment signature.
        app.world_mut()
            .resource_mut::<CachedViews>()
            .frames
            .get_mut(&ViewKey::Street)
            .unwrap()
            .signature
            .rect
            .full_width = 128;
        app.update();
        assert!(!app.world().resource::<CachedViews>().is_ready());
        visit(&mut app, "a", "night-a");
        assert_ne!(
            app.world().resource::<CachedViews>().frames[&ViewKey::Street].image,
            image
        );
        for index in 0..5 {
            app.world_mut()
                .resource_mut::<CachedViews>()
                .frames
                .get_mut(&ViewKey::Street)
                .unwrap()
                .remaining = 0;
            visit(&mut app, &index.to_string(), &index.to_string());
        }
        assert!(app.world().resource::<CachedViews>().previous.len() <= RETAINED_CITY_VIEWS);
    }
}

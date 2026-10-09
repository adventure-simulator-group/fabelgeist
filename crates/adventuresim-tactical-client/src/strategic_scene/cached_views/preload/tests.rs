use super::*;
use crate::strategic_scene::{
    RetainedPerson,
    protocol::{Person, PersonPresentation, PlaceId, PortraitView},
    status::SceneAssetsReady,
};

#[test]
fn unseen_portraits_finish_before_readiness_and_reuse_their_image_on_first_visit() {
    let rect = CanvasRect {
        x: 20,
        y: 40,
        width: 64,
        height: 64,
        full_width: 64,
        full_height: 64,
        offset_x: 0.0,
        offset_y: 0.0,
    };
    let mut scene = RetainedScene::default();
    let people = (1..=5)
        .map(|id| {
            let id = PortraitId(id);
            scene.people.insert(
                id,
                RetainedPerson {
                    entity: Entity::PLACEHOLDER,
                    anchor: Vec3::X * id.0 as f32,
                    facing: Quat::IDENTITY,
                    layer: id.0 as usize + 1,
                },
            );
            Person {
                id,
                place: PlaceId(id.0.to_string()),
                presentation: PersonPresentation::Scene,
                equipment: Vec::new(),
            }
        })
        .collect();
    let view = StrategicView {
        revision: 1,
        location: "city".into(),
        places: vec![],
        people,
        active_place: None,
        selected: None,
        street: None,
        stage: None,
        forge: None,
        portraits: vec![PortraitView {
            id: PortraitId(1),
            rect,
        }],
    };
    let mut app = App::new();
    super::super::install(&mut app);
    app.init_resource::<Assets<Image>>()
        .insert_resource(scene)
        .insert_resource(view)
        .insert_resource(SceneAssetsReady(true))
        .add_systems(
            Update,
            |mut commands: Commands,
             mut cache: CacheWorld,
             view: Res<StrategicView>,
             scene: Res<RetainedScene>| {
                let visible = specs(&view, &scene)
                    .into_iter()
                    .filter(|spec| {
                        spec.cache
                            == Some(ViewKey::Portrait {
                                person: view.portraits[0].id,
                                slot: 0,
                            })
                    })
                    .collect::<Vec<_>>();
                cache.sync(
                    &mut commands,
                    &visible,
                    &scene,
                    Some(&view),
                    &Window::default(),
                );
            },
        );
    app.update();
    assert_eq!(app.world().resource::<CachedViews>().frames.len(), 5);
    assert!(!app.world().resource::<CachedViews>().is_ready());
    let key = ViewKey::Portrait {
        person: PortraitId(2),
        slot: 0,
    };
    let frame = &app.world().resource::<CachedViews>().frames[&key];
    let (image, sprite) = (frame.image.clone(), frame.sprite);
    assert_eq!(
        app.world().get::<Visibility>(sprite),
        Some(&Visibility::Hidden)
    );
    for _ in 0..32 {
        app.update();
        let world = app.world_mut();
        let cameras = world
            .query_filtered::<&Camera, With<SnapshotCamera>>()
            .iter(world)
            .count();
        assert!(
            cameras <= CONCURRENT_CAPTURES,
            "inactive cameras consume visibility slots too"
        );
        let active = world
            .query_filtered::<&Camera, With<SnapshotCamera>>()
            .iter(world)
            .filter(|camera| camera.is_active)
            .count();
        assert!(active <= CONCURRENT_CAPTURES);
        if world.resource::<CachedViews>().is_ready() {
            break;
        }
    }
    assert!(app.world().resource::<CachedViews>().is_ready());
    app.world_mut().resource_mut::<StrategicView>().portraits[0] = PortraitView {
        id: PortraitId(2),
        rect: CanvasRect { x: 400, ..rect },
    };
    app.update();
    let frame = &app.world().resource::<CachedViews>().frames[&key];
    assert_eq!(frame.image, image);
    assert!(frame.camera.is_none());
    assert_eq!(frame.remaining, 0);
    assert_eq!(
        app.world().get::<Visibility>(sprite),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<&Camera, With<SnapshotCamera>>()
            .iter(app.world())
            .filter(|camera| camera.is_active)
            .count(),
        0
    );
}

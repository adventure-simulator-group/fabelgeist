use super::*;
use crate::strategic_scene::{
    RetainedPerson,
    protocol::{Person, PersonPresentation, PlaceId, PortraitView},
    status::{SceneAssetsReady, SceneInstallationReady},
};
use adventuresim_building_generator::spatial_geometry::Position;

#[test]
fn requested_portraits_capture_on_first_visit_and_reuse_their_completed_images() {
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
                    anchor: Position::from_metres(Vec3::X * id.0 as f32).unwrap(),
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
    super::install(&mut app);
    app.init_resource::<Assets<Image>>()
        .insert_resource(scene)
        .insert_resource(view)
        .insert_resource(SceneAssetsReady(true))
        .insert_resource(SceneInstallationReady(true))
        .add_systems(
            Update,
            |mut commands: Commands,
             mut cache: CacheWorld,
             view: Res<StrategicView>,
             scene: Res<RetainedScene>| {
                let portrait = &view.portraits[0];
                let person = &scene.people[&portrait.id];
                let mut spec = super::super::views::person_view(
                    portrait.rect,
                    person.anchor,
                    person.facing,
                    true,
                    super::super::views::focused_layers(&scene, Some(portrait.id)),
                );
                spec.cache = Some(ViewKey::Portrait {
                    person: portrait.id,
                    slot: 0,
                });
                let visible = [spec];
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
    assert_eq!(app.world().resource::<CachedViews>().frames.len(), 1);
    assert!(!app.world().resource::<CachedViews>().is_ready());
    let key = ViewKey::Portrait {
        person: PortraitId(1),
        slot: 0,
    };
    let frame = &app.world().resource::<CachedViews>().frames[&key];
    let (image, sprite) = (frame.image.clone(), frame.sprite);
    assert_eq!(
        app.world().get::<Visibility>(sprite),
        Some(&Visibility::Inherited)
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
    let new_key = ViewKey::Portrait {
        person: PortraitId(2),
        slot: 0,
    };
    let cache = app.world().resource::<CachedViews>();
    assert_eq!(cache.frames.len(), 2);
    assert!(cache.frames[&new_key].remaining > 0);
    assert!(!cache.is_ready());
    for _ in 0..32 {
        app.update();
        if app.world().resource::<CachedViews>().is_ready() {
            break;
        }
    }
    assert!(app.world().resource::<CachedViews>().is_ready());
    app.world_mut().resource_mut::<StrategicView>().portraits[0] = PortraitView {
        id: PortraitId(1),
        rect,
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

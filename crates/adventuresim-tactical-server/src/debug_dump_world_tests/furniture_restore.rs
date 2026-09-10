use super::*;
use adventuresim_building_generator::furniture::{FurnitureKey, FurnitureKind, FurnitureVariant};

#[test]
fn furniture_dump_restores_collision_and_the_same_activity_groups() {
    let _guard = DUMP_DIR_LOCK.lock().unwrap();
    let input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/furniture-review.json"
    )))
    .unwrap();
    let group = FurnitureGroup {
        id: FurnitureGroupId(71),
        kind: FurnitureGroupKind::Receiving,
        anchor: FurnitureAnchor::Building { id: 1 },
        footprint: FurnitureFootprint {
            centre_metres: Vec2::new(5.0, 7.0),
            half_extents_metres: Vec2::new(2.0, 3.0),
            orientation: BuildingOrientation::IDENTITY,
        },
    };
    let key = FurnitureKey {
        kind: FurnitureKind::Barrel,
        variant: FurnitureVariant::Compact,
    };
    let scene = SceneFurniture {
        id: FurnitureInstanceId(92),
        key,
        group_id: group.id,
    };
    let transform = Transform::from_xyz(5.0, 0.0, 7.0);
    let mut app = App::new();
    app.insert_resource(SceneVistaBundleResource(crate::scene_setup::vista_bundle(
        &input,
    )))
    .add_observer(on_debug_dump_world_request)
    .add_observer(crate::furniture::on_furniture_added)
    .add_observer(crate::furniture::on_group_added)
    .add_observer(crate::furniture::on_vista_furniture_added);
    app.world_mut().spawn((scene, transform));
    app.world_mut().spawn(SceneFurnitureGroup(group.clone()));
    app.world_mut().flush();
    let expected_groups = app
        .world()
        .resource::<SceneVistaBundleResource>()
        .0
        .as_ref()
        .unwrap()
        .furniture_groups
        .clone();
    assert_eq!(expected_groups, vec![group]);

    let distant = GeneratedFurniture {
        scene: SceneFurniture {
            id: FurnitureInstanceId(93),
            ..scene
        },
        position_metres: Vec3::new(250.0, 2.0, 10.0),
        orientation: BuildingOrientation::IDENTITY,
    };
    let vista_entity = app
        .world_mut()
        .spawn(SceneVistaFurniture(vec![distant]))
        .id();
    app.world_mut().flush();
    assert!(app.world().get::<Collider>(vista_entity).is_none());
    assert!(app.world().get::<RigidBody>(vista_entity).is_none());
    let before = dump_dir_snapshot();
    app.world_mut().trigger(FromClient {
        client_id: ClientId::Server,
        message: DebugDumpWorldRequest,
    });
    let dump_path = newest_dump_file(&before);
    let mut restored = App::new();
    restored
        .insert_resource(test_args(Some(dump_path.clone())))
        .insert_resource(SceneVistaBundleResource(crate::scene_setup::vista_bundle(
            &input,
        )))
        .add_observer(crate::furniture::on_furniture_added)
        .add_observer(crate::furniture::on_group_added)
        .add_observer(crate::furniture::on_vista_furniture_added);
    load_world_dump(restored.world_mut());
    restored.world_mut().flush();
    let mut query = restored
        .world_mut()
        .query::<(&SceneFurniture, &Transform, &Collider, &RigidBody)>();
    let (loaded, pose, collider, body) = query
        .single(restored.world())
        .expect("restored furniture must recreate its physical collider");
    assert_eq!(*loaded, scene);
    assert_eq!(*pose, transform);
    assert_eq!(*body, RigidBody::Static);
    let ray_origin = transform.translation + Vec3::new(0.0, 0.4, -3.0);
    assert!(
        collider
            .cast_ray(
                transform.translation,
                Rotation::default(),
                ray_origin,
                Vec3::Z,
                6.0,
                false
            )
            .is_some()
    );
    assert_eq!(
        restored
            .world()
            .resource::<SceneVistaBundleResource>()
            .0
            .as_ref()
            .unwrap()
            .furniture_groups,
        expected_groups
    );
    assert_eq!(
        restored
            .world()
            .resource::<SceneVistaBundleResource>()
            .0
            .as_ref()
            .unwrap()
            .distant_furniture,
        vec![distant]
    );
    let mut vista_query = restored
        .world_mut()
        .query::<(Entity, &SceneVistaFurniture)>();
    let (entity, scenery) = vista_query.single(restored.world()).unwrap();
    assert_eq!(scenery.0, vec![distant]);
    assert!(restored.world().get::<Collider>(entity).is_none());
    std::fs::remove_file(dump_path).unwrap();
}

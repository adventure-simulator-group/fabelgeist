use super::*;
use adventuresim_building_generator::furniture::{FurnitureKey, FurnitureKind, FurnitureVariant};
use adventuresim_tactical_core::scene_input::furniture::FurnitureLocation;

#[test]
fn interior_furniture_restore_preserves_room_identity_and_rebuilds_rotated_collision() {
    let _guard = DUMP_DIR_LOCK.lock().unwrap();
    let scene = SceneFurniture {
        id: FurnitureInstanceId(100_092),
        key: FurnitureKey {
            kind: FurnitureKind::Workbench,
            variant: FurnitureVariant::Compact,
        },
        location: FurnitureLocation::Interior {
            building_id: 81,
            room_id: 7,
            storey: 1,
        },
    };
    let transform = Transform::from_xyz(5.0, 3.0, 7.0)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    let mut app = App::new();
    app.add_observer(on_debug_dump_world_request)
        .add_observer(crate::furniture::on_furniture_added);
    let entity = app.world_mut().spawn((scene, transform)).id();
    app.world_mut().flush();
    assert!(app.world().get::<Collider>(entity).is_some());
    let before = dump_dir_snapshot();
    app.world_mut().trigger(FromClient {
        client_id: ClientId::Server,
        message: DebugDumpWorldRequest,
    });
    let dump_path = newest_dump_file(&before);

    let mut restored = App::new();
    restored
        .insert_resource(test_args(Some(dump_path.clone())))
        .add_observer(crate::furniture::on_furniture_added);
    load_world_dump(restored.world_mut());
    restored.world_mut().flush();
    let mut query = restored
        .world_mut()
        .query::<(&SceneFurniture, &Transform, &Collider, &RigidBody)>();
    let (loaded, pose, collider, body) = query
        .single(restored.world())
        .expect("interior fixtures restore through the production furniture observer");
    assert_eq!(*loaded, scene);
    assert_eq!(*pose, transform);
    assert_eq!(*body, RigidBody::Static);
    let ray_origin = pose.translation + pose.rotation * Vec3::new(0.0, 0.82, -2.0);
    assert!(
        collider
            .cast_ray(
                pose.translation,
                Rotation(pose.rotation),
                ray_origin,
                pose.rotation * Vec3::Z,
                4.0,
                false,
            )
            .is_some(),
        "the elevated, rotated bench top must block a ray after restore"
    );
    let below_origin = pose.translation + pose.rotation * Vec3::new(0.0, -0.1, -2.0);
    assert!(
        collider
            .cast_ray(
                pose.translation,
                Rotation(pose.rotation),
                below_origin,
                pose.rotation * Vec3::Z,
                4.0,
                false,
            )
            .is_none(),
        "an upper-storey fixture must not acquire collision below its own floor"
    );
    std::fs::remove_file(dump_path).unwrap();
}

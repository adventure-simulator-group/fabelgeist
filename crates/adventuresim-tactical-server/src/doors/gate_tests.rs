use super::*;

#[test]
fn property_gate_has_authoritative_collision_hinge_and_passage_control() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../assets/tactical-scenes/compound-review.json"
    ))
    .unwrap();
    assert_eq!(input.compounds.len(), 2);
    for boundary in input.generate().unwrap().boundaries {
        assert_property_gate(boundary);
    }
}

fn assert_property_gate(boundary: GeneratedBoundary) {
    let building_id = boundary.scene().front_building_id();
    let spec = boundary
        .scene()
        .boundary()
        .gate
        .door(boundary.scene().property_id())
        .unwrap();
    let elevation = Vec3::Y * boundary.elevation_metres().metres();
    let spec = adventuresim_tactical_core::scene_coordinates::GateDatum::from_metres(
        boundary.elevation_metres().metres(),
    )
    .unwrap()
    .door(spec)
    .unwrap()
    .leaf();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default(), TransformPlugin));
    app.add_observer(super::super::boundaries::on_scene_boundary_added);
    let anchor = app
        .world_mut()
        .spawn((
            boundary.into_scene(),
            Transform::from_translation(elevation),
        ))
        .id();
    app.world_mut().flush();
    assert_eq!(
        *app.world().get::<RigidBody>(anchor).unwrap(),
        RigidBody::Static
    );
    let mut query = app
        .world_mut()
        .query::<(Entity, &SceneDoor, &Transform, &Collider, &DoorController)>();
    let (entity, door, transform, collider, controller) = query.single(app.world()).unwrap();
    assert_eq!(door.opening_id, spec.opening);
    assert_eq!(door.building_id, building_id);
    let ray = spec.closed_centre.metres() + Vec3::NEG_Z * 2.0;
    assert!(
        collider
            .cast_ray(
                transform.translation,
                Rotation(transform.rotation),
                ray,
                Vec3::Z,
                4.0,
                false
            )
            .is_some()
    );
    let rotation = Quat::from_rotation_y(spec.open_angle_radians.radians());
    let open_centre = spec.hinge_centre.metres()
        + rotation * (spec.closed_centre.metres() - spec.hinge_centre.metres());
    assert!(
        collider
            .cast_ray(
                open_centre,
                Rotation(rotation * transform.rotation),
                ray,
                Vec3::Z,
                4.0,
                false
            )
            .is_none()
    );
    let joint = controller.joint;
    let doorway = controller.doorway_centre.metres();
    let outward = controller.outward.vector();
    let gate_entity = entity;
    let mut exemption = DoorPassageExemptions::default();
    exemption.grant(gate_entity);
    app.world_mut().spawn((
        Transform::from_translation(doorway - outward),
        CharacterController::default(),
        exemption,
    ));
    let mut schedule = Schedule::default();
    schedule.add_systems(update_door_passages);
    schedule.run(app.world_mut());
    assert_eq!(
        app.world()
            .get::<RevoluteJoint>(joint)
            .unwrap()
            .motor
            .target_position,
        spec.open_angle_radians.radians()
    );
}

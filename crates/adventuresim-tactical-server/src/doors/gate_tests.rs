use super::*;

#[test]
fn property_gate_has_authoritative_collision_hinge_and_passage_control() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../assets/tactical-scenes/compound-review.json"
    ))
    .unwrap();
    let compound = &input.compounds[0];
    let spec = compound.boundary.gate.door(compound.id);
    let elevation = Vec3::Y * 3.0;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default(), TransformPlugin));
    app.add_observer(super::super::boundaries::on_scene_boundary_added);
    let anchor = app
        .world_mut()
        .spawn((
            SceneBoundary {
                property_id: compound.id,
                front_building_id: compound.front_building_id,
                boundary: compound.boundary.clone(),
            },
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
    assert_eq!(door.opening_id, spec.opening.0);
    assert_eq!(door.building_id, compound.front_building_id);
    let ray = spec.closed_centre + elevation + Vec3::NEG_Z * 2.0;
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
    let rotation = Quat::from_rotation_y(spec.open_angle_radians);
    let open_centre =
        spec.hinge_centre + elevation + rotation * (spec.closed_centre - spec.hinge_centre);
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
    let doorway = controller.doorway_centre;
    let outward = controller.outward;
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
        spec.open_angle_radians
    );
}

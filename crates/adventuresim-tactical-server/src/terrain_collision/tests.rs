use super::*;
use crate::compound_review_fixture::compound_review_terrain;

#[test]
fn scene_observer_restores_separate_static_support_bodies_at_the_scene_transform() {
    let (terrain, plans) = compound_review_terrain(vec![0.0; 101 * 101]);
    let restored: SceneTerrain =
        serde_json::from_value(serde_json::to_value(&terrain).unwrap()).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default(), TransformPlugin));
    app.add_observer(on_scene_terrain_added);
    let offset = Vec3::new(7.0, 2.0, -3.0);
    let scene = app
        .world_mut()
        .spawn((
            SceneId("terrain-restore-regression".into()),
            Transform::from_translation(offset),
            restored,
        ))
        .id();
    app.update();
    app.update();
    assert!(app.world().get::<Replicated>(scene).is_some());
    assert!(app.world().get::<Collider>(scene).is_none());
    let mut shapes = app
        .world_mut()
        .query::<(&Collider, &GlobalTransform, &RigidBody)>();
    assert_eq!(shapes.iter(app.world()).count(), plans.len() + 1);
    for plan in plans {
        for member in plan.member_support() {
            let point = member.contact.centre_metres();
            let floor = member.elevation.metres() + offset.y;
            let ray = Vec3::new(point.x, floor + 10.0, point.y) + offset.with_y(0.0);
            let mut heights: Vec<_> = shapes
                .iter(app.world())
                .filter_map(|(shape, global, body)| {
                    assert_eq!(*body, RigidBody::Static);
                    let transform = global.compute_transform();
                    shape
                        .cast_ray(
                            transform.translation,
                            Rotation(transform.rotation),
                            ray,
                            Vec3::NEG_Y,
                            20.0,
                            false,
                        )
                        .map(|(distance, _)| ray.y - distance)
                })
                .collect();
            heights.sort_by(f32::total_cmp);
            assert!(
                (heights.last().unwrap() - floor).abs() < 0.001,
                "{heights:?}"
            );
        }
    }
    // Collision lifetime follows the transient scene, including restored shapes.
    app.world_mut().despawn(scene);
    assert_eq!(shapes.iter(app.world()).count(), 0);
}

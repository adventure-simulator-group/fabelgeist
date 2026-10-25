use super::*;
use adventuresim_tactical_core::city_layout::grounding::*;
use adventuresim_tactical_core::city_layout::{
    CitySceneLayout, CompoundGradingPolicy, StreetApronDimensions,
};

#[test]
fn scene_observer_restores_separate_static_support_bodies_at_the_scene_transform() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../assets/tactical-scenes/compound-review.json"
    ))
    .unwrap();
    let sampled = SceneTerrain::from_heightmap(101, 101, 1.0, vec![0.0; 101 * 101]).unwrap();
    let source = sampled.sampled_geographic_surface().unwrap();
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        distant: input.distant_buildings.clone(),
        compounds: input.compounds.clone(),
        ..default()
    };
    let policy = CompoundGradingPolicy {
        limits: SupportLimits::new(
            adventuresim_tactical_core::city_layout::grounding::SupportGrade::from_ratio(0.65)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        stairs: CourtStairLimits::new(
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.19)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.25)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.05)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
                .unwrap(),
        ),
        embedment: FoundationEmbedment::from_metres(0.2).unwrap(),
        street_apron: StreetApronDimensions::from_metres(Vec2::new(1.0, 4.0)).unwrap(),
    };
    let plans = layout.plan_compound_support(&source, policy).unwrap();
    let terrain = sampled.with_property_surface(
        BoundedSettlementTerrain::compile(
            &plans
                .iter()
                .map(|plan| plan.support_surface().unwrap())
                .collect::<Vec<_>>(),
            &source,
            policy.embedment,
        )
        .unwrap(),
    );
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

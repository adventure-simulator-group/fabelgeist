//! A furnished occupied shop must admit the actual humanoid collider and mesh.

use std::time::{Duration, Instant};

use super::*;
use adventuresim_tactical_core::scene_input::{
    GeneratedBuildingRecipe, TacticalBuildingPlacement, compile_tactical_building_collider,
    furniture::{FurnitureLayout, InteriorBuildingLayout},
};
use bevy::ecs::system::SystemState;

fn furnished_shop(
    building: GeneratedBuilding,
    interior: InteriorBuildingLayout,
    establishment: SceneEstablishment,
    terrain: SceneTerrain,
) -> (App, OperatorStations) {
    let stations = OperatorStations::prepare(
        std::slice::from_ref(&building),
        std::slice::from_ref(&interior),
        &[establishment],
    )
    .unwrap();
    let mut furniture = FurnitureLayout::default();
    furniture
        .install_interior(&building, interior.layout)
        .unwrap();
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        PhysicsPlugins::default(),
        crate::openings::BuildingOpeningsPlugin,
        crate::navigation::TacticalNavigationPlugin,
    ));
    app.init_resource::<TacticalCombatConfig>();
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    for collider in terrain.colliders().unwrap() {
        app.world_mut()
            .spawn((RigidBody::Static, collider, Transform::default()));
    }
    app.world_mut().spawn(terrain);
    let transform = building.transform().unwrap();
    let building_entity = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            compile_tactical_building_collider(&building.collision).unwrap(),
            transform,
        ))
        .id();
    crate::openings::spawn_building_openings(
        &mut app.world_mut().commands(),
        building_entity,
        &SceneBuilding {
            id: building.placement.id,
            program: building.placement.program.clone(),
            orientation: building.placement.orientation,
        },
        &transform,
        &building.plan,
        &building.collision,
    )
    .unwrap();
    app.world_mut().flush();
    for instance in furniture.instances {
        app.world_mut().spawn((
            RigidBody::Static,
            furniture_collider(instance.scene.key).unwrap(),
            Transform::from_translation(instance.position_metres.metres())
                .with_rotation(Quat::from_rotation_y(instance.orientation.yaw_radians())),
        ));
    }
    app.finish();
    app.cleanup();
    let deadline = Instant::now() + Duration::from_secs(60);
    while !app.world().resource::<TacticalNavigation>().ready {
        assert!(
            Instant::now() < deadline,
            "furnished shop navigation mesh did not become ready"
        );
        app.update();
        std::thread::yield_now();
    }
    (app, stations)
}

#[test]
fn occupied_shop_admits_a_reachable_operator_and_rejects_a_new_physical_blocker() {
    let (building, interior, establishment) = super::super::placement::tests::merchant();
    let ground = building.placement.base_elevation_metres.metres();
    let terrain = SceneTerrain::from_heightmap(65, 65, 1.0, vec![ground; 65 * 65]).unwrap();
    let (mut app, stations) = furnished_shop(building, interior, establishment, terrain);
    let doors: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<SceneDoor>>()
        .iter(app.world())
        .collect();
    assert!(!doors.is_empty(), "shop must retain its physical doors");
    for door in doors {
        app.world_mut()
            .entity_mut(door)
            .insert(AngularVelocity(Vec3::Y));
    }
    for _ in 0..30 {
        app.update();
        assert!(
            app.world().resource::<TacticalNavigation>().ready,
            "moving operable closures must not invalidate architectural navigation"
        );
    }
    let mut queries = SystemState::<ResidentPlacementWorld>::new(app.world_mut());
    let placement = queries.get(app.world()).unwrap();
    let position = placement
        .operator(&stations.0[0].approaches[0])
        .expect("occupied shop needs an accessible humanoid operator station");
    let collider = crate::player_projection::player_collider();
    let offset = -collider.aabb(default(), Rotation::default()).min.y;
    app.world_mut().spawn((
        Player {
            name: "Customer".into(),
        },
        CharacterId(999),
        RigidBody::Dynamic,
        collider,
        Transform::from_translation(position.metres() + Vec3::Y * offset),
    ));
    for _ in 0..3 {
        app.update();
    }
    let mut queries = SystemState::<ResidentPlacementWorld>::new(app.world_mut());
    let placement = queries.get(app.world()).unwrap();
    assert!(
        placement.supported(position).is_none(),
        "blocked standing volume must not advertise a live operator"
    );
}

#[test]
fn small_town_shop_admits_its_canonical_merchant() {
    let mut placement: TacticalBuildingPlacement =
        serde_json::from_str(include_str!("small-town-shop.json")).unwrap();
    // Shift by whole navigation cells to fit the fixture terrain while keeping
    // the captured shop's orientation and rasterization phase.
    placement.centre_metres =
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            placement.centre_metres.metres() - Vec2::new(40.0, 0.0),
        )
        .unwrap();
    let recipe = GeneratedBuildingRecipe::generate(placement.program.clone()).unwrap();
    let building = GeneratedBuilding {
        placement,
        plan: recipe.plan,
        collision: recipe.collision,
    };
    let (_, mut interior, mut establishment) = super::super::placement::tests::merchant();
    interior.building_id = building.placement.id;
    interior.layout = adventuresim_building_generator::interior::furnish(
        &building.plan,
        &building.placement.program,
    )
    .unwrap();
    establishment.building_id = building.placement.id;
    let ground = building.placement.base_elevation_metres.metres();
    let terrain = SceneTerrain::from_heightmap(65, 65, 1.0, vec![ground; 65 * 65]).unwrap();
    let (mut app, stations) = furnished_shop(building, interior, establishment, terrain);
    let mut queries = SystemState::<ResidentPlacementWorld>::new(app.world_mut());
    let world = queries.get(app.world()).unwrap();
    assert!(
        stations.0[0]
            .approaches
            .iter()
            .any(|approach| world.operator(approach).is_ok()),
        "small town's real shop must have a physically reachable merchant station"
    );
}

//! Traversal uses the production KCC, collider shape and authored movement limits.
//! These checks isolate the proposed support surface; they do not accept a city.
use super::*;
use adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_ahoy::{AhoySystems, CharacterController, input::AccumulatedInput};
use bevy_enhanced_input::EnhancedInputPlugin;
use std::time::Duration;

mod gardens;
mod outdoor;
mod production;
mod single;

struct Walker {
    app: App,
    entity: Entity,
}

#[derive(Component)]
struct WalkGoal(Vec2);

#[derive(Component, Default)]
struct LastDriveVelocity(Vec3);

#[derive(Component)]
struct ExactBuildingGeometry(crate::scene_input::GeneratedBuilding);

fn drive(mut walkers: Query<(&Position, &WalkGoal, &mut AccumulatedInput)>) {
    for (position, goal, mut input) in &mut walkers {
        let delta = goal.0 - position.0.xz();
        // A route waypoint asks the pedestrian to arrive and stop. Taper input
        // over one body radius instead of steering a full-speed U-turn through
        // a narrow doorway with the real motor's bounded turning acceleration.
        let movement = delta.normalize_or_zero()
            * (delta.length() / HUMANOID_COLLISION_RADIUS_METRES).min(1.0);
        input.last_movement = Some(Vec2::new(movement.x, -movement.y));
    }
}

fn capture_drive(mut walkers: Query<(&LinearVelocity, &mut LastDriveVelocity)>) {
    for (velocity, mut sample) in &mut walkers {
        sample.0 = velocity.0;
    }
}

impl Walker {
    fn on_surface(mesh: &PropertySupportMesh, start: Vec2, elevation: f32) -> Self {
        Self::on_collider(mesh.collider(), start, elevation)
    }

    fn on_collider(surface: Collider, start: Vec2, elevation: f32) -> Self {
        Self::on_colliders([surface], start, elevation)
    }

    fn on_colliders(
        surfaces: impl IntoIterator<Item = Collider>,
        start: Vec2,
        elevation: f32,
    ) -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::input::InputPlugin,
            TransformPlugin,
            EnhancedInputPlugin,
            crate::physics::AdventureSimulatorPhysicsPlugin::default(),
        ));
        let tick = Duration::from_secs_f64(1.0 / 64.0);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(tick));
        app.insert_resource(Time::<Fixed>::from_duration(tick));
        app.add_systems(
            FixedPostUpdate,
            drive.before(crate::physics::AdventureSimulatorPhysicsSet::ApplyCharacterMotor),
        );
        app.add_systems(
            FixedPostUpdate,
            capture_drive
                .after(crate::physics::AdventureSimulatorPhysicsSet::ApplyCharacterMotor)
                .before(AhoySystems::MoveCharacters),
        );
        for (index, surface) in surfaces.into_iter().enumerate() {
            app.world_mut().spawn((
                Name::new(format!("support shape {index}")),
                RigidBody::Static,
                surface,
                Transform::default(),
            ));
        }
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let collider = Collider::cylinder(HUMANOID_COLLISION_RADIUS_METRES, 1.9);
        let bottom = -collider.aabb(Vec3::ZERO, Rotation::default()).min.y;
        let controller = CharacterController {
            speed: config.movement.speeds_metres_per_second.walk,
            gravity: motor.gravity_metres_per_second_squared,
            step_size: motor.maximum_step_height_metres,
            min_walk_cos: motor.maximum_walkable_slope_degrees.to_radians().cos(),
            ..default()
        };
        let entity = app
            .world_mut()
            .spawn((
                controller,
                crate::physics::MovementPace::Walk,
                LastDriveVelocity::default(),
                collider,
                Transform::from_xyz(start.x, elevation + bottom + 0.05, start.y),
            ))
            .id();
        app.finish();
        app.cleanup();
        for _ in 0..64 {
            app.update();
        }
        Self { app, entity }
    }

    fn attempt_walk_to(&mut self, target: Vec2) -> Vec3 {
        let start = self
            .app
            .world()
            .get::<Position>(self.entity)
            .unwrap()
            .0
            .xz();
        let speed = crate::combat_config::TacticalCombatConfig::default()
            .movement
            .speeds_metres_per_second
            .walk;
        let tick = self.app.world().resource::<Time<Fixed>>().delta_secs();
        // Preserve the existing ten-second control window. Long geographic
        // approaches need a route-length budget, not a larger movement limit.
        let route_ticks = ((start.distance(target) / speed * 2.0 + 1.0) / tick).ceil() as usize;
        self.app
            .world_mut()
            .entity_mut(self.entity)
            .insert(WalkGoal(target));
        for _ in 0..route_ticks.max(640) {
            let position = self.app.world().get::<Position>(self.entity).unwrap().0;
            let delta = target - position.xz();
            if delta.length() < 0.08 {
                self.app
                    .world_mut()
                    .entity_mut(self.entity)
                    .remove::<WalkGoal>();
                // Use the production motor to brake; do not zero its velocity.
                for _ in 0..64 {
                    if self
                        .app
                        .world()
                        .get::<LinearVelocity>(self.entity)
                        .unwrap()
                        .xz()
                        .length()
                        < 0.01
                    {
                        break;
                    }
                    self.app.update();
                }
                return self.app.world().get::<Position>(self.entity).unwrap().0;
            }
            self.app.update();
        }
        self.app.world().get::<Position>(self.entity).unwrap().0
    }

    fn walk_to(&mut self, target: Vec2) -> Vec3 {
        let position = self.attempt_walk_to(target);
        if position.xz().distance(target) >= 0.08 {
            self.capture_step_casts(position);
        }
        assert!(
            position.xz().distance(target) < 0.08,
            "target {target:?}, stopped at {position:?}, velocity {:?}, input {:?}, transform {:?}",
            self.app.world().get::<LinearVelocity>(self.entity),
            self.app.world().get::<AccumulatedInput>(self.entity),
            self.app.world().get::<Transform>(self.entity),
        );
        position
    }

    fn install_property_buildings(&mut self, plan: &CompoundSupportPlan, fixture: &Fixture) {
        use crate::scene_input::{
            GeneratedBuilding, TacticalBuildingPlacement, compile_tactical_building_collider,
        };
        let value = &fixture.document;
        for key in ["front_distant_placement", "rear_distant_placement"] {
            let distant: DistantBuildingPlacement =
                serde_json::from_value(value[key].clone()).unwrap();
            let mut placement = TacticalBuildingPlacement::from(distant);
            placement.base_elevation_metres = plan
                .member_support()
                .iter()
                .find(|m| m.building_id == placement.id)
                .unwrap()
                .elevation
                .metres();
            let recipe = GeneratedBuildingRecipe::generate(placement.program.clone()).unwrap();
            let building = GeneratedBuilding {
                placement,
                plan: recipe.plan,
                collision: recipe.collision,
            };
            self.app.world_mut().spawn((
                crate::scene_input::SceneBuilding {
                    id: building.placement.id,
                    program: building.placement.program.clone(),
                    orientation: building.placement.orientation,
                },
                RigidBody::Static,
                compile_tactical_building_collider(&building.collision),
                building.transform(),
                ExactBuildingGeometry(building),
            ));
        }
    }

    fn capture_step_casts(&mut self, position: Vec3) {
        let collider = self
            .app
            .world()
            .get::<Collider>(self.entity)
            .unwrap()
            .clone();
        let controller = self
            .app
            .world()
            .get::<CharacterController>(self.entity)
            .unwrap()
            .clone();
        let velocity = self
            .app
            .world()
            .get::<LastDriveVelocity>(self.entity)
            .unwrap()
            .0;
        let tick = self.app.world().resource::<Time<Fixed>>().delta_secs();
        let wish = (self.app.world().get::<WalkGoal>(self.entity).unwrap().0 - position.xz())
            .normalize_or_zero()
            * controller.speed;
        let direction = Vec3::new(wish.x, 0.0, wish.y).normalize_or_zero();
        println!(
            "controller speed {}, grounded/state {:?}, output {:?}, wish {wish:?}, final velocity {:?}",
            controller.speed,
            self.app
                .world()
                .get::<bevy_ahoy::CharacterControllerState>(self.entity),
            self.app
                .world()
                .get::<bevy_ahoy::CharacterControllerOutput>(self.entity),
            self.app.world().get::<LinearVelocity>(self.entity)
        );
        if let Some(ground) = self
            .app
            .world()
            .get::<bevy_ahoy::CharacterControllerState>(self.entity)
            .and_then(|s| s.grounded)
        {
            println!(
                "ground entity {:?}, shape bounds {:?}",
                self.app.world().get::<Name>(ground.entity),
                self.app
                    .world()
                    .get::<Collider>(ground.entity)
                    .map(|c| c.aabb(Vec3::ZERO, Rotation::default()))
            );
        }
        let mut state = bevy::ecs::system::SystemState::<MoveAndSlide>::new(self.app.world_mut());
        let query = state.get(self.app.world()).unwrap();
        let cast = |from, movement| {
            query.cast_move(
                &collider,
                from,
                Quat::IDENTITY,
                movement,
                controller.move_and_slide.skin_width,
                &controller.filter,
            )
        };
        let raised = position + Vec3::Y * controller.step_size;
        println!(
            "drive velocity {velocity:?}, tick {tick}, step {}, skin {}",
            controller.step_size, controller.move_and_slide.skin_width
        );
        println!(
            "up {:?}, ledge {:?}, down {:?}",
            cast(position, Vec3::Y * controller.step_size),
            cast(raised, direction * controller.min_step_ledge_space),
            cast(
                raised + Vec3::new(wish.x, 0.0, wish.y) * tick,
                Vec3::NEG_Y * controller.step_size
            )
        );
        if let Some(hit) = cast(raised, direction * controller.min_step_ledge_space) {
            println!(
                "blocked entity name {:?}, building {:?}",
                self.app.world().get::<Name>(hit.entity),
                self.app
                    .world()
                    .get::<crate::scene_input::SceneBuilding>(hit.entity)
            );
            let Some(ExactBuildingGeometry(building)) = self.app.world().get(hit.entity) else {
                return;
            };
            let point = building
                .transform()
                .compute_affine()
                .inverse()
                .transform_point3(hit.point2)
                + building.collision.bounds.centre();
            for cuboid in &building.collision.cuboids {
                let rotation = Quat::from_rotation_y(cuboid.yaw_radians)
                    * Quat::from_rotation_x(cuboid.crossfall_radians)
                    * Quat::from_rotation_z(cuboid.longfall_radians);
                let offset = rotation.inverse() * (point - cuboid.centre);
                if offset
                    .abs()
                    .cmple(cuboid.size * 0.5 + Vec3::splat(0.001))
                    .all()
                {
                    let solid = building
                        .plan
                        .resolved_geometry
                        .solids
                        .iter()
                        .find(|s| s.id == cuboid.source);
                    println!(
                        "blocked member {}, arch point {point:?}, cuboid {cuboid:?}, solid {solid:?}",
                        building.placement.id
                    );
                }
            }
        }
    }
}

#[test]
fn a_riser_above_the_configured_step_limit_remains_blocked() {
    let surface = Collider::compound(vec![
        (
            Vec3::new(0.0, -0.5, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(4.0, 1.0, 4.0),
        ),
        (
            Vec3::new(0.0, 0.2, -0.5),
            Quat::IDENTITY,
            Collider::cuboid(2.0, 0.4, 1.0),
        ),
    ]);
    let mut walker = Walker::on_collider(surface, Vec2::new(0.0, 0.415), 0.0);
    let position = walker.attempt_walk_to(Vec2::new(0.0, -0.4));
    assert!(
        position.z > 0.4,
        "over-height riser was traversed: {position:?}"
    );
    assert!(position.y - 0.95 < 0.1);
}

#[test]
fn an_ordinary_step_with_insufficient_headroom_remains_blocked() {
    let surface = Collider::compound(vec![
        (
            Vec3::new(0.0, -0.5, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(4.0, 1.0, 4.0),
        ),
        (
            Vec3::new(0.0, 0.09, -0.5),
            Quat::IDENTITY,
            Collider::cuboid(2.0, 0.18, 1.0),
        ),
        (
            Vec3::new(0.0, 2.25, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(4.0, 0.5, 4.0),
        ),
    ]);
    let mut walker = Walker::on_collider(surface, Vec2::new(0.0, 0.415), 0.0);
    let position = walker.attempt_walk_to(Vec2::new(0.0, -0.4));
    assert!(
        position.z > 0.4,
        "blocked headroom was traversed: {position:?}"
    );
    assert!(position.y - 0.95 < 0.1);
}

#[test]
fn pedestrian_starts_at_an_ordinary_riser_below_the_configured_step_limit() {
    let surface = Collider::compound(vec![
        (
            Vec3::new(0.0, -0.5, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(4.0, 1.0, 4.0),
        ),
        (
            Vec3::new(0.0, 0.09, -0.5),
            Quat::IDENTITY,
            Collider::cuboid(2.0, 0.18, 1.0),
        ),
    ]);
    let mut walker = Walker::on_collider(surface, Vec2::new(0.0, 0.415), 0.0);
    walker.walk_to(Vec2::new(0.0, -0.4));
}

#[test]
fn pedestrian_control_traverses_the_same_compound_on_a_level_court() {
    let fixture = Fixture::load();
    let plan = fixture.plan(CourtTreatment::Level);
    let member = plan.member_support()[0];
    let route = fixture
        .property
        .access
        .iter()
        .find(|route| route.ends_at(member.court_threshold_metres))
        .unwrap();
    let mut walker = Walker::on_surface(
        &plan.mesh().unwrap(),
        route.start_metres,
        plan.court_elevation().metres(),
    );
    walker.walk_to(route.end_metres);
}

#[test]
fn goslar_1238_pedestrian_traverses_both_court_stairs_without_jumping() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let mesh = plan.mesh().unwrap();
    for member in plan.member_support() {
        let route = fixture
            .property
            .access
            .iter()
            .find(|route| route.ends_at(member.court_threshold_metres))
            .unwrap();
        let mut walker =
            Walker::on_surface(&mesh, route.start_metres, plan.court_elevation().metres());
        let arrived = walker.walk_to(route.end_metres);
        let bottom = 0.95;
        assert!(
            (arrived.y - bottom - member.elevation.metres()).abs() < 0.08,
            "building {} floor {}, pedestrian feet {}",
            member.building_id,
            member.elevation.metres(),
            arrived.y - bottom
        );
        walker.walk_to(route.start_metres);
    }
}

#[test]
fn goslar_1238_pedestrian_traverses_closed_foundations_without_jumping() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let foundation = plan
        .foundations(
            &super::foundations::geographic_fixture(),
            FoundationEmbedment::from_metres(0.2).unwrap(),
        )
        .unwrap();
    for member in plan.member_support() {
        let route = fixture
            .property
            .access
            .iter()
            .find(|r| r.ends_at(member.court_threshold_metres))
            .unwrap();
        let mut walker = Walker::on_collider(
            foundation.collider(),
            route.start_metres,
            plan.court_elevation().metres(),
        );
        let arrived = walker.walk_to(route.end_metres);
        assert!(
            (arrived.y - 0.95 - member.elevation.metres()).abs() < 0.08,
            "foundation member {}, floor {}, feet {}, arrived {arrived:?}",
            member.building_id,
            member.elevation.metres(),
            arrived.y - 0.95,
        );
        walker.walk_to(route.start_metres);
    }
}

#[test]
fn goslar_1238_pedestrian_enters_from_unchanged_geographic_ground_and_traverses_the_court() {
    traverse_court(&Fixture::load());
}

fn traverse_court(fixture: &Fixture) {
    // Full doorway approach must clear the collider's rear radius before the
    // actor's head reaches the wall. The short surface-only landing is invalid
    // for this entry, even though it admits an isolated threshold-centre goal.
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let terrain = BoundedSettlementTerrain::compile(
        &(std::slice::from_ref(&plan))
            .iter()
            .map(|plan| plan.support_surface().unwrap())
            .collect::<Vec<_>>(),
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    let passage = fixture
        .property
        .access
        .iter()
        .find(|r| r.contains_centreline(fixture.property.boundary.gate.centre_metres))
        .unwrap();
    let mut walker = Walker::on_colliders(
        terrain.colliders(),
        passage.start_metres,
        plan.levels.street.metres(),
    );
    walker.install_property_buildings(&plan, fixture);
    let (boundary, gate) = crate::city_layout::grounding::BoundarySupportMesh::project(
        &fixture.property,
        &terrain.foundations[0],
        SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    walker.app.world_mut().spawn((
        RigidBody::Static,
        boundary.collider(),
        Transform::from_xyz(0.0, gate.metres(), 0.0),
    ));
    let door = fixture.property.boundary.gate.door(fixture.property.id);
    let rotation = Quat::from_rotation_y(door.open_angle_radians);
    let open_centre = door.hinge_centre
        + rotation * (door.closed_centre - door.hinge_centre)
        + Vec3::Y * gate.metres();
    walker.app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(door.size_metres.x, door.size_metres.y, door.size_metres.z),
        Transform::from_translation(open_centre)
            .with_rotation(rotation * Quat::from_rotation_y(door.closed_yaw_radians)),
    ));
    walker.walk_to(fixture.property.boundary.gate.centre_metres);
    walker.walk_to(passage.end_metres);
    for member in plan.member_support() {
        let route = fixture
            .property
            .access
            .iter()
            .find(|r| r.ends_at(member.court_threshold_metres))
            .unwrap();
        walker.walk_to(route.start_metres);
        let arrived = walker.walk_to(route.end_metres);
        assert!(
            (arrived.y - 0.95 - member.elevation.metres()).abs() < 0.08,
            "geographic member {}, floor {}, feet {}",
            member.building_id,
            member.elevation.metres(),
            arrived.y - 0.95
        );
        walker.walk_to(route.start_metres);
    }
    walker.walk_to(passage.end_metres);
    walker.walk_to(fixture.property.boundary.gate.centre_metres);
    walker.walk_to(passage.start_metres);
}

#[test]
fn a_surface_only_door_landing_does_not_count_as_usable_architectural_access() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let terrain = BoundedPropertyTerrain::compile(
        &plan,
        &super::foundations::geographic_fixture(),
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    let rear = plan.member_support()[1];
    let route = fixture
        .property
        .access
        .iter()
        .find(|r| r.ends_at(rear.court_threshold_metres))
        .unwrap();
    let mut walker = Walker::on_colliders(
        terrain.colliders(),
        route.start_metres,
        plan.court_elevation().metres(),
    );
    walker.install_property_buildings(&plan, &fixture);
    let stopped = walker.attempt_walk_to(route.end_metres);
    assert!(
        stopped.xz().distance(route.end_metres) > 0.2,
        "undersized landing unexpectedly permits doorway entry: {stopped:?}"
    );
    assert!(stopped.y - 0.95 - rear.elevation.metres() > 0.1);
}

#[test]
fn goslar_1238_front_street_door_remains_accessible_in_both_directions() {
    traverse_street(&Fixture::load());
}

#[test]
fn goslar_965_uses_its_existing_front_setback_for_geographic_entry_and_return() {
    let fixture = Fixture::load_965();
    traverse_court(&fixture);
    traverse_street(&fixture);
}

#[test]
fn goslar_965_street_door_is_traversable_with_an_unchanged_recipe_on_level_ground() {
    let mut fixture = Fixture::load_965();
    for triangle in fixture.document["geographic_triangles"]
        .as_array_mut()
        .unwrap()
    {
        for vertex in triangle.as_array_mut().unwrap() {
            vertex[1] = serde_json::json!(20.0);
        }
    }
    traverse_street(&fixture);
}

fn traverse_street(fixture: &Fixture) {
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let terrain = BoundedSettlementTerrain::compile(
        &(std::slice::from_ref(&plan))
            .iter()
            .map(|plan| plan.support_surface().unwrap())
            .collect::<Vec<_>>(),
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    let value = &fixture.document;
    let threshold: Vec2 = serde_json::from_value(value["front_street_threshold"].clone()).unwrap();
    let outward = fixture.property.plot.orientation.local_to_world(-Vec2::Y);
    let external_run = value["doorway_solution"]["street_entry_apron"]["dimensions_metres"][1]
        .as_f64()
        .unwrap() as f32;
    let street = threshold + outward * (external_run + 2.0);
    let inside = threshold - outward * 0.65;
    let mut walker = Walker::on_colliders(
        terrain.colliders(),
        street,
        source.elevation_at(street).unwrap().metres(),
    );
    walker.install_property_buildings(&plan, fixture);
    let entered = walker.walk_to(inside);
    assert!((entered.y - 0.95 - plan.member_support()[0].elevation.metres()).abs() < 0.08);
    walker.walk_to(street);
}

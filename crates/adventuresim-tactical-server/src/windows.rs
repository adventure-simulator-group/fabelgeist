//! Authoritative interaction and constrained motion for building casements.

use adventuresim_building_generator::{BuildingCollision, BuildingPlan, compile_operable_windows};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::bevy_replicon::prelude::Replicated;
use bevy::{ecs::system::SystemParam, prelude::*};

const WINDOW_ANGULAR_SPEED_RADIANS_PER_SECOND: f32 = 2.8;

#[derive(Component)]
struct WindowController {
    hinge_centre: Vec3,
    closed_rotation: Quat,
    open_angle_radians: f32,
    current_angle_radians: f32,
    open: bool,
}

pub(crate) struct WindowServerPlugin;

impl Plugin for WindowServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            animate_windows.before(PhysicsSystems::Prepare),
        );
    }
}

#[derive(SystemParam)]
pub(crate) struct WindowGrabber<'w, 's> {
    windows: Query<'w, 's, (&'static SceneWindow, &'static mut WindowController)>,
    characters: Query<'w, 's, &'static Transform, With<Player>>,
}

impl WindowGrabber<'_, '_> {
    pub(crate) fn try_toggle_from_inside(&mut self, actor: Entity, window_entity: Entity) -> bool {
        let Ok(actor_transform) = self.characters.get(actor) else {
            return false;
        };
        let Ok((window, mut controller)) = self.windows.get_mut(window_entity) else {
            return false;
        };
        if !can_grab_window_from_inside(
            actor_transform.translation,
            window.opening_centre_metres.metres(),
            window.tangent.vector(),
            window.outward.vector(),
            window.size_metres.metres().x * 0.5,
        ) {
            return false;
        }
        controller.open = !controller.open;
        debug!(
            actor = ?actor,
            window = ?window_entity,
            building_id = window.building_id.0,
            opening_id = window.opening_id.0,
            open = controller.open,
            "Toggled interior window catch"
        );
        true
    }
}

pub(crate) fn spawn_building_windows(
    commands: &mut Commands,
    building: &SceneBuilding,
    building_transform: &Transform,
    plan: &BuildingPlan,
    collision: &BuildingCollision,
) -> Result {
    use adventuresim_building_generator::spatial_geometry::Position as GeometryPosition;
    use adventuresim_tactical_core::scene_coordinates::{CollisionCentreDatum, Scene};
    let datum = CollisionCentreDatum::new(
        collision.bounds.centre()?,
        GeometryPosition::<Scene>::from_metres(building_transform.translation)?,
        building.orientation,
    )?;
    for window in compile_operable_windows(plan)? {
        spawn_window(commands, building, datum.window(window)?);
    }
    Ok(())
}

fn spawn_window(
    commands: &mut Commands,
    building: &SceneBuilding,
    pose: adventuresim_tactical_core::scene_coordinates::SceneWindowPose,
) {
    let window = pose.leaf();
    let closed_centre = window.closed_centre.metres();
    let hinge_centre = window.hinge_centre.metres();
    let closed_rotation = pose.native_rotation();
    let tangent = window.tangent.spatial();
    let outward = window.outward.spatial();
    commands.spawn((
        Name::new(format!(
            "Building {} window {} casement",
            building.id, window.opening.0
        )),
        Replicated,
        SceneWindow {
            leaf: window.leaf,
            building_id: building.id,
            opening_id: window.opening,
            size_metres: window.size_metres,
            opening_centre_metres: window.closed_centre,
            tangent,
            outward,
            bars: window.bars,
        },
        RigidBody::Kinematic,
        Collider::cuboid(
            window.size_metres.metres().x,
            window.size_metres.metres().y,
            window.size_metres.metres().z,
        ),
        CollisionLayers::new(TACTICAL_WINDOW_LAYER, LayerMask::DEFAULT),
        Transform::from_translation(closed_centre).with_rotation(closed_rotation),
        WindowController {
            hinge_centre,
            closed_rotation,
            open_angle_radians: window.open_angle_radians.radians(),
            current_angle_radians: 0.0,
            open: false,
        },
    ));
}

fn animate_windows(
    time: Res<Time>,
    mut windows: Query<(&SceneWindow, &mut WindowController, &mut Transform)>,
) {
    for (window, mut controller, mut transform) in &mut windows {
        let target = if controller.open {
            controller.open_angle_radians
        } else {
            0.0
        };
        let maximum_step = WINDOW_ANGULAR_SPEED_RADIANS_PER_SECOND * time.delta_secs();
        let delta = (target - controller.current_angle_radians).clamp(-maximum_step, maximum_step);
        controller.current_angle_radians += delta;
        transform.rotation =
            Quat::from_rotation_y(controller.current_angle_radians) * controller.closed_rotation;
        transform.translation = controller.hinge_centre
            + transform.rotation * Vec3::X * window.size_metres.metres().x * 0.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn only_an_inside_actor_can_toggle_the_window_catch() {
        let mut world = World::new();
        let window = world
            .spawn((
                SceneWindow {
                    leaf: adventuresim_building_generator::WindowLeafKind::LeadedGlass,
                    building_id: adventuresim_tactical_core::scene_input::SceneBuildingId::from(1),
                    opening_id: adventuresim_building_generator::OpeningAssemblyId(2),
                    size_metres: adventuresim_building_generator::spatial_geometry::LeafDimensions::from_metres(Vec3::new(1.0, 1.0, 0.025)).unwrap(),
                    opening_centre_metres: adventuresim_building_generator::spatial_geometry::Position::ORIGIN,
                    tangent: adventuresim_building_generator::spatial_geometry::SpatialDirection::from_normalized(Vec3::X).unwrap(),
                    outward: adventuresim_building_generator::spatial_geometry::SpatialDirection::from_normalized(Vec3::Z).unwrap(),
                    bars: adventuresim_building_generator::WindowBarPresence::Absent,
                },
                WindowController {
                    hinge_centre: Vec3::NEG_X * 0.5,
                    closed_rotation: Quat::IDENTITY,
                    open_angle_radians: 1.0,
                    current_angle_radians: 0.0,
                    open: false,
                },
            ))
            .id();
        let inside = world
            .spawn((
                Player {
                    name: "Inside".to_owned(),
                },
                Transform::from_xyz(0.0, 0.0, -0.5),
            ))
            .id();
        let outside = world
            .spawn((
                Player {
                    name: "Outside".to_owned(),
                },
                Transform::from_xyz(0.0, 0.0, 0.5),
            ))
            .id();

        assert!(
            world
                .run_system_once(move |mut windows: WindowGrabber| {
                    windows.try_toggle_from_inside(inside, window)
                })
                .unwrap()
        );
        assert!(world.get::<WindowController>(window).unwrap().open);
        assert!(
            !world
                .run_system_once(move |mut windows: WindowGrabber| {
                    windows.try_toggle_from_inside(outside, window)
                })
                .unwrap()
        );
    }
    #[test]
    fn converted_barred_window_keeps_collider_hinge_and_inward_swing() {
        use adventuresim_building_generator::spatial_geometry::Position;
        use adventuresim_building_generator::{BuildingArchetype, BuildingProgram, generate};
        use adventuresim_tactical_core::scene_coordinates::{CollisionCentreDatum, Scene};
        use adventuresim_tactical_core::scene_input::BuildingOrientation;
        let program = BuildingProgram::fixture(BuildingArchetype::TownHouse, 42);
        let plan = generate(&program).unwrap();
        let building = SceneBuilding {
            id: adventuresim_tactical_core::scene_input::SceneBuildingId::from(8),
            program,
            orientation: BuildingOrientation::from_radians(0.37).unwrap(),
        };
        let datum = CollisionCentreDatum::new(
            Position::from_metres(Vec3::new(2.0, -1.0, 3.0)).unwrap(),
            Position::<Scene>::from_metres(Vec3::new(-7.0, 4.0, 11.0)).unwrap(),
            building.orientation,
        )
        .unwrap();
        let mut leaf = compile_operable_windows(&plan).unwrap()[0];
        leaf.bars = adventuresim_building_generator::WindowBarPresence::Present;
        let pose = datum.window(leaf).unwrap();
        let converted = pose.leaf();
        let mut world = World::new();
        world
            .run_system_once(move |mut commands: Commands| {
                spawn_window(&mut commands, &building, pose)
            })
            .unwrap();
        let entity = world
            .query_filtered::<Entity, With<SceneWindow>>()
            .single(&world)
            .unwrap();
        let window = *world.get::<SceneWindow>(entity).unwrap();
        let controller = world.get::<WindowController>(entity).unwrap();
        assert_eq!(controller.hinge_centre, converted.hinge_centre.metres());
        assert_eq!(controller.closed_rotation, pose.native_rotation());
        assert_eq!(window.opening_id, leaf.opening);
        assert_eq!(window.building_id, 8.into());
        assert_eq!(
            window.bars,
            adventuresim_building_generator::WindowBarPresence::Present
        );
        assert_eq!(
            world.get::<Transform>(entity).unwrap().translation,
            converted.closed_centre.metres()
        );
        let half = world
            .get::<Collider>(entity)
            .unwrap()
            .shape()
            .as_cuboid()
            .unwrap()
            .half_extents;
        assert_eq!(
            Vec3::new(half.x, half.y, half.z),
            leaf.size_metres.metres() * 0.5
        );
        world.get_mut::<WindowController>(entity).unwrap().open = true;
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(1));
        world.insert_resource(time);
        world.run_system_once(animate_windows).unwrap();
        let transform = world.get::<Transform>(entity).unwrap();
        let expected_rotation =
            Quat::from_rotation_y(converted.open_angle_radians.radians()) * pose.native_rotation();
        assert_eq!(transform.rotation, expected_rotation);
        assert_eq!(
            transform.translation,
            converted.hinge_centre.metres()
                + expected_rotation * Vec3::X * leaf.size_metres.metres().x * 0.5
        );
        assert!(
            (transform.translation - converted.closed_centre.metres())
                .dot(converted.outward.spatial().vector())
                < 0.0
        );
    }
}

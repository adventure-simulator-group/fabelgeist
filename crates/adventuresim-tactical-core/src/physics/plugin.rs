//! Bevy wiring for tactical physics: the full simulation for servers and
//! players, a presentation-only solver for client ragdolls, or a read-only
//! collider fixture for tools that never step bodies.

use avian3d::{collider_tree::ColliderTreeSystems, prelude::*};
use bevy::prelude::*;
use bevy_ahoy::{AhoyPlugins, AhoySystems, camera::AhoyCameraPlugin};

use super::apply_character_motor;

pub struct AdventureSimulatorPhysicsPlugin {
    pub enable_simulation: bool,
    /// Runs Avian's solver for explicitly enabled client-only presentation
    /// bodies while keeping every ordinary replicated rigid body disabled.
    pub enable_presentation_simulation: bool,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdventureSimulatorPhysicsSet {
    ApplyCharacterMotor,
}

impl Default for AdventureSimulatorPhysicsPlugin {
    fn default() -> Self {
        Self {
            enable_simulation: true,
            enable_presentation_simulation: false,
        }
    }
}

impl Plugin for AdventureSimulatorPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::combat_config::TacticalCombatConfig>();
        if self.enable_simulation {
            app.add_plugins((
                crate::doors::tactical_physics_plugins(),
                AhoyPlugins::new(FixedPostUpdate),
            ))
            .add_systems(
                FixedPostUpdate,
                apply_character_motor
                    .in_set(AdventureSimulatorPhysicsSet::ApplyCharacterMotor)
                    .before(AhoySystems::MoveCharacters),
            );
        } else if self.enable_presentation_simulation {
            app.add_plugins((PhysicsPlugins::new(FixedPostUpdate), AhoyCameraPlugin))
                .register_required_components::<RigidBody, RigidBodyDisabled>()
                // Optimize the collider trees inside the broad phase, as wasm
                // always does, so the step needs no solver-nested task join
                // before spatial queries see the trees.
                .insert_resource(ColliderTreeOptimization {
                    use_async_tasks: false,
                    ..default()
                })
                // Collision detection and the collider trees keep running for
                // camera and aim queries; the solver and sleeping only matter
                // while a presentation body (a ragdoll part) is enabled.
                .configure_sets(
                    PhysicsSchedule,
                    (PhysicsStepSystems::Solver, PhysicsStepSystems::Sleeping)
                        .run_if(any_presentation_body_simulates),
                );
        } else {
            app.add_plugins((
                PhysicsSchedulePlugin::new(FixedPostUpdate),
                BroadPhaseCorePlugin,
                ColliderHierarchyPlugin,
                ColliderTransformPlugin::new(FixedPostUpdate),
                PhysicsTransformPlugin::new(FixedPostUpdate),
                ColliderBackendPlugin::<Collider>::new(FixedPostUpdate),
                ColliderTreePlugin::<Collider>::default(),
                AhoyCameraPlugin,
            ))
            // `SolverSystems::Finalize` is normally nested by Avian's solver
            // plugin. This read-only fixture omits the solver, so retain the
            // collider-tree completion step in the equivalent physics phase.
            .configure_sets(
                PhysicsSchedule,
                ColliderTreeSystems::EndOptimize.in_set(PhysicsStepSystems::Finalize),
            )
            .register_required_components::<RigidBody, RigidBodyDisabled>();
        }

        #[cfg(feature = "avian_debug")]
        app.add_plugins(PhysicsDebugPlugin)
            .insert_gizmo_config(
                PhysicsGizmos {
                    // Joint gizmos have no per-entity switch, and the only
                    // joints on a client are presentation ragdolls, whose
                    // anchor lines hide the body under review.
                    joint_anchor_color: None,
                    joint_separation_color: None,
                    ..default()
                },
                GizmoConfig {
                    depth_bias: -1.0,
                    ..default()
                },
            )
            .init_resource::<PhysicsLengthUnit>();
    }
}

/// Whether any enabled body can move. Every replicated body carries
/// `RigidBodyDisabled`; presentation ragdoll parts remove it while they live.
fn any_presentation_body_simulates(bodies: Query<&RigidBody, Without<RigidBodyDisabled>>) -> bool {
    bodies.iter().any(|body| !body.is_static())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;
    use bevy_enhanced_input::EnhancedInputPlugin;

    use super::*;

    const START_HEIGHT_METRES: f32 = 10.0;

    fn presentation_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::input::InputPlugin,
            TransformPlugin,
            EnhancedInputPlugin,
            AdventureSimulatorPhysicsPlugin {
                enable_simulation: false,
                enable_presentation_simulation: true,
            },
        ));
        let tick = Duration::from_secs_f64(1.0 / 30.0);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(tick));
        app.insert_resource(Time::<Fixed>::from_duration(tick));
        app.finish();
        app.cleanup();
        app
    }

    fn spawn_ball(app: &mut App, enabled: bool) -> Entity {
        let mut ball = app.world_mut().spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.25),
            Position::from_xyz(0.0, START_HEIGHT_METRES, 0.0),
        ));
        if enabled {
            ball.remove::<RigidBodyDisabled>();
        }
        ball.id()
    }

    fn height_after_steps(app: &mut App, ball: Entity) -> f32 {
        for _ in 0..15 {
            app.update();
        }
        app.world().get::<Position>(ball).unwrap().y
    }

    #[test]
    fn an_enabled_presentation_body_falls() {
        let mut app = presentation_app();
        let ball = spawn_ball(&mut app, true);
        assert!(height_after_steps(&mut app, ball) < START_HEIGHT_METRES - 0.5);
    }

    #[test]
    fn a_replicated_body_stays_where_it_was_put() {
        let mut app = presentation_app();
        let ball = spawn_ball(&mut app, false);
        assert_eq!(height_after_steps(&mut app, ball), START_HEIGHT_METRES);
    }
}

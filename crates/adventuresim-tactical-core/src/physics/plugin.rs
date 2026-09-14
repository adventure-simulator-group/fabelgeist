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
                .register_required_components::<RigidBody, RigidBodyDisabled>();
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

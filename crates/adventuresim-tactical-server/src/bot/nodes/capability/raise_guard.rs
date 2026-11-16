//! Holding a raised weapon guard.

use super::*;

/// Keeps the weapon guard raised while the bot is able to fight, using the
/// same guard state that player input drives. Quicksteps and passive weapon
/// interception both require it. It does not turn the bot; facing is a
/// separate node.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct RaiseGuardNode;

impl BehaviourNode for RaiseGuardNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<RaiseGuardNode>()
        .with_system(maintain_guard)
        .register();
}

fn maintain_guard(
    tasks: Query<TaskRef<RaiseGuardNode>>,
    mut bodies: Query<(&TacticalCombatState, &mut SkeletonState)>,
) {
    for task in &tasks {
        let Ok((state, mut skeleton)) = bodies.get_mut(task.actor()) else {
            continue;
        };
        let guard = if state.is_incapacitated() {
            WeaponGuardState::Lowered
        } else {
            WeaponGuardState::Raised
        };
        set_weapon_guard(&mut skeleton, guard);
    }
}

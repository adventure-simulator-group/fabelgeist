//! Holding a raised weapon guard.

use super::*;

/// Keeps the weapon guard raised while the bot is able to fight, using the
/// same guard state that player input drives. Quicksteps and passive weapon
/// interception both require it. With `stance: WhileEngaged` the guard is
/// lowered while the bot has nothing to fight. It does not turn the bot;
/// facing is a separate node.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct RaiseGuardNode {
    pub stance: GuardStance,
}

impl BehaviourNode for RaiseGuardNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = GuardStance;

    fn build_task(&self) -> Self::Task {
        self.stance
    }
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<RaiseGuardNode>()
        .with_system(maintain_guard)
        .register();
}

fn maintain_guard(tasks: Query<TaskRef<RaiseGuardNode>>, mut steering: SteeringQuery) {
    for task in &tasks {
        if let Ok(mut steering) = steering.get_mut(task.actor()) {
            steering.guard = Some(*task);
        }
    }
}

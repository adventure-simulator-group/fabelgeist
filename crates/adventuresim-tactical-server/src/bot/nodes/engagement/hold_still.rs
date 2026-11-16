//! Stopping in place.

use super::*;

/// Clears the bot's movement intent and succeeds.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct HoldStillNode;

impl BehaviourNode for HoldStillNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<HoldStillNode>()
        .with_setup_observer(hold_still)
        .register();
}

fn hold_still(
    event: On<TaskAdded, TaskWorker<HoldStillNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<HoldStillNode>>,
    mut movement: MovementQuery,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    steer(&mut movement, task.actor(), None);
    cmd.entity(event.entity).insert(TaskStatus::Success);
}

//! Condition: the current target is still the one to fight.

use super::*;

/// Succeeds while the bot's [`BotTarget`] is still the nearest opponent that
/// can fight. A nearer opponent, or the target dropping, fails it.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct IsTargetStillNearestNode;

impl BehaviourNode for IsTargetStillNearestNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<IsTargetStillNearestNode>()
        .with_setup_observer(check_target_still_nearest)
        .register();
}

fn check_target_still_nearest(
    event: On<TaskAdded, TaskWorker<IsTargetStillNearestNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<IsTargetStillNearestNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    let still_nearest = senses
        .target(actor)
        .is_some_and(|target| senses.nearest_opponent(actor) == Some(target));
    cmd.entity(event.entity).insert(status_for(still_nearest));
}

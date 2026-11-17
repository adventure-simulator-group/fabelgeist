//! Condition: the current target is still the one to fight.

use super::*;

/// Succeeds while the bot's [`BotTarget`] is still the opponent its behaviour
/// decision chose. A new choice, or no opponent at all, fails it.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct IsTargetStillChosenNode;

impl BehaviourNode for IsTargetStillChosenNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<IsTargetStillChosenNode>()
        .with_setup_observer(check_target_still_chosen)
        .register();
}

fn check_target_still_chosen(
    event: On<TaskAdded, TaskWorker<IsTargetStillChosenNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<IsTargetStillChosenNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    let still_chosen = senses
        .target(actor)
        .is_some_and(|target| senses.chosen_target(actor) == Some(target));
    cmd.entity(event.entity).insert(status_for(still_chosen));
}

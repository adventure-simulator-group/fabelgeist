//! Taking up the opponent the bot's behaviour decision chose.

use super::*;

/// Targets the opponent chosen by the bot's behaviour decision (see
/// [`BotTarget`]): the nearest perceived opponent, an attacker it remembers,
/// or an ordered target. Fails, and clears the target, when there is none.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct AcquireTargetNode;

impl BehaviourNode for AcquireTargetNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<AcquireTargetNode>()
        .with_setup_observer(acquire_target)
        .register();
}

fn acquire_target(
    event: On<TaskAdded, TaskWorker<AcquireTargetNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<AcquireTargetNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    let chosen = senses.chosen_target(actor);
    match chosen {
        Some(opponent) => cmd.entity(actor).insert(BotTarget(opponent)),
        None => cmd.entity(actor).remove::<BotTarget>(),
    };
    cmd.entity(event.entity)
        .insert(status_for(chosen.is_some()));
}

//! Giving up the fight.

use super::*;

/// Stops the bot and marks it as yielded (see the mission outcome rules),
/// then succeeds. A yielded bot is no longer able to fight.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct YieldNode;

impl BehaviourNode for YieldNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<YieldNode>()
        .with_setup_observer(yield_combat)
        .register();
}

fn yield_combat(
    event: On<TaskAdded, TaskWorker<YieldNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<YieldNode>>,
    mut movement: MovementQuery,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    steer(&mut movement, actor, None);
    cmd.entity(actor).insert(CombatantYielded);
    cmd.trigger(BotContinuationDecisionEvent {
        combatant: actor,
        decision: BotContinuationDecision::Yield,
    });
    cmd.entity(event.entity).insert(TaskStatus::Success);
}

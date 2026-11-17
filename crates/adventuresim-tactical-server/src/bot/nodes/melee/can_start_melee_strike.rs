//! Condition: the bot can start a melee strike now.

use super::*;

/// Succeeds when the bot wields a melee weapon whose attack is available and
/// the target can be reached, with a lunge if needed.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct CanStartMeleeStrikeNode;

impl BehaviourNode for CanStartMeleeStrikeNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<CanStartMeleeStrikeNode>()
        .with_setup_observer(check_can_strike)
        .register();
}

fn check_can_strike(
    event: On<TaskAdded, TaskWorker<CanStartMeleeStrikeNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<CanStartMeleeStrikeNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let can_strike = senses
        .engagement(task.actor())
        .is_some_and(|engagement| engagement.can_start_melee_strike());
    cmd.entity(event.entity).insert(status_for(can_strike));
}

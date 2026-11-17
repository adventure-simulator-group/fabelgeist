//! Condition: the bot holds a melee weapon it can no longer strike with.

use super::*;

/// Succeeds when the bot wields a melee weapon but its injuries leave no
/// available melee attack.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct MeleeStrikeUnavailableNode;

impl BehaviourNode for MeleeStrikeUnavailableNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<MeleeStrikeUnavailableNode>()
        .with_setup_observer(check_strike_unavailable)
        .register();
}

fn check_strike_unavailable(
    event: On<TaskAdded, TaskWorker<MeleeStrikeUnavailableNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<MeleeStrikeUnavailableNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let unavailable = senses.engagement(task.actor()).is_some_and(|engagement| {
        engagement.facts.weapon_is_melee && !engagement.facts.melee_attack_available
    });
    cmd.entity(event.entity).insert(status_for(unavailable));
}

//! Condition: the bot can still fight.

use super::*;

/// Succeeds while the bot is neither incapacitated nor has yielded.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct IsAbleToFightNode;

impl BehaviourNode for IsAbleToFightNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<IsAbleToFightNode>()
        .with_setup_observer(check_able_to_fight)
        .register();
}

fn check_able_to_fight(
    event: On<TaskAdded, TaskWorker<IsAbleToFightNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<IsAbleToFightNode>>,
    combatants: Query<(&TacticalCombatState, Has<CombatantYielded>)>,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let able = combatants
        .get(task.actor())
        .is_ok_and(|(state, yielded)| !state.is_incapacitated() && !yielded);
    cmd.entity(event.entity).insert(status_for(able));
}

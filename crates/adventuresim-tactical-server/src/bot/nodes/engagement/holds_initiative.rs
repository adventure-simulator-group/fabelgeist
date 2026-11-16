//! Condition: the bot still holds the initiative.

use super::*;

/// Succeeds until one of the bot's attacks is turned into a defense (see
/// [`InitiativeLost`]).
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct HoldsInitiativeNode;

impl BehaviourNode for HoldsInitiativeNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<HoldsInitiativeNode>()
        .with_setup_observer(check_initiative)
        .register();
}

fn check_initiative(
    event: On<TaskAdded, TaskWorker<HoldsInitiativeNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<HoldsInitiativeNode>>,
    lost: Query<(), With<InitiativeLost>>,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    cmd.entity(event.entity)
        .insert(status_for(!lost.contains(task.actor())));
}

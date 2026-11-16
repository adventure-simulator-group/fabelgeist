//! Choosing the opponent to fight.

use super::*;

/// Targets the nearest opponent that can still fight (see [`BotTarget`]).
/// Fails, and clears the target, when there is none.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct AcquireNearestOpponentNode;

impl BehaviourNode for AcquireNearestOpponentNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<AcquireNearestOpponentNode>()
        .with_setup_observer(acquire_nearest_opponent)
        .register();
}

fn acquire_nearest_opponent(
    event: On<TaskAdded, TaskWorker<AcquireNearestOpponentNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<AcquireNearestOpponentNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    let nearest = senses.nearest_opponent(actor);
    match nearest {
        Some(opponent) => cmd.entity(actor).insert(BotTarget(opponent)),
        None => cmd.entity(actor).remove::<BotTarget>(),
    };
    cmd.entity(event.entity)
        .insert(status_for(nearest.is_some()));
}

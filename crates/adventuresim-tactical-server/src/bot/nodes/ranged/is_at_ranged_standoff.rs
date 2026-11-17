//! Condition: the bot stands at its ranged standoff distance.

use super::*;

/// Succeeds when the target is within weapon reach and inside the standoff
/// band configured for ranged attackers.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct IsAtRangedStandoffNode;

impl BehaviourNode for IsAtRangedStandoffNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<IsAtRangedStandoffNode>()
        .with_setup_observer(check_at_standoff)
        .register();
}

fn check_at_standoff(
    event: On<TaskAdded, TaskWorker<IsAtRangedStandoffNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<IsAtRangedStandoffNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let in_position = senses.engagement(task.actor()).is_some_and(|engagement| {
        ranged_standoff_movement(
            engagement.distance_metres,
            engagement.facts.weapon_reach,
            senses.offense_config(),
        )
        .is_none()
    });
    cmd.entity(event.entity).insert(status_for(in_position));
}

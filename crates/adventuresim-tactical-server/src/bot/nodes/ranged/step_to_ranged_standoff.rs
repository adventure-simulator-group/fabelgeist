//! Moving toward the ranged standoff distance.

use super::*;

/// Steps toward or away from the target, for one frame, to reach the ranged
/// standoff band. Fails without a target.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct StepToRangedStandoffNode;

impl BehaviourNode for StepToRangedStandoffNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<StepToRangedStandoffNode>()
        .with_setup_observer(step_to_standoff)
        .register();
}

fn step_to_standoff(
    event: On<TaskAdded, TaskWorker<StepToRangedStandoffNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<StepToRangedStandoffNode>>,
    senses: BotSenses,
    mut steering: SteeringQuery,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    let Some(engagement) = senses.engagement(actor) else {
        cmd.entity(event.entity).insert(TaskStatus::Failure);
        return;
    };
    let intent = ranged_standoff_movement(
        engagement.distance_metres,
        engagement.facts.weapon_reach,
        senses.offense_config(),
    );
    steer(&mut steering, actor, intent);
    cmd.entity(event.entity).insert(TaskStatus::Success);
}

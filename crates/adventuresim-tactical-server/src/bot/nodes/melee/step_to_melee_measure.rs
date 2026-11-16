//! Closing to, or opening to, striking measure.

use super::*;

/// Steps toward the target for one frame, or away from it when a long
/// weapon is inside its preferred striking measure. Fails without a target.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct StepToMeleeMeasureNode;

impl BehaviourNode for StepToMeleeMeasureNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<StepToMeleeMeasureNode>()
        .with_setup_observer(step_to_measure)
        .register();
}

fn step_to_measure(
    event: On<TaskAdded, TaskWorker<StepToMeleeMeasureNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<StepToMeleeMeasureNode>>,
    senses: BotSenses,
    mut movement: MovementQuery,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    let Some(engagement) = senses.engagement(actor) else {
        cmd.entity(event.entity).insert(TaskStatus::Failure);
        return;
    };
    let too_close = engagement.facts.weapon_is_melee
        && below_preferred_long_weapon_measure(
            engagement.facts.weapon_reach,
            engagement.facts.preferred_melee_measure,
            engagement.distance_metres,
            senses.offense_config().long_weapon_measure_threshold_metres,
        );
    let intent = if too_close { RETREAT } else { ADVANCE };
    steer(&mut movement, actor, Some(intent));
    cmd.entity(event.entity).insert(TaskStatus::Success);
}

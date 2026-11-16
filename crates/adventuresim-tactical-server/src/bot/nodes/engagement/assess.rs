//! Assessing the opponent before taking the initiative.

use super::*;

/// Holds still for a seeded initiative delay, which a higher Instinct
/// shortens, then succeeds. Starting an assessment regains any
/// [`InitiativeLost`]; losing the initiative again meanwhile restarts the
/// delay.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct AssessNode;

/// Task for [`AssessNode`].
#[derive(Debug, Default, Reflect)]
pub struct AssessTask {
    delay: Timer,
}

impl BehaviourNode for AssessNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = AssessTask;

    fn build_task(&self) -> Self::Task {
        AssessTask::default()
    }
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<AssessNode>()
        .with_setup_observer(begin_assessment)
        .with_system(assess)
        .register();
}

fn begin_assessment(
    event: On<TaskAdded, TaskWorker<AssessNode>>,
    mut cmd: Commands,
    mut tasks: Query<TaskMut<AssessNode>>,
    senses: BotSenses,
    mut random: ResMut<CombatRandom>,
    mut movement: MovementQuery,
) {
    let Ok(mut task) = tasks.get_mut(event.entity) else {
        return;
    };
    let actor = task.actor();
    steer(&mut movement, actor, None);
    if !restart_delay(&mut cmd, &mut task, &senses, &mut random) {
        cmd.entity(event.entity).insert(TaskStatus::Failure);
    }
}

/// A defense committed during the assessment costs the initiative again, so
/// the delay restarts once rather than forcing a second assessment afterward.
fn assess(
    mut cmd: Commands,
    time: Res<Time<()>>,
    mut tasks: Query<TaskMut<AssessNode>>,
    lost: Query<(), With<InitiativeLost>>,
    senses: BotSenses,
    mut random: ResMut<CombatRandom>,
    mut movement: MovementQuery,
) {
    for mut task in &mut tasks {
        let actor = task.actor();
        let entity = task.entity();
        steer(&mut movement, actor, None);
        if lost.contains(actor) {
            if !restart_delay(&mut cmd, &mut task, &senses, &mut random) {
                cmd.entity(entity).insert(TaskStatus::Failure);
            }
            continue;
        }
        task.delay.tick(time.delta());
        if task.delay.is_finished() {
            cmd.entity(entity).insert(TaskStatus::Success);
        }
    }
}

/// Regains the initiative and samples a fresh delay. Returns whether the
/// delay could be scheduled (failures are logged).
fn restart_delay(
    cmd: &mut Commands,
    task: &mut TaskMut<AssessNode>,
    senses: &BotSenses,
    random: &mut CombatRandom,
) -> bool {
    let actor = task.actor();
    cmd.entity(actor).remove::<InitiativeLost>();
    let delay = initiative_delay_seconds(
        random,
        senses.assessment_instinct(actor),
        senses.offense_config(),
    )
    .and_then(AnimationElapsed::native_duration);
    match delay {
        Ok(delay) => {
            task.delay = timer_after(delay);
            true
        }
        Err(error) => {
            warn!(?actor, %error, "cannot schedule bot initiative");
            false
        }
    }
}

//! Drawing, loosing, and recovering from a ranged shot.

use super::*;

/// Starts a shot at the target, waits out the configured draw while the
/// server completes the committed shot, then waits out the configured
/// cooldown. Fails if no shot can start.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct RangedShotNode;

/// Task for [`RangedShotNode`]. `Starting` only exists until the setup
/// observer schedules the shot.
#[derive(Debug, Default, Reflect)]
pub enum RangedShotTask {
    #[default]
    Starting,
    Drawing {
        draw: Timer,
        cooldown: Duration,
    },
    Cooling(Timer),
}

impl BehaviourNode for RangedShotNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = RangedShotTask;

    fn build_task(&self) -> Self::Task {
        RangedShotTask::default()
    }
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<RangedShotNode>()
        .with_setup_observer(begin_draw)
        .with_system(shoot)
        .register();
}

fn begin_draw(
    event: On<TaskAdded, TaskWorker<RangedShotNode>>,
    mut cmd: Commands,
    mut tasks: Query<TaskMut<RangedShotNode>>,
    senses: BotSenses,
    mut random: ResMut<CombatRandom>,
    mut steering: SteeringQuery,
) {
    let Ok(mut task) = tasks.get_mut(event.entity) else {
        return;
    };
    let actor = task.actor();
    steer(&mut steering, actor, None);
    let Some(engagement) = senses.engagement(actor) else {
        cmd.entity(event.entity).insert(TaskStatus::Failure);
        return;
    };
    match schedule_shot(senses.offense_config(), &mut random) {
        Ok((windup, scheduled)) => {
            cmd.trigger(RangedAttackStartedIntent {
                attacker: actor,
                target: Some(engagement.target),
                animation_windup: windup,
                minimum_windup: windup,
            });
            *task = scheduled;
        }
        Err(error) => {
            warn!(?actor, %error, "cannot schedule bot ranged shot");
            cmd.entity(event.entity).insert(TaskStatus::Failure);
        }
    }
}

fn shoot(
    mut cmd: Commands,
    time: Res<Time<()>>,
    mut tasks: Query<TaskMut<RangedShotNode>>,
    mut steering: SteeringQuery,
) {
    for mut task in &mut tasks {
        let actor = task.actor();
        let entity = task.entity();
        steer(&mut steering, actor, None);
        let next = match &mut *task {
            RangedShotTask::Starting => None,
            RangedShotTask::Drawing { draw, cooldown } => {
                draw.tick(time.delta());
                draw.is_finished()
                    .then(|| RangedShotTask::Cooling(timer_after(*cooldown)))
            }
            RangedShotTask::Cooling(cooldown) => {
                cooldown.tick(time.delta());
                if cooldown.is_finished() {
                    cmd.entity(entity).insert(TaskStatus::Success);
                }
                None
            }
        };
        if let Some(next) = next {
            *task = next;
        }
    }
}

/// Times the draw and the cooldown, with the seeded cadence jitter added to
/// the cooldown. Returns the draw as the animation windup alongside the task.
fn schedule_shot(
    config: &AiOffenseConfig,
    random: &mut CombatRandom,
) -> ElapsedResult<(CombatDuration, RangedShotTask)> {
    let jitter = AnimationElapsed::from_seconds(
        random.range_f32(0.0, config.cadence_jitter_seconds.seconds()),
    )?;
    let task = RangedShotTask::Drawing {
        draw: timer_after(config.windup_seconds.native_duration()?),
        cooldown: config
            .cooldown_seconds
            .checked_add(jitter)?
            .native_duration()?,
    };
    Ok((CombatDuration::from_elapsed(config.windup_seconds)?, task))
}

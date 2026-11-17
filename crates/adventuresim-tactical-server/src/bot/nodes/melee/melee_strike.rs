//! Winding up, delivering, and recovering from a melee strike.

use super::*;

/// Starts the weapon's preferred strike at the target, waits out the windup
/// (or the lunge needed to arrive, if longer), then the weapon's recovery.
/// A long weapon keeps backing off during the windup while the target is
/// inside its preferred measure. Fails if no strike can start, or if the
/// strike stops being possible during the windup.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct MeleeStrikeNode;

struct ScheduledStrike {
    animation_windup: CombatDuration,
    task: MeleeStrikeTask,
}

/// Task for [`MeleeStrikeNode`]. `Starting` only exists until the setup
/// observer schedules the strike.
#[derive(Debug, Default, Reflect)]
pub enum MeleeStrikeTask {
    #[default]
    Starting,
    WindingUp {
        windup: Timer,
        recovery: Duration,
    },
    Recovering(Timer),
}

impl BehaviourNode for MeleeStrikeNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = MeleeStrikeTask;

    fn build_task(&self) -> Self::Task {
        MeleeStrikeTask::default()
    }
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<MeleeStrikeNode>()
        .with_setup_observer(begin_strike)
        .with_system(strike)
        .register();
}

fn begin_strike(
    event: On<TaskAdded, TaskWorker<MeleeStrikeNode>>,
    mut cmd: Commands,
    mut tasks: Query<TaskMut<MeleeStrikeNode>>,
    senses: BotSenses,
    mut random: ResMut<CombatRandom>,
    mut steering: SteeringQuery,
) {
    let Ok(mut task) = tasks.get_mut(event.entity) else {
        return;
    };
    let actor = task.actor();
    let config = senses.offense_config();
    let Some(engagement) = senses.engagement(actor) else {
        cmd.entity(event.entity).insert(TaskStatus::Failure);
        return;
    };
    let Some(plan) = engagement.strike_plan() else {
        cmd.entity(event.entity).insert(TaskStatus::Failure);
        return;
    };
    let Some(reported_precision) = ReportedPrecision::new(config.hit_precision) else {
        warn!(?actor, "bot hit precision is not finite");
        cmd.entity(event.entity).insert(TaskStatus::Failure);
        return;
    };
    steer(&mut steering, actor, engagement.windup_movement(config));
    match schedule_strike(&plan, config, &mut random) {
        Ok(scheduled) => {
            cmd.trigger(MeleeAttackStartedIntent {
                attacker: actor,
                target: Some(engagement.target),
                windup: scheduled.animation_windup,
                reported_precision,
                strike_family: plan.strike_family,
                hand: AttackHand::Main,
            });
            *task = scheduled.task;
        }
        Err(error) => {
            warn!(?actor, %error, "cannot schedule bot melee strike");
            cmd.entity(event.entity).insert(TaskStatus::Failure);
        }
    }
}

fn strike(
    mut cmd: Commands,
    time: Res<Time<()>>,
    mut tasks: Query<TaskMut<MeleeStrikeNode>>,
    senses: BotSenses,
    mut steering: SteeringQuery,
) {
    let config = senses.offense_config();
    for mut task in &mut tasks {
        let actor = task.actor();
        let entity = task.entity();
        let next = match &mut *task {
            MeleeStrikeTask::Starting => None,
            MeleeStrikeTask::WindingUp { windup, recovery } => {
                let Some(engagement) = senses
                    .engagement(actor)
                    .filter(|engagement| engagement.can_start_melee_strike())
                else {
                    cmd.entity(entity).insert(TaskStatus::Failure);
                    continue;
                };
                steer(&mut steering, actor, engagement.windup_movement(config));
                windup.tick(time.delta());
                windup
                    .is_finished()
                    .then(|| MeleeStrikeTask::Recovering(timer_after(*recovery)))
            }
            MeleeStrikeTask::Recovering(recovery) => {
                steer(&mut steering, actor, None);
                recovery.tick(time.delta());
                if recovery.is_finished() {
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

/// Times the windup (no shorter than the lunge) and the recovery, with the
/// seeded cadence jitter added to the recovery.
fn schedule_strike(
    plan: &StrikePlan,
    config: &AiOffenseConfig,
    random: &mut CombatRandom,
) -> ElapsedResult<ScheduledStrike> {
    let jitter = AnimationElapsed::from_seconds(
        random.range_f32(0.0, config.cadence_jitter_seconds.seconds()),
    )?;
    Ok(ScheduledStrike {
        animation_windup: CombatDuration::from_elapsed(config.windup_seconds)?,
        task: MeleeStrikeTask::WindingUp {
            windup: timer_after(config.windup_seconds.max(plan.arrival).native_duration()?),
            recovery: plan.recovery.checked_add(jitter)?.native_duration()?,
        },
    })
}

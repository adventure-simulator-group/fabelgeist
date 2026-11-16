//! Backing away when the bot can no longer strike.

use super::*;

/// Backs away from the target for `duration`, then succeeds. Fails early if
/// a melee attack becomes available again.
#[derive(Debug, Reflect, Clone, Copy)]
pub struct WithdrawNode {
    pub duration: Duration,
}

/// Task for [`WithdrawNode`].
#[derive(Debug, Default, Reflect)]
pub struct WithdrawTask {
    remaining: Timer,
}

impl Default for WithdrawNode {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(1),
        }
    }
}

impl BehaviourNode for WithdrawNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = WithdrawTask;

    fn build_task(&self) -> Self::Task {
        WithdrawTask {
            remaining: timer_after(self.duration),
        }
    }
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<WithdrawNode>()
        .with_setup_observer(begin_withdrawal)
        .with_system(withdraw)
        .register();
}

fn begin_withdrawal(
    event: On<TaskAdded, TaskWorker<WithdrawNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<WithdrawNode>>,
    mut movement: MovementQuery,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    steer(&mut movement, actor, Some(RETREAT));
    cmd.trigger(BotContinuationDecisionEvent {
        combatant: actor,
        decision: BotContinuationDecision::Withdraw,
    });
}

fn withdraw(
    mut cmd: Commands,
    time: Res<Time<()>>,
    mut tasks: Query<TaskMut<WithdrawNode>>,
    senses: BotSenses,
    mut movement: MovementQuery,
) {
    for mut task in &mut tasks {
        let actor = task.actor();
        let entity = task.entity();
        if senses
            .engagement(actor)
            .is_some_and(|engagement| engagement.facts.melee_attack_available)
        {
            cmd.entity(entity).insert(TaskStatus::Failure);
            continue;
        }
        task.remaining.tick(time.delta());
        if task.remaining.is_finished() {
            steer(&mut movement, actor, None);
            cmd.entity(entity).insert(TaskStatus::Success);
        } else {
            steer(&mut movement, actor, Some(RETREAT));
        }
    }
}

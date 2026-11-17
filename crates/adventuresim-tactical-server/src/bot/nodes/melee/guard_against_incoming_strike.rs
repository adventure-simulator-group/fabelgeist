//! Holding guard through a strike the bot has recognized.

use super::*;

/// Holds still behind the raised guard until the target's strike passes the
/// half of its windup, ends, or the target is gone.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct GuardAgainstIncomingStrikeNode;

impl BehaviourNode for GuardAgainstIncomingStrikeNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<GuardAgainstIncomingStrikeNode>()
        .with_system(guard_against_incoming_strike)
        .register();
}

fn guard_against_incoming_strike(
    mut cmd: Commands,
    tasks: Query<TaskRef<GuardAgainstIncomingStrikeNode>>,
    senses: BotSenses,
    mut steering: SteeringQuery,
) {
    for task in &tasks {
        let actor = task.actor();
        steer(&mut steering, actor, None);
        let still_incoming = senses
            .target(actor)
            .and_then(|target| senses.skeleton(target))
            .is_some_and(strike_is_incoming);
        if !still_incoming {
            cmd.entity(task.entity()).insert(TaskStatus::Success);
        }
    }
}

//! Condition: the bot can shoot its target.

use super::*;

/// Succeeds when the bot wields a usable ranged weapon and carries arrows.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct CanShootTargetNode;

impl BehaviourNode for CanShootTargetNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<CanShootTargetNode>()
        .with_setup_observer(check_can_shoot)
        .register();
}

fn check_can_shoot(
    event: On<TaskAdded, TaskWorker<CanShootTargetNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<CanShootTargetNode>>,
    senses: BotSenses,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let can_shoot = senses
        .engagement(task.actor())
        .is_some_and(|engagement| engagement.facts.use_ranged);
    cmd.entity(event.entity).insert(status_for(can_shoot));
}

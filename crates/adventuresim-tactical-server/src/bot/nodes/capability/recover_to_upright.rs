//! Getting back up after a knockdown.

use super::*;

/// Starts the authored get-up transition whenever the bot is down but able to
/// stand.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct RecoverToUprightNode;

impl BehaviourNode for RecoverToUprightNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<RecoverToUprightNode>()
        .with_system(recover_to_upright)
        .register();
}

fn recover_to_upright(
    tasks: Query<TaskRef<RecoverToUprightNode>>,
    mut bodies: Query<(&TacticalCombatState, &mut SkeletonState)>,
    config: Res<TacticalCombatConfig>,
) {
    for task in &tasks {
        let Ok((state, mut skeleton)) = bodies.get_mut(task.actor()) else {
            continue;
        };
        if !state.is_incapacitated() && !skeleton.is_posture_transitioning() {
            begin_get_up_transition_configured(&mut skeleton, &config);
        }
    }
}

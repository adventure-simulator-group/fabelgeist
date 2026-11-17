//! Facing the current target.

use super::*;

/// Turns the bot toward its [`BotTarget`] every frame. Movement intents are
/// relative to this facing.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct FaceTargetNode;

impl BehaviourNode for FaceTargetNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<FaceTargetNode>()
        .with_system(face_target)
        .register();
}

fn face_target(
    tasks: Query<TaskRef<FaceTargetNode>>,
    senses: BotSenses,
    mut steering: SteeringQuery,
) {
    for task in &tasks {
        let actor = task.actor();
        let Some(yaw) = senses
            .target(actor)
            .and_then(|target| senses.yaw_toward(actor, target))
        else {
            continue;
        };
        face(&mut steering, actor, yaw);
    }
}

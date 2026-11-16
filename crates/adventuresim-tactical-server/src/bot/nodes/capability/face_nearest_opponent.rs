//! Facing the nearest opponent without engaging it.

use super::*;

/// Turns the bot toward the nearest opponent every frame while it is able to
/// fight.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct FaceNearestOpponentNode;

impl BehaviourNode for FaceNearestOpponentNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<FaceNearestOpponentNode>()
        .with_system(face_nearest_opponent)
        .register();
}

fn face_nearest_opponent(
    tasks: Query<TaskRef<FaceNearestOpponentNode>>,
    senses: BotSenses,
    mut looks: Query<&mut CharacterLook>,
) {
    for task in &tasks {
        let actor = task.actor();
        if senses.is_incapacitated(actor) {
            continue;
        }
        let Some(yaw) = senses
            .nearest_opponent(actor)
            .and_then(|opponent| senses.yaw_toward(actor, opponent))
        else {
            continue;
        };
        face(&mut looks, actor, yaw);
    }
}

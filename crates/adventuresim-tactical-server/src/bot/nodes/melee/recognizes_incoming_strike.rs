//! Condition: the bot notices a strike already on its way.

use super::*;

/// Succeeds when the target is early in a committed strike that the bot
/// could otherwise trade into, and the bot reads it. The chance grows with how
/// much of the windup is visible and with the bot's Instinct.
///
/// This stops seeded cadence from always handing the initiative to the same
/// combatant; strikes may still overlap when the read fails.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct RecognizesIncomingStrikeNode;

impl BehaviourNode for RecognizesIncomingStrikeNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<RecognizesIncomingStrikeNode>()
        .with_setup_observer(read_incoming_strike)
        .register();
}

fn read_incoming_strike(
    event: On<TaskAdded, TaskWorker<RecognizesIncomingStrikeNode>>,
    mut cmd: Commands,
    tasks: Query<TaskRef<RecognizesIncomingStrikeNode>>,
    senses: BotSenses,
    mut random: ResMut<CombatRandom>,
) {
    let Ok(task) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    // Most frames no strike is coming; skip the attack facts then.
    let incoming = senses
        .target(actor)
        .and_then(|target| senses.skeleton(target))
        .is_some_and(strike_is_incoming);
    let recognized = incoming
        && senses.engagement(actor).is_some_and(|engagement| {
            engagement.facts.weapon_is_melee
                && engagement.melee_target_reachable()
                && random.unit_f32()
                    < committed_threat_recognition_probability(
                        engagement.target_skeleton.action_phase(),
                        engagement.facts.instinct,
                    )
        });
    cmd.entity(event.entity).insert(status_for(recognized));
}

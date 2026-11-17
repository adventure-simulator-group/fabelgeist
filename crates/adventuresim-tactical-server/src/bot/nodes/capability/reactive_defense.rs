//! Reflex dodges against attacks the bot sees coming.

use super::*;

/// Grants the bot reflex defense while it runs. The reactions themselves are
/// scheduled by the defense observers when an attack starts (see
/// [`ReactiveDefenseAi`]).
#[derive(Debug, Reflect, Clone, Copy)]
pub struct ReactiveDefenseNode {
    /// Ignore attacks from outside the bot's narrow frontal arc.
    pub requires_facing: bool,
    pub dodge: DodgeTendency,
}

/// How often a reactive bot reads an attack correctly and dodges it.
#[derive(Debug, Default, Reflect, Clone, Copy, PartialEq, Eq)]
pub enum DodgeTendency {
    /// The ordinary dodge chance from the combat configuration.
    #[default]
    Configured,
    /// Every attack, timed to dodge inside the attacker's windup.
    Always,
}

impl Default for ReactiveDefenseNode {
    fn default() -> Self {
        Self {
            requires_facing: true,
            dodge: DodgeTendency::Configured,
        }
    }
}

impl BehaviourNode for ReactiveDefenseNode {
    type Info<'a> = LeafNodeInfo<'a>;
    type Task = ();

    fn build_task(&self) -> Self::Task {}
}

impl DodgeTendency {
    const CERTAIN: f64 = 1.0;

    fn chances(self, config: &TacticalCombatConfig) -> DefenseChances {
        DefenseChances {
            dodge_chance: match self {
                Self::Configured => config.ai.ordinary.defense.dodge_chance,
                Self::Always => Self::CERTAIN,
            },
        }
    }
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<ReactiveDefenseNode>()
        .with_setup_observer(grant_reactive_defense)
        .register();
}

/// Existing [`DefenseChances`] are kept, so a restored world dump or a
/// debugging override survives the tree starting.
fn grant_reactive_defense(
    event: On<TaskAdded, TaskWorker<ReactiveDefenseNode>>,
    mut cmd: Commands,
    tasks: Query<(TaskRef<ReactiveDefenseNode>, NodeRef<ReactiveDefenseNode>)>,
    chances: Query<(), With<DefenseChances>>,
    config: Res<TacticalCombatConfig>,
) {
    let Ok((task, node)) = tasks.get(event.entity) else {
        return;
    };
    let actor = task.actor();
    let mut actor_commands = cmd.entity(actor);
    actor_commands.insert(ReactiveDefenseAi {
        requires_facing: node.requires_facing,
    });
    if !chances.contains(actor) {
        actor_commands.insert(node.dodge.chances(&config));
    }
}

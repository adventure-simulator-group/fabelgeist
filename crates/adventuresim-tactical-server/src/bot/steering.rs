//! What the tree asks the bot's body to do this frame.
//!
//! `kisya_edbt` runs trees in `Update` without exposing a system set, so the
//! nodes cannot be ordered inside [`ControlSet`]. Nodes therefore write their
//! requests here, and [`apply_bot_steering`] applies them in
//! `ControlSet::Execute`, after behaviour decisions and before navigation
//! turns a permitted advance into a route.

use crate::control::{AgentState, ControlSet, ThreatMemory};

use super::*;

/// The movement, facing, and guard the bot's tree currently requests.
#[derive(Component, Debug, Default)]
pub(crate) struct BotSteering {
    /// Movement intent relative to facing. Held until a node changes it.
    pub(super) movement: Option<Vec2>,
    /// Facing to turn to. Applied once.
    pub(super) yaw: Option<f32>,
    /// Guard the bot keeps while able to fight. Held until a node changes it.
    pub(super) guard: Option<GuardStance>,
}

/// When a bot holds its weapon guard raised.
#[derive(Debug, Default, Reflect, Clone, Copy, PartialEq, Eq)]
pub enum GuardStance {
    /// Whenever it is able to fight.
    #[default]
    Always,
    /// While it has a target, remembers an attack, or is attacking or
    /// blocking.
    WhileEngaged,
}

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Behaviour, BotSteering>()
        .add_systems(Update, apply_bot_steering.in_set(ControlSet::Execute));
}

type SteeredBots<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut BotSteering,
        &'static mut AuthoritativeMovementIntent,
        &'static mut CharacterLook,
        &'static mut SkeletonState,
        &'static TacticalCombatState,
        Option<&'static AgentState>,
        Option<&'static ThreatMemory>,
    ),
>;

fn apply_bot_steering(mut bots: SteeredBots) {
    for (mut steering, mut movement, mut look, mut skeleton, state, agent, memory) in &mut bots {
        movement.0 = steering.movement;
        if let Some(yaw) = steering.yaw.take() {
            look.yaw = yaw;
        }
        let Some(stance) = steering.guard else {
            continue;
        };
        let engaged = match stance {
            GuardStance::Always => true,
            GuardStance::WhileEngaged => {
                agent.is_some_and(|agent| agent.target().is_some())
                    || memory.is_some_and(|memory| memory.threat.is_some())
                    || matches!(
                        skeleton.action_kind(),
                        SkeletonAction::Attack | SkeletonAction::Block
                    )
            }
        };
        let guard = if engaged && !state.is_incapacitated() {
            WeaponGuardState::Raised
        } else {
            WeaponGuardState::Lowered
        };
        set_weapon_guard(&mut skeleton, guard);
    }
}

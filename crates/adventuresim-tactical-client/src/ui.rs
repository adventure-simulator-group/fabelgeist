//! The player's combat HUD: incapacitation wheels, the aim reticle, weapon
//! guard, attack timer and result, limb condition and the tactical outcome.
//!
//! Each system computes what the HUD shows and pushes it to the browser page
//! through [`crate::hud`] when it changed. The page draws it in DOM; the native
//! client has no HUD.
use adventuresim_tactical_core::{
    player::{CharacterId, Player},
    prelude::*,
};
use adventuresim_tactical_netcode::{
    client::WeaponGuardInputState,
    message::{SuccessfulAttackResponse, TacticalOutcome, TacticalOutcomeResponse},
    prelude::*,
};
use bevy::{prelude::*, transform::TransformSystems};
use serde::Serialize;

mod wheel;

use crate::{
    camera::CameraAimState,
    hud::{self, HudText, HudView, LastPushed, Reticle},
    player::{AttackState, ClientPlayer},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                push_reticle,
                push_weapon_guard,
                push_attack_timer,
                push_limbs,
            ),
        )
        // Enemy wheels follow heads on screen, so they read this frame's
        // propagated transforms.
        .add_systems(
            PostUpdate,
            wheel::push_incapacitation_wheels.after(TransformSystems::Propagate),
        )
        .add_observer(clear_outcome_for_new_session)
        .add_observer(on_tactical_outcome_display)
        .add_observer(on_successful_attack_display);
    }
}

fn push_reticle(aim: Res<CameraAimState>, mut last: Local<LastPushed<Reticle>>) {
    let state = match aim.0 {
        None => Reticle::Hidden,
        Some(observation) if observation.blocked() => Reticle::MuzzleBlocked,
        Some(_) => Reticle::Aiming,
    };
    if let Some(state) = last.update(state) {
        hud::reticle(*state);
    }
}

fn weapon_guard_label(guard: WeaponGuardState, ranged: bool) -> &'static str {
    match guard {
        WeaponGuardState::Lowered => "Guard: Lowered",
        WeaponGuardState::Raised if ranged => "Guard: Aiming",
        WeaponGuardState::Raised => "Guard: Blocking",
    }
}

fn push_weapon_guard(
    guard: Res<WeaponGuardInputState>,
    players: Query<Entity, With<ControlledPlayer>>,
    viewer: TacticalPlayerViewer,
    mut last: Local<LastPushed<&'static str>>,
) {
    let ranged = players
        .iter()
        .next()
        .and_then(|entity| viewer.get(entity).ok())
        .is_some_and(|player| player.weapon_is_ranged());
    if let Some(label) = last.update(weapon_guard_label(guard.desired, ranged)) {
        hud::text(HudText::Guard, label);
    }
}

fn push_attack_timer(
    player: Query<&AttackState, With<ClientPlayer>>,
    mut last: Local<LastPushed<String>>,
) {
    let text = player
        .single()
        .ok()
        .filter(|state| state.is_attacking())
        .map(|state| format!("{:.1}s", state.pre_hit_timer.remaining().as_secs_f32()))
        .unwrap_or_default();
    if let Some(text) = last.update(text) {
        hud::text(HudText::AttackTimer, text);
    }
}

fn limb_percentages(limbs: &Limbs) -> [f32; 7] {
    [
        limbs.head,
        limbs.chest,
        limbs.stomach,
        limbs.left_arm,
        limbs.right_arm,
        limbs.left_leg,
        limbs.right_leg,
    ]
    .map(|limb| (limb * 100.0).round())
}

fn push_limbs(
    player: Query<&Limbs, (With<ClientPlayer>, Changed<Limbs>)>,
    mut last: Local<LastPushed<[f32; 7]>>,
) {
    let Ok(limbs) = player.single() else {
        return;
    };
    if let Some(percent) = last.update(limb_percentages(limbs)) {
        hud::limbs(percent);
    }
}

/// Text tone of a HUD line; the page colours each one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Tone {
    Success,
    Error,
}

#[derive(Debug, PartialEq, Serialize)]
struct OutcomeView {
    label: &'static str,
    tone: Tone,
}

impl From<TacticalOutcome> for OutcomeView {
    fn from(outcome: TacticalOutcome) -> Self {
        match outcome {
            TacticalOutcome::Victory => Self {
                label: "VICTORY",
                tone: Tone::Success,
            },
            TacticalOutcome::Defeat => Self {
                label: "DEFEAT",
                tone: Tone::Error,
            },
        }
    }
}

fn push_outcome(outcome: Option<TacticalOutcome>) {
    if let Ok(json) = serde_json::to_string(&outcome.map(OutcomeView::from)) {
        hud::view(HudView::Outcome, &json);
    }
}

fn on_tactical_outcome_display(event: On<TacticalOutcomeResponse>) {
    push_outcome(Some(event.outcome));
}

/// The browser keeps one page across sessions, so a new session starts
/// without the last one's banner.
fn clear_outcome_for_new_session(_client: On<Add, AdventureSimulatorClient>) {
    push_outcome(None);
}

/// "Attacking <target>: success", then what the attack did.
#[derive(Debug, PartialEq, Serialize)]
struct AttackResultView {
    target: String,
    /// The target's display colour, CSS hex.
    color: String,
    verdict: Tone,
    detail: String,
}

impl AttackResultView {
    fn new(event: &SuccessfulAttackResponse, target: &Player, id: &CharacterId) -> Self {
        let (verdict, detail) = match event.result {
            AttackResult::ToAttacker { balance_damage, .. } => {
                let reason = match event.defender_response {
                    DefenderResponse::None => "Missed",
                    DefenderResponse::Block { .. } => "Blocked",
                    DefenderResponse::Dodge { .. } => "Dodged",
                    DefenderResponse::Parry { .. } => "Parried",
                };
                (
                    Tone::Error,
                    format!(
                        "{reason}! Got {balance_damage:.1} balance damage\n\n[part: {}]",
                        event.body_part
                    ),
                )
            }
            AttackResult::ToDefender {
                cut_damage,
                blunt_damage,
                balance_damage,
                ..
            } => (
                Tone::Success,
                format!(
                    "Dealt..\n{:.1} damage ({cut_damage:.1}C + {blunt_damage:.1}B)\n{balance_damage:.1} balance damage\n\n[part: {} | flanking: {:.1}]",
                    cut_damage + blunt_damage,
                    event.body_part,
                    event.flanking
                ),
            ),
        };
        Self {
            target: target.name.clone(),
            color: id.color().to_srgba().to_hex(),
            verdict,
            detail,
        }
    }
}

fn on_successful_attack_display(
    event: On<SuccessfulAttackResponse>,
    targets: Query<(&Player, &CharacterId)>,
) {
    let Some((target, id)) = event.hit.first().and_then(|e| targets.get(*e).ok()) else {
        return;
    };
    if let Ok(json) = serde_json::to_string(&AttackResultView::new(&event, target, id)) {
        hud::view(HudView::AttackResult, &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_outcome_labels_are_unambiguous() {
        assert_eq!(
            serde_json::to_string(&OutcomeView::from(TacticalOutcome::Victory)).unwrap(),
            r#"{"label":"VICTORY","tone":"success"}"#
        );
        assert_eq!(
            serde_json::to_string(&OutcomeView::from(TacticalOutcome::Defeat)).unwrap(),
            r#"{"label":"DEFEAT","tone":"error"}"#
        );
    }

    #[test]
    fn guard_label_names_the_raised_stance_by_weapon() {
        assert_eq!(
            weapon_guard_label(WeaponGuardState::Lowered, true),
            "Guard: Lowered"
        );
        assert_eq!(
            weapon_guard_label(WeaponGuardState::Raised, true),
            "Guard: Aiming"
        );
        assert_eq!(
            weapon_guard_label(WeaponGuardState::Raised, false),
            "Guard: Blocking"
        );
    }

    #[test]
    fn limb_readout_is_whole_percent_in_body_order() {
        let limbs = Limbs {
            head: 0.994,
            left_leg: 0.125,
            ..default()
        };
        let percent = limb_percentages(&limbs);
        assert_eq!(percent[0], 99.0);
        assert_eq!(percent[5], 13.0);
    }
}

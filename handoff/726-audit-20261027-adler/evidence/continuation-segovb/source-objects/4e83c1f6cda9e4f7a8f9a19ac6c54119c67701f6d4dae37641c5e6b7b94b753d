//! Typed battle timeline and native sample-tick quantization.

use super::*;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MeleeTimelineEvent {
    pub sequence: u64,
    pub tick: u64,
    pub time_seconds: BattleTime,
    pub kind: MeleeTimelineKind,
    pub combatant_id: Option<u64>,
    pub target_id: Option<u64>,
    pub engagement_distance_before_metres: Option<f32>,
    pub engagement_distance_after_metres: Option<f32>,
    pub movement_action: Option<MeleeMovementAction>,
    pub movement_elapsed_seconds: Option<AnimationElapsed>,
    pub movement_displacement_metres: Option<f32>,
    pub movement_velocity_before_metres_per_second: Option<SignedSpeed>,
    pub movement_velocity_after_metres_per_second: Option<SignedSpeed>,
    pub movement_speed_limit_metres_per_second: Option<NonnegativeSpeed>,
    pub readiness_before_seconds: Option<BattleTime>,
    pub readiness_after_seconds: Option<BattleTime>,
    pub attack_id: Option<u64>,
    pub attack_started_tick: Option<u64>,
    pub attack_contact_tick: Option<u64>,
    pub attack_recovery_tick: Option<u64>,
    pub phase_before: Option<MeleeTimelinePhase>,
    pub phase_after: Option<MeleeTimelinePhase>,
    pub response_choice: Option<MeleeResponseChoice>,
    pub committed_finish_trade_probability: Option<f32>,
    pub committed_completed_work_fraction: Option<f32>,
    pub committed_expected_intercept_benefit: Option<f32>,
    pub consecutive_intercepts_before: Option<u8>,
    pub consecutive_intercepts_after: Option<u8>,
    /// One-shot guarded hesitation added after a repeated phase interception.
    /// This is elapsed readiness time, not uncharged action work.
    pub phase_adaptation_delay_seconds: Option<AnimationElapsed>,
    pub response_availability: Option<MeleeResponseAvailability>,
    pub affected_attack_id: Option<u64>,
    pub simultaneous_batch_id: Option<u64>,
    pub simultaneous_members: Vec<u64>,
    pub simultaneous_order: Option<u32>,
    pub terminal_resolution: Option<BattleResolution>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleeTimelineKind {
    Movement,
    AttackStarted,
    Response,
    AttackCanceled,
    AttackTransformed,
    Contact,
    Terminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleeTimelinePhase {
    NeutralGuard,
    Windup,
    Recovery,
    UnableToContinue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleeResponseAvailability {
    NeutralGuard,
    ReciprocalWindup,
    OccupiedByEarlierAttack,
    OccupiedRecovery,
    NoImplement,
    DodgeChosen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleeResponseChoice {
    None,
    Block,
    Parry,
    Dodge,
    FinishTrade,
}

impl MeleeTimelineEvent {
    const STEPS_PER_SECOND: f32 = 64.0;

    pub(super) fn at(kind: MeleeTimelineKind, time_seconds: BattleTime) -> Self {
        Self {
            sequence: 0,
            tick: Self::tick_at(time_seconds),
            time_seconds,
            kind,
            combatant_id: None,
            target_id: None,
            engagement_distance_before_metres: None,
            engagement_distance_after_metres: None,
            movement_action: None,
            movement_elapsed_seconds: None,
            movement_displacement_metres: None,
            movement_velocity_before_metres_per_second: None,
            movement_velocity_after_metres_per_second: None,
            movement_speed_limit_metres_per_second: None,
            readiness_before_seconds: None,
            readiness_after_seconds: None,
            attack_id: None,
            attack_started_tick: None,
            attack_contact_tick: None,
            attack_recovery_tick: None,
            phase_before: None,
            phase_after: None,
            response_choice: None,
            committed_finish_trade_probability: None,
            committed_completed_work_fraction: None,
            committed_expected_intercept_benefit: None,
            consecutive_intercepts_before: None,
            consecutive_intercepts_after: None,
            phase_adaptation_delay_seconds: None,
            response_availability: None,
            affected_attack_id: None,
            simultaneous_batch_id: None,
            simultaneous_members: Vec::new(),
            simultaneous_order: None,
            terminal_resolution: None,
        }
    }

    pub(super) fn tick_at(time: BattleTime) -> u64 {
        (time.seconds().max(0.0) * Self::STEPS_PER_SECOND).round() as u64
    }

    pub(super) fn with_terminal(mut self, resolution: BattleResolution) -> Self {
        self.terminal_resolution = Some(resolution);
        self
    }
}

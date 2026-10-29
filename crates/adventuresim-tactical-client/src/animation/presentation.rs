use super::*;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
use adventuresim_tactical_core::animation::{AnimationSampleTick, NonnegativeLength};
use attack::advance_presented_attack;
use cadence::{AuthoredCadence, apply_authored_cadence};
use events::emit_locomotion_presentation_events;
pub(crate) use events::{LocomotionPresentationEvent, LocomotionPresentationPayload};
pub(super) use events::{
    coalesced_contacts, latest_coalesced_landing, trace_locomotion_presentation_events,
};

#[cfg(test)]
pub(super) use events::CoalescedContact;

mod attack;
mod cadence;
mod events;
pub(super) mod plugin;

pub struct TacticalAnimationPlugin;

/// Client-only locomotion state used for rendering between replicated server
/// samples. Gameplay semantics and presentation events continue to read the
/// authoritative `SkeletonState` directly.
#[derive(Component, Debug, Clone, derive_more::Deref)]
pub(crate) struct PresentedSkeleton {
    #[deref]
    pub(crate) state: SkeletonState,
    source_tick: AnimationSampleTick,
    presentation_tick: Option<AnimationSampleTick>,
    pub(crate) phase_error_remaining: CycleDelta,
    pub(crate) last_phase_prediction_delta: CycleDelta,
    pub(crate) last_phase_correction_delta: CycleDelta,
    pub(crate) last_phase_measurement_error: Option<CycleDelta>,
    pub(crate) last_phase_source_changed: bool,
    authored_cadence: Option<AuthoredCadence>,
    quickstep_phase: Option<UnitInterval>,
    attack_phase: Option<PresentedAttackPhase>,
}

#[derive(Debug, Clone, Copy)]
struct PresentedAttackPhase {
    start_tick: AnimationSampleTick,
    animation: AttackAnimation,
    hand: AttackHand,
    continuation: bool,
    phase: UnitInterval,
    error_remaining: ActionProgressDelta,
}

impl PresentedAttackPhase {
    fn matches(self, state: &SkeletonState) -> bool {
        state.action_kind() == SkeletonAction::Attack
            && state.action_start_tick() == Some(self.start_tick)
            && state.attack_animation() == Some(self.animation)
            && state.attack_hand() == self.hand
            && state.attack_is_continuation() == self.continuation
    }
}

impl PresentedSkeleton {
    pub(super) fn new(
        state: SkeletonState,
        presentation_tick: Option<AnimationSampleTick>,
    ) -> Self {
        let source_tick = state.locomotion_sample_tick;
        Self {
            state,
            source_tick,
            presentation_tick,
            phase_error_remaining: CycleDelta::ZERO,
            last_phase_prediction_delta: CycleDelta::ZERO,
            last_phase_correction_delta: CycleDelta::ZERO,
            last_phase_measurement_error: None,
            last_phase_source_changed: false,
            authored_cadence: None,
            quickstep_phase: None,
            attack_phase: None,
        }
    }
    fn predict_gait(
        &mut self,
        previous: &SkeletonState,
        authoritative: &SkeletonState,
        next: &mut SkeletonState,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<()> {
        let delta_seconds = elapsed.seconds();
        let playback = playback_tuning();
        let source_gap = authoritative
            .locomotion_sample_tick
            .checked_delta(self.source_tick);
        let source_is_contiguous =
            source_gap.is_some_and(|gap| gap <= playback.maximum_source_gap_ticks);
        if source_is_contiguous && can_predict_locomotion(previous, authoritative)? {
            let response = playback
                .velocity_response_per_second
                .exponential_fraction(elapsed)?
                .fraction();
            next.local_velocity = previous
                .local_velocity
                .lerp(authoritative.local_velocity, response);
            next.world_velocity = previous
                .world_velocity
                .lerp(authoritative.world_velocity, response);

            let prediction_delta = if next.weapon_guard() == WeaponGuardState::Raised
                && let Some(step) = next.raised_footwork().step()
            {
                let duration_ticks = step
                    .contact_tick()
                    .saturating_sub(step.start_tick())
                    .max(AnimationSampleSpan::ONE);
                CycleDelta::from_arithmetic(
                    delta_seconds * locomotion_sample_rate().hertz() * 0.5
                        / duration_ticks.samples() as f32,
                )?
            } else {
                let speed = presentation_phase_speed(next)?;
                gait_cycle_phase_delta(
                    locomotion_profile(next, next.animation_speed()?)?,
                    speed,
                    elapsed,
                )?
            };
            let predicted = previous
                .gait_phase
                .checked_advance(prediction_delta)?
                .wrapped();
            self.last_phase_prediction_delta = prediction_delta;
            let error = circular_phase_delta(predicted, authoritative.gait_phase)?;
            let discontinuous = error.cycles().abs() > playback.phase_snap_error.phase_units();
            if self.last_phase_source_changed && discontinuous {
                next.gait_phase = authoritative.gait_phase;
                self.phase_error_remaining = CycleDelta::ZERO;
                self.last_phase_measurement_error = Some(error);
            } else {
                if self.last_phase_source_changed {
                    self.last_phase_measurement_error = Some(error);
                    let deadband = playback.phase_drift_deadband.phase_units();
                    let measured_drift = if error.cycles().abs() <= deadband {
                        0.0
                    } else {
                        error.cycles() - error.cycles().signum() * deadband
                    };
                    // Keep one continuous correction accumulator. Replacing it
                    // with every packet's error makes packet timing modulate the
                    // displayed gait speed even when physical speed is constant.
                    let measured_drift = CycleDelta::from_arithmetic(measured_drift)?;
                    let difference = measured_drift.checked_sub(self.phase_error_remaining)?;
                    let adjustment = CycleDelta::from_arithmetic(
                        difference.cycles() * playback.phase_drift_measurement_blend.fraction(),
                    )?;
                    self.phase_error_remaining =
                        self.phase_error_remaining.checked_add(adjustment)?;
                }
                let maximum_correction = playback
                    .phase_correction_rate_per_second
                    .phase_units_per_second()
                    * delta_seconds;
                let correction = CycleDelta::from_arithmetic(
                    self.phase_error_remaining
                        .cycles()
                        .clamp(-maximum_correction, maximum_correction),
                )?;
                next.gait_phase = predicted.checked_advance(correction)?.wrapped();
                self.phase_error_remaining = self.phase_error_remaining.checked_sub(correction)?;
                self.last_phase_correction_delta = correction;
            }
        } else {
            self.phase_error_remaining = CycleDelta::ZERO;
        }
        Ok(())
    }
}

pub(super) fn playback_tuning() -> AnimationPlaybackConfig {
    runtime_animation_config().playback
}

pub(super) fn maximum_authored_stance_slip_metres() -> NonnegativeLength {
    playback_tuning().maximum_authored_stance_slip_metres
}

pub(super) fn circular_phase_delta(
    from: CyclePhase,
    to: CyclePhase,
) -> AnimationValueResult<CycleDelta> {
    let difference = to.checked_difference(from)?;
    CycleDelta::from_arithmetic((difference.cycles() + 0.5).rem_euclid(1.0) - 0.5)
}

pub(super) fn can_predict_locomotion(
    previous: &SkeletonState,
    authoritative: &SkeletonState,
) -> AnimationValueResult<bool> {
    let action_is_predictable = authoritative.action_kind() == SkeletonAction::None
        || (authoritative.weapon_guard() == WeaponGuardState::Raised
            && authoritative.action_kind() == SkeletonAction::Attack);
    Ok(authoritative.is_surface_supported()
        && action_is_predictable
        && (authoritative.posture() == Posture::Upright || authoritative.is_downed_turning())
        && authoritative.animation_speed()? > STATIONARY_ANIMATION_SPEED_LIMIT
        && previous.posture() == authoritative.posture()
        && previous.weapon_guard() == authoritative.weapon_guard()
        && previous.action_kind() == authoritative.action_kind()
        && previous.is_surface_supported() == authoritative.is_surface_supported()
        && previous.animation_pack == authoritative.animation_pack
        && previous.contact_sequence == authoritative.contact_sequence
        && same_guard_step(previous, authoritative))
}

pub(super) fn advance_presented_skeleton(
    presented: &mut PresentedSkeleton,
    authoritative: &SkeletonState,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<()> {
    advance_presented_skeleton_with_strides(
        presented,
        authoritative,
        elapsed,
        &AuthoredLocomotionStrides::default(),
    )
}

pub(super) fn update_presented_skeletons(
    mut commands: Commands,
    time: Res<Time>,
    procedural_clock: Res<ProceduralAnimationClock>,
    strides: pose_buffer::CalibrationLookup,
    mut players: Query<(Entity, &SkeletonState, Option<&mut PresentedSkeleton>), AnimatedActors>,
) {
    for (entity, authoritative, presented) in &mut players {
        let Some(mut presented) = presented else {
            commands.entity(entity).insert(PresentedSkeleton::new(
                authoritative.clone(),
                procedural_clock.fixed_step().map(|step| step.tick),
            ));
            continue;
        };
        let delta_seconds = if let Some(step) = procedural_clock.fixed_step() {
            let tick = step.tick;
            let advances = presented.presentation_tick != Some(tick);
            presented.presentation_tick = Some(tick);
            if advances {
                step.delta_seconds
            } else {
                AnimationElapsed::ZERO
            }
        } else {
            AnimationElapsed::from(time.delta())
        };
        if let Err(error) = advance_presented_skeleton_with_strides(
            &mut presented,
            authoritative,
            delta_seconds,
            strides.for_actor(entity),
        ) {
            warn!(?entity, %error, "Rejected presentation interval");
        }
    }
}

fn same_guard_step(previous: &SkeletonState, authoritative: &SkeletonState) -> bool {
    match (
        previous.raised_footwork().step(),
        authoritative.raised_footwork().step(),
    ) {
        (Some(previous), Some(authoritative)) => {
            previous.swing_foot() == authoritative.swing_foot()
                && previous.start_tick() == authoritative.start_tick()
                && previous.contact_tick() == authoritative.contact_tick()
        }
        (None, None) => true,
        _ => false,
    }
}

fn advance_presented_skeleton_with_strides(
    presented: &mut PresentedSkeleton,
    authoritative: &SkeletonState,
    elapsed: AnimationElapsed,
    authored_strides: &AuthoredLocomotionStrides,
) -> AnimationValueResult<()> {
    let elapsed = elapsed.capped_by_seconds(0.1)?;
    let source_changed = presented.source_tick != authoritative.locomotion_sample_tick;
    presented.last_phase_prediction_delta = CycleDelta::ZERO;
    presented.last_phase_correction_delta = CycleDelta::ZERO;
    presented.last_phase_measurement_error = None;
    presented.last_phase_source_changed = source_changed;
    let previous = presented.state.clone();
    let mut next = authoritative.clone();

    presented.predict_gait(&previous, authoritative, &mut next, elapsed)?;

    advance_presented_quickstep(
        &previous,
        authoritative,
        &mut next,
        &mut presented.quickstep_phase,
        elapsed,
    )?;
    advance_presented_attack(
        &previous,
        authoritative,
        &mut next,
        &mut presented.attack_phase,
        source_changed,
        elapsed,
    )?;
    apply_authored_cadence(
        &previous,
        &mut next,
        &mut presented.authored_cadence,
        authored_strides,
        elapsed,
    )?;
    presented.state = next;
    presented.source_tick = authoritative.locomotion_sample_tick;
    Ok(())
}

/// Keep authored quicksteps on a render-time timeline. Authoritative tactical
/// ticks can be coalesced between render observations (including phase 0.13 →
/// 1.0); copying that discontinuity directly skips the authored load and tuck.
/// Gameplay still owns action admission/contact. Presentation only bounds how
/// quickly the already-authoritative action phase may advance visually.
fn advance_presented_quickstep(
    previous: &SkeletonState,
    authoritative: &SkeletonState,
    next: &mut SkeletonState,
    phase: &mut Option<UnitInterval>,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<()> {
    let delta_seconds = elapsed.seconds();
    let continuing = previous.is_quickstep();
    if !authoritative.is_quickstep() && !continuing {
        *phase = None;
        return Ok(());
    }
    if !authoritative.is_quickstep() && phase.is_some_and(|phase| phase == UnitInterval::ONE) {
        *phase = None;
        return Ok(());
    }

    let source = if authoritative.is_quickstep() {
        authoritative
    } else {
        previous
    };
    let preparation_ticks = source
        .action_preparation_ticks()
        .unwrap_or(AnimationSampleSpan::ONE)
        .max(AnimationSampleSpan::ONE);
    let entering = phase.is_none() && !previous.is_quickstep() && authoritative.is_quickstep();
    let mut visual_phase = phase.unwrap_or_else(|| {
        if previous.is_quickstep() {
            previous.action_phase()
        } else {
            source.action_phase()
        }
    });
    let action_span = preparation_ticks.saturating_mul(2);
    if !entering {
        visual_phase = UnitInterval::from_fraction(
            (visual_phase.fraction()
                + delta_seconds.max(0.0) * locomotion_sample_rate().hertz()
                    / action_span.samples() as f32)
                .min(1.0),
        )?;
    }

    let mut candidate = source.clone();
    let start_tick = source
        .action_start_tick()
        .unwrap_or(AnimationSampleTick::ZERO);
    let elapsed_ticks = (visual_phase.fraction() * action_span.samples() as f32).round() as u64;
    candidate.advance_action(
        start_tick
            .saturating_add(AnimationSampleSpan::from(elapsed_ticks))
            .min(start_tick.saturating_add(action_span)),
    )?;
    *next = candidate;
    *phase = Some(visual_phase);
    Ok(())
}

fn presentation_phase_speed(skeleton: &SkeletonState) -> AnimationValueResult<NonnegativeSpeed> {
    let speed = skeleton.animation_speed()?;
    if skeleton.is_downed_turning() {
        speed.checked_scale(2.0).map_err(AnimationValueError::from)
    } else if matches!(skeleton.body(), BodyState::Prone | BodyState::Supine) {
        // Match the authority: physical translation while downed is presented
        // as an idle-pose slide, not a predicted crawl/scamper cycle.
        Ok(NonnegativeSpeed::ZERO)
    } else {
        Ok(speed)
    }
}

#[cfg(test)]
mod authored_cadence_tests {
    use super::*;

    #[test]
    fn ordinary_attack_advances_between_repeated_authoritative_contact_samples() {
        let mut authoritative =
            SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        authoritative
            .begin_attack(
                AttackSpec::main(StrikeFamily::Swing, false),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(12u64),
            )
            .unwrap();
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(12u64))
            .unwrap();
        assert_eq!(authoritative.action_phase().fraction(), 0.5);
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let strides = AuthoredLocomotionStrides::default();

        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        assert_eq!(presented.action_phase().fraction(), 0.5);

        // No new server tick: render presentation must cross contact instead
        // of displaying the same pose for another frame.
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        assert!(presented.action_phase().fraction() > 0.5);
        assert!(presented.action_phase().fraction() < 0.55);
        assert_eq!(authoritative.action_phase().fraction(), 0.5);
    }

    #[test]
    fn ordinary_attack_reconciles_packets_without_reversing_visual_phase() {
        let mut authoritative =
            SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        authoritative
            .begin_attack(
                AttackSpec::main(StrikeFamily::Swing, false),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(12u64),
            )
            .unwrap();
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(12u64))
            .unwrap();
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let strides = AuthoredLocomotionStrides::default();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        let predicted = presented.action_phase().fraction();

        authoritative.locomotion_sample_tick = authoritative.locomotion_sample_tick.wrapping_next();
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(13u64))
            .unwrap();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();

        assert!(presented.action_phase().fraction() > predicted);
        assert!(presented.action_phase().fraction() <= predicted + 0.05);
        assert!(presented.attack_phase.is_some_and(|phase| {
            phase.error_remaining.fraction() < 0.0 && phase.error_remaining.fraction().abs() < 0.05
        }));
    }

    #[test]
    fn delayed_attack_packet_converges_without_skipping_authored_motion() {
        let mut authoritative =
            SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        authoritative
            .begin_attack_timed(
                AttackSpec::main(StrikeFamily::Swing, true),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(17u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(28u64),
            )
            .unwrap();
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(5u64))
            .unwrap();
        authoritative = authoritative.with_locomotion_sample_tick(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(5u64),
        );
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let strides = AuthoredLocomotionStrides::default();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        let previous_phase = presented.action_phase().fraction();

        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(13u64))
            .unwrap();
        authoritative = authoritative.with_locomotion_sample_tick(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(13u64),
        );
        assert!(
            authoritative.action_phase().fraction() - previous_phase
                > playback_tuning().phase_snap_error.phase_units()
        );
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();

        let nominal_delta = 0.5 / 17.0;
        let maximum_correction = playback_tuning()
            .phase_correction_rate_per_second
            .phase_units_per_second()
            / locomotion_sample_rate().hertz();
        assert!(presented.action_phase().fraction() > previous_phase);
        assert!(
            presented.action_phase().fraction()
                <= (previous_phase + nominal_delta + maximum_correction).next_up()
        );
        assert!(presented.action_phase().fraction() < authoritative.action_phase().fraction());
    }

    #[test]
    fn completed_authoritative_attack_finishes_its_presented_tail() {
        let mut attack = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        attack
            .begin_attack_timed(
                AttackSpec::main(StrikeFamily::Swing, true),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
            )
            .unwrap();
        attack
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(17u64))
            .unwrap();
        attack = attack.with_locomotion_sample_tick(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(17u64),
        );
        assert!((attack.action_phase().fraction() - 0.85).abs() < 0.0001);

        let mut presented = PresentedSkeleton::new(attack.clone(), None);
        let strides = AuthoredLocomotionStrides::default();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &attack,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();

        let mut idle = attack.with_locomotion_sample_tick(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(21u64),
        );
        idle.advance_action(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(21u64),
        )
        .unwrap();
        assert_eq!(idle.action_kind(), SkeletonAction::None);
        for _ in 0..4 {
            advance_presented_skeleton_with_strides(
                &mut presented,
                &idle,
                locomotion_sample_rate().interval().elapsed(),
                &strides,
            )
            .unwrap();
        }
        assert_eq!(presented.action_kind(), SkeletonAction::Attack);
        assert_eq!(presented.action_phase().fraction(), 1.0);

        advance_presented_skeleton_with_strides(
            &mut presented,
            &idle,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        assert_eq!(presented.action_kind(), SkeletonAction::None);
    }

    #[test]
    fn follow_up_recovery_spends_time_on_its_authored_overshoot() {
        let mut authoritative =
            SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        let spec = AttackSpec::main(StrikeFamily::Swing, true);
        let recovery_scale = 1.0 + spec.curve.overshoot;
        authoritative
            .begin_attack_timed(
                spec,
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
            )
            .unwrap();
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64))
            .unwrap();
        assert!(authoritative.attack_is_continuation());
        assert_eq!(authoritative.action_phase().fraction(), 0.5);

        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let strides = AuthoredLocomotionStrides::default();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &authoritative,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();

        let expected_delta = 0.5 / (10.0 * recovery_scale);
        assert!((presented.action_phase().fraction() - (0.5 + expected_delta)).abs() < 0.0001);
        assert_eq!(authoritative.action_phase().fraction(), 0.5);
    }

    #[test]
    fn predicted_follow_up_survives_stale_initial_attack_packet() {
        let mut initial = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        initial
            .begin_attack_timed(
                AttackSpec::main(StrikeFamily::Swing, false),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(30u64),
            )
            .unwrap();
        let continuation_tick = initial.attack_continuation_tick().unwrap();
        initial
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64))
            .unwrap();

        let mut predicted = initial.clone().with_locomotion_sample_tick(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
        );
        predicted
            .begin_attack_timed(
                AttackSpec::main(StrikeFamily::Swing, true),
                continuation_tick,
                continuation_tick
                    .checked_add(AnimationSampleSpan::from(10u64))
                    .unwrap(),
                continuation_tick
                    .checked_add(AnimationSampleSpan::from(20u64))
                    .unwrap(),
            )
            .unwrap();
        let mut presented = PresentedSkeleton::new(initial.clone(), None);
        let strides = AuthoredLocomotionStrides::default();

        advance_presented_skeleton_with_strides(
            &mut presented,
            &predicted,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        assert!(presented.attack_has_queued_continuation());
        assert!(!presented.attack_is_continuation());

        let mut stale = initial.clone().with_locomotion_sample_tick(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(11u64),
        );
        stale
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(11u64))
            .unwrap();
        assert!(!stale.attack_is_continuation());
        advance_presented_skeleton_with_strides(
            &mut presented,
            &stale,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        assert!(presented.attack_has_queued_continuation());
        assert!(!presented.attack_is_continuation());

        stale = initial
            .clone()
            .with_locomotion_sample_tick(continuation_tick);
        stale.advance_action(continuation_tick).unwrap();
        advance_presented_skeleton_with_strides(
            &mut presented,
            &stale,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        assert!(presented.attack_is_continuation());
        assert_eq!(presented.action_start_tick(), Some(continuation_tick));

        let stale_follow_phase = presented.action_phase().fraction();
        predicted
            .advance_action(
                continuation_tick
                    .checked_add(AnimationSampleSpan::from(1u64))
                    .unwrap(),
            )
            .unwrap();
        predicted = predicted.with_locomotion_sample_tick(
            continuation_tick
                .checked_add(AnimationSampleSpan::from(1u64))
                .unwrap(),
        );
        advance_presented_skeleton_with_strides(
            &mut presented,
            &predicted,
            locomotion_sample_rate().interval().elapsed(),
            &strides,
        )
        .unwrap();
        assert!(presented.attack_is_continuation());
        assert!(presented.action_phase().fraction() >= stale_follow_phase);
    }

    #[test]
    fn coalesced_quickstep_phase_still_traverses_the_authored_timeline() {
        let mut authoritative =
            SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        authoritative
            .begin_dodge(
                DodgeSpec::quickstep(Vec2::X).unwrap(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
            )
            .unwrap();
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);

        // Simulate a coalesced authoritative observation that jumps directly
        // to contact/recovery complete. Presentation must still visit frame 6,
        // the midpoint of the authoritative 0/3/6/9/12 source timeline.
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(30u64))
            .unwrap();
        let strides = AuthoredLocomotionStrides::default();
        for _ in 0..15 {
            advance_presented_skeleton_with_strides(
                &mut presented,
                &authoritative,
                locomotion_sample_rate().interval().elapsed(),
                &strides,
            )
            .unwrap();
        }
        let evaluation = AnimationEvaluation::from_skeleton(&presented.state).unwrap();
        assert!((presented.action_phase().fraction() - 0.75).abs() < 0.051);
        assert!(evaluation.lower_body.iter().any(|sample| matches!(
            sample.sampling,
            PoseSampling::Timeline { progress } if (progress.fraction() - 0.75).abs() < 0.051
        )));
    }

    #[test]
    fn released_guard_cannot_replace_the_presented_quickstep_timeline() {
        let mut authoritative =
            SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        authoritative
            .begin_dodge(
                DodgeSpec::quickstep(Vec2::X).unwrap(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
            )
            .unwrap();
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64))
            .unwrap();
        let mut presented = PresentedSkeleton::new(authoritative.clone(), None);
        let strides = AuthoredLocomotionStrides::default();

        // The server has already observed landing and guard release. The
        // presentation timeline still owns the complete authored action
        // output before it may route back to ordinary/combat locomotion.
        authoritative
            .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(31u64))
            .unwrap();
        authoritative = authoritative.with_weapon_guard(WeaponGuardState::Lowered);
        authoritative = authoritative.with_local_velocity(Vec3::X * 2.0);

        let initial = AnimationEvaluation::from_skeleton(&presented.state).unwrap();
        let mut progresses = initial
            .lower_body
            .iter()
            .filter_map(|sample| match sample.sampling {
                PoseSampling::Timeline { progress } => Some(progress.fraction()),
                _ => None,
            })
            .collect::<Vec<_>>();
        while presented.is_quickstep() {
            advance_presented_skeleton_with_strides(
                &mut presented,
                &authoritative,
                locomotion_sample_rate().interval().elapsed(),
                &strides,
            )
            .unwrap();
            let evaluation = AnimationEvaluation::from_skeleton(&presented.state).unwrap();
            if presented.is_quickstep() {
                assert!(!evaluation.lower_body.is_empty());
                for sample in &evaluation.lower_body {
                    assert_eq!(sample.pose, SemanticPose::QuickstepRightTakeoff);
                    let PoseSampling::Timeline { progress } = sample.sampling else {
                        panic!("quickstep lower body must remain timeline sampled")
                    };
                    progresses.push(progress.fraction());
                }
            }
            assert!(progresses.len() < 64, "presented quickstep did not finish");
        }

        assert!(progresses.first().is_some_and(|progress| *progress <= 0.01));
        assert!(
            progresses
                .iter()
                .any(|progress| (*progress - 0.5).abs() <= 0.11)
        );
        assert!(progresses.last().is_some_and(|progress| *progress >= 0.99));
        let released = AnimationEvaluation::from_skeleton(&presented.state).unwrap();
        assert!(released.lower_body.iter().all(|sample| !matches!(
            sample.pose,
            SemanticPose::QuickstepRightTakeoff
                | SemanticPose::StrafeCycle
                | SemanticPose::SkipCycle
        )));
    }
}

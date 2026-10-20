use super::*;

impl PresentedAttackPhase {
    /// The whole native bounded prediction keeps division before the final cap.
    /// An overflowing transient delta can still complete a finite progress.
    fn predicted(
        self,
        source: &SkeletonState,
        elapsed: AnimationElapsed,
        duration: AnimationSampleSpan,
    ) -> AnimationValueResult<UnitInterval> {
        let scale = if source.attack_is_continuation() && self.phase.fraction() >= 0.5 {
            1.0 + source.attack_curve().overshoot
        } else {
            1.0
        };
        let delta = elapsed.seconds() * locomotion_sample_rate().hertz() * 0.5
            / duration.max(AnimationSampleSpan::ONE).samples() as f32;
        Ok(UnitInterval::from_fraction(
            (self.phase.fraction() + delta / scale).min(1.0),
        )?)
    }

    fn advanced(
        self,
        source: &SkeletonState,
        measure_source: bool,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<Self> {
        let duration = if self.phase.fraction() < 0.5 {
            source.action_preparation_ticks()
        } else {
            source.action_recovery_ticks()
        }
        .unwrap_or(AnimationSampleSpan::ONE)
        .max(AnimationSampleSpan::ONE);
        let predicted = self.predicted(source, elapsed, duration)?;
        let mut error = self.error_remaining.fraction();
        let presentation_owned_recovery =
            source.attack_is_continuation() && self.phase.fraction() >= 0.5;
        if measure_source && !presentation_owned_recovery {
            let measured_error = source.action_phase().fraction() - predicted.fraction();
            let drift = if measured_error.abs() <= presentation_phase_drift_deadband() {
                0.0
            } else {
                measured_error - measured_error.signum() * presentation_phase_drift_deadband()
            };
            error += (drift - error) * playback_tuning().phase_drift_measurement_blend.fraction();
        } else if presentation_owned_recovery {
            error = 0.0;
        }
        let maximum = presentation_phase_correction_rate_per_second() * elapsed.seconds();
        let correction = error.clamp(-maximum, maximum);
        let phase = UnitInterval::from_fraction(
            (predicted.fraction() + correction).clamp(self.phase.fraction(), 1.0),
        )?;
        error -= phase.fraction() - predicted.fraction();
        Ok(Self {
            phase,
            error_remaining: ActionProgressDelta::from_arithmetic(error)?,
            ..self
        })
    }
}

/// Advance an admitted ordinary attack between replicated observations. All
/// checked candidate progress is ready before its presentation fields commit.
pub(super) fn advance_presented_attack(
    previous: &SkeletonState,
    authoritative: &SkeletonState,
    next: &mut SkeletonState,
    presented: &mut Option<PresentedAttackPhase>,
    source_changed: bool,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<()> {
    let mut candidate = next.clone();
    let mut candidate_phase = *presented;
    let stale = reconcile_attack_source(previous, authoritative, &mut candidate)?;
    let source = candidate.clone();
    if source.action_kind() != SkeletonAction::Attack {
        if !complete_attack_tail(previous, &mut candidate, &mut candidate_phase, elapsed)? {
            candidate_phase = None;
        }
    } else if let Some(start_tick) = source.action_start_tick() {
        if !candidate_phase.is_some_and(|phase| phase.matches(&source)) {
            candidate_phase = Some(PresentedAttackPhase {
                start_tick,
                animation: source.attack_animation().unwrap_or(AttackAnimation::Thrust),
                hand: source.attack_hand(),
                continuation: source.attack_is_continuation(),
                phase: source.action_phase(),
                error_remaining: ActionProgressDelta::ZERO,
            });
        } else if let Some(phase) = candidate_phase {
            let phase = phase.advanced(&source, source_changed && !stale, elapsed)?;
            candidate = source.clone();
            candidate.set_presentation_action_phase(phase.phase);
            debug_assert!(
                previous.action_kind() != SkeletonAction::Attack
                    || phase.phase >= previous.action_phase()
            );
            candidate_phase = Some(phase);
        }
    } else {
        candidate_phase = None;
    }
    *next = candidate;
    *presented = candidate_phase;
    Ok(())
}

fn reconcile_attack_source(
    previous: &SkeletonState,
    authoritative: &SkeletonState,
    next: &mut SkeletonState,
) -> AnimationValueResult<bool> {
    let same_initial_attack = authoritative.action_kind() == SkeletonAction::Attack
        && !authoritative.attack_is_continuation()
        && authoritative.attack_animation() == previous.attack_animation()
        && authoritative.attack_hand() == previous.attack_hand();
    let stale_missing_queue = previous.attack_has_queued_continuation()
        && same_initial_attack
        && !authoritative.attack_has_queued_continuation()
        && previous.action_start_tick() == authoritative.action_start_tick()
        && previous
            .attack_continuation_tick()
            .is_some_and(|transition_tick| {
                authoritative.locomotion_sample_tick
                    <= transition_tick.saturating_add(playback_tuning().maximum_source_gap_ticks)
            });
    let stale_initial_attack = previous.attack_is_continuation()
        && same_initial_attack
        && authoritative.attack_continuation_tick() == previous.action_start_tick();
    let stale_attack_payload = stale_missing_queue || stale_initial_attack;
    if stale_attack_payload {
        *next = previous.clone();
        next.advance_action(authoritative.locomotion_sample_tick)?;
    } else if authoritative.attack_has_queued_continuation() {
        next.advance_action(authoritative.locomotion_sample_tick)?;
    }
    Ok(stale_attack_payload)
}

fn complete_attack_tail(
    previous: &SkeletonState,
    next: &mut SkeletonState,
    presented: &mut Option<PresentedAttackPhase>,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<bool> {
    let Some(mut phase) = *presented else {
        return Ok(false);
    };
    if previous.action_kind() != SkeletonAction::Attack || phase.phase == UnitInterval::ONE {
        return Ok(false);
    }
    let duration = previous
        .action_recovery_ticks()
        .unwrap_or(AnimationSampleSpan::ONE)
        .max(AnimationSampleSpan::ONE);
    phase.phase = phase.predicted(previous, elapsed, duration)?;
    phase.error_remaining = ActionProgressDelta::ZERO;
    *next = previous.clone();
    next.set_presentation_action_phase(phase.phase);
    *presented = Some(phase);
    Ok(true)
}

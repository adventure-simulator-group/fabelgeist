//! Authored-stride cadence selection and retained presentation phase.

use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct AuthoredCadence {
    kind: AuthoredCadenceKind,
    phase: CyclePhase,
    limit: AuthoredCadenceLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthoredCadenceKind {
    Ordinary,
    Combat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthoredCadenceLimit {
    Unrestricted,
    StanceSlip,
    CadenceCapped,
}

pub(super) fn apply_authored_cadence(
    previous: &SkeletonState,
    next: &mut SkeletonState,
    cadence: &mut Option<AuthoredCadence>,
    strides: &AuthoredLocomotionStrides,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<()> {
    if next.posture() != Posture::Upright || next.is_quickstep() {
        *cadence = None;
        return Ok(());
    }
    let kind = if next.weapon_guard() == WeaponGuardState::Raised && !next.is_guarded_sprinting() {
        AuthoredCadenceKind::Combat
    } else {
        AuthoredCadenceKind::Ordinary
    };
    let speed = next.animation_speed()?;
    let measurement = match kind {
        AuthoredCadenceKind::Ordinary => strides.ordinary(speed)?,
        AuthoredCadenceKind::Combat => {
            strides.combat(next.raised_locomotion().local_direction())?
        }
    };
    let continuing = cadence.filter(|current| current.kind == kind);
    if measurement.is_none() && !(speed <= STATIONARY_ANIMATION_SPEED_LIMIT && continuing.is_some())
    {
        *cadence = None;
        return Ok(());
    }
    let mut phase = continuing.map_or(previous.gait_phase, |current| current.phase);
    let mut limit = continuing.map_or(AuthoredCadenceLimit::Unrestricted, |current| current.limit);
    if speed > STATIONARY_ANIMATION_SPEED_LIMIT
        && let Some(measurement) = measurement
    {
        let selection = playback_tuning()
            .authored_step_cadence
            .select_for_motion(speed, measurement.step_distance)?;
        limit = if selection.is_capped() {
            AuthoredCadenceLimit::CadenceCapped
        } else if measurement.maximum_stance_slip > maximum_authored_stance_slip_metres() {
            AuthoredCadenceLimit::StanceSlip
        } else {
            AuthoredCadenceLimit::Unrestricted
        };
        if limit == AuthoredCadenceLimit::CadenceCapped
            && !continuing.is_some_and(|current| current.limit == AuthoredCadenceLimit::CadenceCapped)
        {
            warn!(
                speed_metres_per_second = speed.metres_per_second(),
                requested_steps_per_second = ?selection.requested(),
                applied_steps_per_second = selection.applied().per_second(),
                maximum_stance_slip_metres = measurement.maximum_stance_slip.metres(),
                "Authored locomotion clip cannot represent requested speed"
            );
        }
        let delta = selection.applied().cycle_delta(elapsed)?;
        phase = phase.checked_advance(delta)?.wrapped();
    }
    next.gait_phase = phase;
    *cadence = Some(AuthoredCadence { kind, phase, limit });
    Ok(())
}

#[cfg(test)]
mod tests;

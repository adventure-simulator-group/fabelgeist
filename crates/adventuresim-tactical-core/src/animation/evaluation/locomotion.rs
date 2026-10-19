//! Checked locomotion contributions from body motion and authored contact cycles.

use super::*;

pub(super) fn combat_lower_body_samples(
    state: &SkeletonState,
) -> AnimationValueResult<Vec<PoseSample>> {
    let speed = state.animation_speed();
    if speed <= 0.05 || !state.raised_locomotion().is_moving() {
        return Ok(vec![anchor_sample(SemanticPose::CombatStance)]);
    }
    if state.guarded_sprint_locomotion() {
        return locomotion_samples(speed, state.gait_phase);
    }
    let direction = state.raised_locomotion().local_direction();
    let strafe = direction.x.abs();
    let skip = direction.y.abs();
    let total = strafe + skip;
    if total <= 0.0 {
        return Ok(vec![anchor_sample(SemanticPose::CombatStance)]);
    }
    let mut samples = Vec::with_capacity(2);
    if strafe > 0.0 {
        let phase = combat_cycle_phase(state.gait_phase)?;
        let mut sample = cycle_sample(
            SemanticPose::StrafeCycle,
            if direction.x < 0.0 {
                reverse_cycle_phase(phase)?
            } else {
                phase
            },
        );
        sample.weight = BlendWeight::from_factor(strafe / total)?;
        samples.push(sample);
    }
    if skip > 0.0 {
        let phase = combat_cycle_phase(state.gait_phase)?;
        // Forward skip selects the opposite authored contact half-cycle.
        let phase = if direction.y < 0.0 {
            CyclePhase::from_cycles((phase.cycles() + 0.5).rem_euclid(1.0))?
        } else {
            phase
        };
        let mut sample = cycle_sample(SemanticPose::SkipCycle, phase);
        sample.weight = BlendWeight::from_factor(skip / total)?;
        samples.push(sample);
    }
    Ok(samples)
}

pub(super) fn quickstep_lower_body_samples(
    state: &SkeletonState,
) -> AnimationValueResult<Vec<PoseSample>> {
    // The full action phase keeps authored load/launch/landing on the physical force curve.
    let progress = state.action_phase().clamp(0.0, 1.0);
    let direction = state.action_direction().normalize_or_zero();
    let directional = [
        (direction.y.max(0.0), SemanticPose::QuickstepForwardTakeoff),
        (direction.x.max(0.0), SemanticPose::QuickstepRightTakeoff),
        ((-direction.x).max(0.0), SemanticPose::QuickstepLeftTakeoff),
        ((-direction.y).max(0.0), SemanticPose::QuickstepBackTakeoff),
    ];
    let total = directional.iter().map(|(weight, _)| weight).sum::<f32>();
    directional
        .into_iter()
        .filter(|(weight, _)| *weight > 0.0)
        .map(|(weight, pose)| {
            Ok(PoseSample {
                pose,
                sampling: PoseSampling::Timeline { progress },
                weight: BlendWeight::from_factor(weight / total)?,
                mirror_lower_body: ClipMirror::Original,
            })
        })
        .collect()
}

pub(super) fn gait_or_idle(
    speed: f32,
    phase: CyclePhase,
    idle: SemanticPose,
    contact: SemanticPose,
) -> AnimationValueResult<Vec<PoseSample>> {
    if speed < 0.05 {
        Ok(vec![PoseSample {
            pose: idle,
            sampling: PoseSampling::Anchor,
            weight: BlendWeight::ONE,
            mirror_lower_body: ClipMirror::Original,
        }])
    } else {
        alternating_contact_pair(phase, contact)
    }
}

/// Ordinary upright locomotion blends the forward gait against the authored
/// strafe cycle in the hips' frame. As the root turns toward world velocity,
/// the lateral contribution naturally falls to zero.
pub(super) fn upright_locomotion_samples(
    state: &SkeletonState,
) -> AnimationValueResult<Vec<PoseSample>> {
    let speed = state.animation_speed();
    let phase = state.gait_phase.wrapped();
    let direction = state.animation_local_velocity().xz().normalize_or_zero();
    let lateral = direction.x.abs();
    let longitudinal = direction.y.abs();
    let total = lateral + longitudinal;
    if lateral <= 0.0 || total <= 0.0 {
        return locomotion_samples(speed, phase);
    }

    let locomotion = smoothstep01(
        speed
            / crate::combat_config::runtime_animation_config()
                .locomotion
                .blend_speed,
    );
    let longitudinal_weight = longitudinal / total;
    let lateral_weight = lateral / total;
    let mut samples = locomotion_samples(speed, phase)?;
    for sample in &mut samples {
        if sample.pose == SemanticPose::IdleRelaxed {
            continue;
        }
        sample.weight = sample
            .weight
            .checked_scale(BlendWeight::from_factor(longitudinal_weight)?)?;
    }
    let strafe_phase = combat_cycle_phase(phase)?;
    let mut strafe = cycle_sample(
        SemanticPose::StrafeCycle,
        if direction.x < 0.0 {
            reverse_cycle_phase(strafe_phase)?
        } else {
            strafe_phase
        },
    );
    strafe.weight = BlendWeight::from_factor(locomotion * lateral_weight)?;
    samples.push(strafe);
    samples.retain(|sample| !sample.weight.is_absent());
    Ok(samples)
}

pub(super) fn downed_animation_speed(state: &SkeletonState) -> f32 {
    if state.downed_turning() {
        state.animation_speed()
    } else {
        0.0
    }
}

fn combat_cycle_phase(gait_phase: CyclePhase) -> AnimationValueResult<CyclePhase> {
    // Authored combat contacts are frames 6/18 of a 24-frame cycle.
    CyclePhase::from_cycles((gait_phase.cycles() + 0.25).rem_euclid(1.0))
}

fn reverse_cycle_phase(phase: CyclePhase) -> AnimationValueResult<CyclePhase> {
    CyclePhase::from_cycles((1.0 - phase.cycles()).rem_euclid(1.0))
}

fn anchor_sample(pose: SemanticPose) -> PoseSample {
    PoseSample {
        pose,
        sampling: PoseSampling::Anchor,
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }
}

fn alternating_contact_pair(
    phase: CyclePhase,
    contact: SemanticPose,
) -> AnimationValueResult<Vec<PoseSample>> {
    let half = phase.wrapped().cycles() * 2.0;
    let progress = smoothstep01(half.fract());
    let (start_mirrored, end_mirrored) = if half < 1.0 {
        (ClipMirror::Original, ClipMirror::Mirrored)
    } else {
        (ClipMirror::Mirrored, ClipMirror::Original)
    };
    let mut samples = Vec::with_capacity(2);
    if progress < 1.0 {
        samples.push(PoseSample {
            pose: contact,
            sampling: PoseSampling::Anchor,
            weight: BlendWeight::from_factor(1.0 - progress)?,
            mirror_lower_body: start_mirrored,
        });
    }
    if progress > 0.0 {
        samples.push(PoseSample {
            pose: contact,
            sampling: PoseSampling::Anchor,
            weight: BlendWeight::from_factor(progress)?,
            mirror_lower_body: end_mirrored,
        });
    }
    Ok(samples)
}

fn locomotion_samples(speed: f32, phase: CyclePhase) -> AnimationValueResult<Vec<PoseSample>> {
    let locomotion = smoothstep01(
        speed
            / crate::combat_config::runtime_animation_config()
                .locomotion
                .blend_speed,
    );
    let run = ((speed - walk_locomotion_profile().reference_speed)
        / (run_locomotion_profile().reference_speed - walk_locomotion_profile().reference_speed))
        .clamp(0.0, 1.0);
    let mut samples = Vec::with_capacity(5);
    append_scaled(
        &mut samples,
        vec![PoseSample {
            pose: SemanticPose::IdleRelaxed,
            sampling: PoseSampling::Anchor,
            weight: BlendWeight::ONE,
            mirror_lower_body: ClipMirror::Original,
        }],
        BlendWeight::from_factor(1.0 - locomotion)?,
    )?;
    append_scaled(
        &mut samples,
        vec![cycle_sample(SemanticPose::WalkContact, phase)],
        BlendWeight::from_factor(locomotion * (1.0 - run))?,
    )?;
    append_scaled(
        &mut samples,
        vec![cycle_sample(SemanticPose::RunContact, phase)],
        BlendWeight::from_factor(locomotion * run)?,
    )?;
    samples.retain(|sample| !sample.weight.is_absent());
    Ok(samples)
}

fn cycle_sample(pose: SemanticPose, phase: CyclePhase) -> PoseSample {
    PoseSample {
        pose,
        sampling: PoseSampling::Cycle {
            phase: phase.wrapped(),
        },
        weight: BlendWeight::ONE,
        mirror_lower_body: ClipMirror::Original,
    }
}

fn append_scaled(
    into: &mut Vec<PoseSample>,
    samples: Vec<PoseSample>,
    scale: BlendWeight,
) -> AnimationValueResult<()> {
    let scaled = samples
        .into_iter()
        .map(|mut sample| {
            sample.weight = sample.weight.checked_scale(scale)?;
            Ok(sample)
        })
        .collect::<AnimationValueResult<Vec<_>>>()?;
    into.extend(scaled);
    Ok(())
}

#[cfg(test)]
pub(in crate::animation) fn gait_pair(
    phase: f32,
    contact: SemanticPose,
    passing: SemanticPose,
) -> AnimationValueResult<Vec<PoseSample>> {
    let quarter = phase.rem_euclid(1.0) * 4.0;
    // A smooth cubic Hermite coordinate gives every sparse contact/flight
    // anchor zero endpoint velocity. Linear quarter interpolation changed
    // velocity discontinuously at each anchor and produced a visible knee and
    // ankle snap even though the poses themselves were continuous.
    let progress = smoothstep01(quarter.fract());
    let (start, start_mirrored, end, end_mirrored) = match quarter.floor() as u8 {
        0 => (contact, false, passing, false),
        1 => (passing, false, contact, true),
        2 => (contact, true, passing, true),
        _ => (passing, true, contact, false),
    };
    let mut samples = Vec::with_capacity(2);
    if progress < 1.0 {
        samples.push(PoseSample {
            pose: start,
            sampling: PoseSampling::Anchor,
            weight: BlendWeight::from_factor(1.0 - progress)?,
            mirror_lower_body: ClipMirror::from(start_mirrored),
        });
    }
    if progress > 0.0 {
        samples.push(PoseSample {
            pose: end,
            sampling: PoseSampling::Anchor,
            weight: BlendWeight::from_factor(progress)?,
            mirror_lower_body: ClipMirror::from(end_mirrored),
        });
    }
    Ok(samples)
}

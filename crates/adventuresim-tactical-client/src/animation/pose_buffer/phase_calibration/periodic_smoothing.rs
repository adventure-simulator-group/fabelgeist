//! Atomic periodic convolution of a checked, unwrapped authored-phase curve.

use super::{AnimationValueResult, ContactOrientation, CycleDelta, CyclePhase, TimelineDirection};

/// A periodic smoothing stencil requires three distinct samples.
const MINIMUM_PERIODIC_SAMPLES: usize = 3;
const RADIUS: isize = 6;

pub(super) fn smooth_periodic_phase_curve(
    values: &mut [CyclePhase],
    orientation: ContactOrientation,
) -> AnimationValueResult<()> {
    let sample_count = values.len().saturating_sub(1);
    if sample_count < MINIMUM_PERIODIC_SAMPLES {
        return Ok(());
    }
    let deviations = values
        .iter()
        .take(sample_count)
        .enumerate()
        .map(|(index, value)| {
            // Grid ordinals become physical cycle coordinates only in this stencil.
            let phase = CyclePhase::from_cycles(index as f32 / sample_count as f32)?;
            value.checked_difference(orientation.smoothing_baseline(phase)?)
        })
        .collect::<AnimationValueResult<Vec<_>>>()?;
    let mut replacement = Vec::with_capacity(values.len());
    for index in 0..sample_count {
        let mut weighted_sum = CycleDelta::ZERO;
        let mut total_weight = 0.0;
        for offset in -RADIUS..=RADIUS {
            let wrapped = (index as isize + offset).rem_euclid(sample_count as isize) as usize;
            let weight = (RADIUS + 1 - offset.abs()) as f32;
            // Native triangular convolution: multiply each displacement before
            // adding it, retaining the original floating-point accumulation order.
            let weighted = CycleDelta::from_arithmetic(deviations[wrapped].cycles() * weight)?;
            weighted_sum = weighted_sum.checked_add(weighted)?;
            total_weight += weight;
        }
        let phase = CyclePhase::from_cycles(index as f32 / sample_count as f32)?;
        let base = orientation.smoothing_baseline(phase)?;
        let deviation = CycleDelta::from_arithmetic(weighted_sum.cycles() / total_weight)?;
        replacement.push(base.checked_advance(deviation)?);
    }
    let seam = CycleDelta::from_cycles(if orientation.direction == TimelineDirection::Reversed {
        -1.0
    } else {
        1.0
    })?;
    replacement.push(replacement[0].checked_advance(seam)?);
    // Neither the supplied samples nor their periodic seam change until every
    // product, sum and replacement has passed finite arithmetic admission.
    values.copy_from_slice(&replacement);
    Ok(())
}

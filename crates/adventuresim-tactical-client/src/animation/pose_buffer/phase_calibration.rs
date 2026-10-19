//! Fit the periodic physical-cycle to authored-cycle calibration curve.
//!
//! Native two-column cycle coordinates stay inside this fitting kernel. The
//! published curve contains only checked finite, deliberately unwrapped phases.

use super::*;

/// Authored-cycle endpoint allowance for the fitted contact span, in cycles.
const CONTACT_SPAN_ENDPOINT_ALLOWANCE_CYCLES: f32 = 0.000_001;
/// A periodic smoothing stencil requires three distinct samples.
const MINIMUM_PERIODIC_SAMPLES: usize = 3;

pub(super) struct ContactOrientation {
    pub(super) direction: TimelineDirection,
    pub(super) offset: CyclePhase,
}

impl ContactOrientation {
    pub(super) fn fit(
        segments: &[Vec<(f32, f32)>],
        expected_stance_direction: f32,
    ) -> Option<Self> {
        let signed_stance_displacement = segments
            .iter()
            .map(|samples| {
                Some((samples.last()?.1 - samples.first()?.1) * expected_stance_direction.signum())
            })
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .sum::<f32>();
        if signed_stance_displacement.abs()
            <= pose_tuning().minimum_phase_fit_displacement_metres.metres()
        {
            return None;
        }
        let direction = if signed_stance_displacement < 0.0 {
            TimelineDirection::Reversed
        } else {
            TimelineDirection::Forward
        };
        let semantic_contact_centers = [0.0_f32, 0.5_f32];
        let mut offsets =
            segments
                .iter()
                .zip(semantic_contact_centers)
                .map(|(samples, semantic_center)| {
                    let authored_center = (samples.first()?.0 + samples.last()?.0) * 0.5;
                    if direction == TimelineDirection::Reversed {
                        Some(authored_center + semantic_center)
                    } else {
                        Some(authored_center - semantic_center)
                    }
                });
        let first_offset = offsets.next()??.rem_euclid(1.0);
        let (offset_sum, offset_count) =
            offsets.try_fold((first_offset, 1_u32), |(sum, count), offset| {
                let offset = offset?;
                let aligned = first_offset + (offset - first_offset + 0.5).rem_euclid(1.0) - 0.5;
                Some((sum + aligned, count + 1))
            })?;
        let phase_offset = (offset_sum / offset_count as f32).rem_euclid(1.0);
        Some(Self {
            direction,
            offset: CyclePhase::from_cycles(phase_offset).ok()?,
        })
    }
}

impl AuthoredPhaseCurve {
    pub(super) fn fit_contact_spans(
        phase_spans: &[Vec<(f32, f32)>],
        direction: TimelineDirection,
        phase_offset: f32,
    ) -> AnimationValueResult<Self> {
        let intervals = pose_tuning().authored_phase_calibration_intervals;
        let mut authored_phases = (0..=intervals)
            .map(|index| {
                let physical_phase = index as f32 / intervals as f32;
                sample_authored_phase_spans(phase_spans, physical_phase, direction, phase_offset)
            })
            .collect::<Vec<_>>();
        smooth_periodic_phase_curve(&mut authored_phases, direction, phase_offset);
        let authored_phases = authored_phases
            .into_iter()
            .map(CyclePhase::from_cycles)
            .collect::<AnimationValueResult<Vec<_>>>()?;
        Ok(Self { authored_phases })
    }
}

fn sample_authored_phase_spans(
    spans: &[Vec<(f32, f32)>],
    physical_phase: f32,
    direction: TimelineDirection,
    phase_offset: f32,
) -> f32 {
    for span in spans {
        for cycle_offset in [-1.0, 0.0, 1.0] {
            let authored_offset = if direction == TimelineDirection::Reversed {
                -cycle_offset
            } else {
                cycle_offset
            };
            let start = span[0].0 + cycle_offset;
            let end = span[span.len() - 1].0 + cycle_offset;
            if physical_phase + CONTACT_SPAN_ENDPOINT_ALLOWANCE_CYCLES < start
                || physical_phase - CONTACT_SPAN_ENDPOINT_ALLOWANCE_CYCLES > end
            {
                continue;
            }
            let upper = span.partition_point(|(phase, _)| *phase + cycle_offset < physical_phase);
            if upper == 0 {
                return span[0].1 + authored_offset;
            }
            if upper >= span.len() {
                return span[span.len() - 1].1 + authored_offset;
            }
            let (lower_phase, lower_authored) = span[upper - 1];
            let (upper_phase, upper_authored) = span[upper];
            let amount = ((physical_phase - (lower_phase + cycle_offset))
                / (upper_phase - lower_phase))
                .clamp(0.0, 1.0);
            return (lower_authored + authored_offset)
                .lerp(upper_authored + authored_offset, amount);
        }
    }
    if direction == TimelineDirection::Reversed {
        phase_offset - physical_phase
    } else {
        physical_phase + phase_offset
    }
}

fn smooth_periodic_phase_curve(
    values: &mut [f32],
    direction: TimelineDirection,
    phase_offset: f32,
) {
    let sample_count = values.len().saturating_sub(1);
    if sample_count < MINIMUM_PERIODIC_SAMPLES {
        return;
    }
    let deviations = (0..sample_count)
        .map(|index| {
            let phase = index as f32 / sample_count as f32;
            let base = if direction == TimelineDirection::Reversed {
                phase_offset - phase
            } else {
                phase_offset + phase
            };
            values[index] - base
        })
        .collect::<Vec<_>>();
    const RADIUS: isize = 6;
    for (index, value) in values.iter_mut().take(sample_count).enumerate() {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        for offset in -RADIUS..=RADIUS {
            let wrapped = (index as isize + offset).rem_euclid(sample_count as isize) as usize;
            let weight = (RADIUS + 1 - offset.abs()) as f32;
            weighted_sum += deviations[wrapped] * weight;
            total_weight += weight;
        }
        let phase = index as f32 / sample_count as f32;
        let base = if direction == TimelineDirection::Reversed {
            phase_offset - phase
        } else {
            phase_offset + phase
        };
        *value = base + weighted_sum / total_weight;
    }
    values[sample_count] = if direction == TimelineDirection::Reversed {
        values[0] - 1.0
    } else {
        values[0] + 1.0
    };
}

//! Fit the periodic physical-cycle to authored-cycle calibration curve.
//!
//! Native two-column cycle coordinates stay inside this fitting kernel. The
//! published curve contains only checked finite, deliberately unwrapped phases.

use super::*;

impl AuthoredPhaseCurve {
    pub(super) fn fit_contact_spans(
        phase_spans: &[Vec<(f32, f32)>],
        reverse_playback: bool,
        phase_offset: f32,
    ) -> AnimationValueResult<Self> {
        let mut authored_phases = (0..=256)
            .map(|index| {
                let physical_phase = index as f32 / 256.0;
                sample_authored_phase_spans(
                    phase_spans,
                    physical_phase,
                    reverse_playback,
                    phase_offset,
                )
            })
            .collect::<Vec<_>>();
        smooth_periodic_phase_curve(&mut authored_phases, reverse_playback, phase_offset);
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
    reverse_playback: bool,
    phase_offset: f32,
) -> f32 {
    for span in spans {
        for cycle_offset in [-1.0, 0.0, 1.0] {
            let authored_offset = if reverse_playback {
                -cycle_offset
            } else {
                cycle_offset
            };
            let start = span[0].0 + cycle_offset;
            let end = span[span.len() - 1].0 + cycle_offset;
            if physical_phase + 0.000_001 < start || physical_phase - 0.000_001 > end {
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
    if reverse_playback {
        phase_offset - physical_phase
    } else {
        physical_phase + phase_offset
    }
}

fn smooth_periodic_phase_curve(values: &mut [f32], reverse: bool, phase_offset: f32) {
    let sample_count = values.len().saturating_sub(1);
    if sample_count < 3 {
        return;
    }
    let deviations = (0..sample_count)
        .map(|index| {
            let phase = index as f32 / sample_count as f32;
            let base = if reverse {
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
        let base = if reverse {
            phase_offset - phase
        } else {
            phase_offset + phase
        };
        *value = base + weighted_sum / total_weight;
    }
    values[sample_count] = if reverse {
        values[0] - 1.0
    } else {
        values[0] + 1.0
    };
}

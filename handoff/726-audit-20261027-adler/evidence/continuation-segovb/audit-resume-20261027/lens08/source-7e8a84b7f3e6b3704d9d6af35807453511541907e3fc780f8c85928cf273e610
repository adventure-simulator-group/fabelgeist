//! Peak prominence and phase-window checks for sampled visual height.

use super::*;

// Ignore sub-3 mm sampling noise, but reject any additional visible beat in
// the phase-owned vertical curve.
const HEIGHT_PEAK_PROMINENCE_METRES: f32 = 0.003;
const PASSING_PEAK_PHASE_WINDOW: f32 = 0.10;

pub(super) fn prominent_height_peaks(samples: &[HeightSample]) -> ReportResult<Observation<HeightPeaks>> {
    if samples.len() < 3 {
        return Ok(Observation::Unavailable(UnavailableReason::InsufficientHeightSamples));
    }
    let mut peaks = Vec::new();
    for (index, sample) in samples.iter().enumerate() {
        let phase = sample.phase().cycles();
        let height = sample.height().metres();
        let previous = samples[(index + samples.len() - 1) % samples.len()].height().metres();
        let next = samples[(index + 1) % samples.len()].height().metres();
        if height <= previous || height < next {
            continue;
        }
        let Some(left_minimum) = samples
            .iter()
            .filter_map(|candidate| {
                let candidate_phase = candidate.phase().cycles();
                let candidate_height = candidate.height().metres();
                let distance = (phase - candidate_phase).rem_euclid(1.0);
                (distance > 0.0 && distance <= 0.125).then_some(candidate_height)
            })
            .reduce(f32::min) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingHeightNeighborhood));
            };
        let Some(right_minimum) = samples
            .iter()
            .filter_map(|candidate| {
                let candidate_phase = candidate.phase().cycles();
                let candidate_height = candidate.height().metres();
                let distance = (candidate_phase - phase).rem_euclid(1.0);
                (distance > 0.0 && distance <= 0.125).then_some(candidate_height)
            })
            .reduce(f32::min) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingHeightNeighborhood));
            };
        let prominence = fabelgeist_geometry::SignedLength::from_metres(height - left_minimum.max(right_minimum))?;
        if prominence.metres() >= HEIGHT_PEAK_PROMINENCE_METRES {
            peaks.push(phase);
        }
    }
    let phase_distance = |phase: f32, target: f32| {
        let distance = (phase - target).abs();
        distance.min(1.0 - distance)
    };
    let in_passing_windows = [0.25, 0.75].into_iter().all(|target| {
        peaks
            .iter()
            .filter(|&&phase| phase_distance(phase, target) <= PASSING_PEAK_PHASE_WINDOW)
            .count()
            == 1
    });
    Ok(Observation::Measured(HeightPeaks {
        count: peaks.len(),
        in_passing_windows,
    }))
}

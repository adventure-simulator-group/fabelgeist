//! Checked uniform periodic phase mapping; absence belongs to the calibration cache.

use super::*;

/// Periodic authored sample phase indexed by a uniformly advancing physical
/// gait phase. Values are deliberately left unwrapped so interpolation remains
/// continuous across the cycle seam; callers wrap only the final result.
#[derive(Debug, Clone)]
pub(in crate::animation) struct AuthoredPhaseCurve {
    mapping: PhaseCurveMapping,
}

#[derive(Debug, Clone)]
enum PhaseCurveMapping {
    Identity,
    Sampled(Vec<CyclePhase>),
}

impl AuthoredPhaseCurve {
    /// Empty and singleton input express the existing identity mapping, not an
    /// unavailable calibration. A sampled grid closes by one forward/reverse cycle.
    pub(in crate::animation) fn from_samples(
        values: Vec<CyclePhase>,
    ) -> AnimationValueResult<Self> {
        if values.len() < 2 {
            return Ok(Self {
                mapping: PhaseCurveMapping::Identity,
            });
        }
        let first = values[0];
        let last = values[values.len() - 1];
        let forward = first.checked_advance(CycleDelta::from_cycles(1.0)?)?;
        let reverse = first.checked_advance(CycleDelta::from_cycles(-1.0)?)?;
        if last != forward && last != reverse {
            return Err(AnimationValueError::InvalidPhaseCurveSeam);
        }
        Ok(Self {
            mapping: PhaseCurveMapping::Sampled(values),
        })
    }

    pub(in crate::animation) fn sample(
        &self,
        physical_phase: CyclePhase,
    ) -> AnimationValueResult<CyclePhase> {
        let PhaseCurveMapping::Sampled(values) = &self.mapping else {
            return Ok(physical_phase.wrapped());
        };
        let phase = physical_phase.wrapped().cycles();
        let coordinate = phase * (values.len() - 1) as f32;
        // f32 can round a very large grid endpoint upward; the actual native
        // index remains inside the admitted nonempty aligned sample extent.
        let lower = (coordinate.floor() as usize).min(values.len() - 1);
        let upper = (lower + 1).min(values.len() - 1);
        let amount = UnitInterval::from_fraction(coordinate - lower as f32)?;
        CyclePhase::from_arithmetic(
            values[lower]
                .cycles()
                .lerp(values[upper].cycles(), amount.fraction())
                .rem_euclid(1.0),
        )
    }
}

impl AuthoredPhaseCurve {
    pub(super) fn fit_contact_spans(
        spans: &[ContactPhaseSpan],
        orientation: ContactOrientation,
    ) -> AnimationValueResult<Self> {
        let intervals = pose_tuning().authored_phase_calibration_intervals.count();
        let mut authored_phases = (0..=intervals)
            .map(|index| {
                let physical = CyclePhase::from_cycles(index as f32 / intervals as f32)?;
                sample_authored_phase_spans(spans, physical, orientation)
            })
            .collect::<AnimationValueResult<Vec<_>>>()?;
        super::periodic_smoothing::smooth_periodic_phase_curve(&mut authored_phases, orientation)?;
        Self::from_samples(authored_phases)
    }
}

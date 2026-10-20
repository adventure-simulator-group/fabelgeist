//! Fit the periodic physical-cycle to authored-cycle calibration curve.
//!
//! Admitted mapping spans retain finite unwrapped coordinates and strictly
//! increasing physical samples. Native metre projection and interpolation
//! arithmetic stay within this calibration owner.

use super::*;
use adventuresim_building_generator::spatial_geometry::SignedLength;
use adventuresim_tactical_core::animation::AnimationQuantity;
pub(in crate::animation) use curve::AuthoredPhaseCurve;

mod curve;
mod periodic_smoothing;

/// Authored-cycle endpoint allowance for the fitted contact span, in cycles.
const CONTACT_SPAN_ENDPOINT_ALLOWANCE_CYCLES: f32 = 0.000_001;

/// Native rig-local trajectory used to classify height and select a travel axis.
/// The retained authored phase is unwrapped across the loop seam.
pub(super) struct FootTrajectorySample {
    pub(super) authored: CyclePhase,
    pub(super) position: Vec3,
}

pub(super) struct AuthoredContactSample {
    pub(super) authored: CyclePhase,
    pub(super) position: SignedLength,
}

pub(super) struct PhysicalContactSample {
    pub(super) physical: CyclePhase,
    pub(super) position: SignedLength,
}

pub(super) struct ContactPhaseFit {
    pub(super) curve: AuthoredPhaseCurve,
    pub(super) trajectories: Vec<Vec<PhysicalContactSample>>,
}

#[derive(Clone, Copy)]
pub(super) struct ContactOrientation {
    direction: TimelineDirection,
    offset: CyclePhase,
}

#[derive(Clone, Copy)]
struct PhaseMappingSample {
    physical: CyclePhase,
    authored: CyclePhase,
}

struct OrientedContactSample {
    mapping: PhaseMappingSample,
    position: SignedLength,
}

/// Nonempty and strictly increasing in physical phase. Authored coordinates
/// may decrease for reverse playback, and a singleton span is valid.
struct ContactPhaseSpan {
    samples: Vec<PhaseMappingSample>,
}

struct FittedContactSpan {
    mapping: ContactPhaseSpan,
    trajectory: Vec<PhysicalContactSample>,
}

/// Expected direction of a stance along its selected rig-local travel axis.
#[derive(Clone, Copy)]
pub(super) enum StanceDirection {
    #[cfg(test)]
    Increasing,
    Decreasing,
}

impl StanceDirection {
    pub(super) fn sign(self) -> f32 {
        match self {
            #[cfg(test)]
            Self::Increasing => 1.0,
            Self::Decreasing => -1.0,
        }
    }
}

impl ContactOrientation {
    pub(super) fn fit(
        segments: &[Vec<AuthoredContactSample>],
        expected_stance_direction: StanceDirection,
    ) -> AnimationValueResult<Option<Self>> {
        let mut displacements = Vec::with_capacity(segments.len());
        for samples in segments {
            let (Some(first), Some(last)) = (samples.first(), samples.last()) else {
                return Ok(None);
            };
            displacements.push(
                SignedLength::from_metres(
                    (last.position.metres() - first.position.metres())
                        * expected_stance_direction.sign(),
                )
                .map_err(|_| AnimationValueError::Overflow {
                    quantity: AnimationQuantity::CalibrationPosition,
                })?,
            );
        }
        let signed_stance_displacement = SignedLength::from_metres(
            displacements
                .iter()
                .map(|value| value.metres())
                .sum::<f32>(),
        )
        .map_err(|_| AnimationValueError::Overflow {
            quantity: AnimationQuantity::CalibrationPosition,
        })?;
        if signed_stance_displacement.metres().abs()
            <= pose_tuning().minimum_phase_fit_displacement_metres.metres()
        {
            return Ok(None);
        }
        let direction = if signed_stance_displacement.metres() < 0.0 {
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
                    let authored_center = (samples.first()?.authored.cycles()
                        + samples.last()?.authored.cycles())
                        * 0.5;
                    if direction == TimelineDirection::Reversed {
                        Some(authored_center + semantic_center)
                    } else {
                        Some(authored_center - semantic_center)
                    }
                });
        let Some(Some(first_offset)) = offsets.next() else {
            return Ok(None);
        };
        let first_offset = first_offset.rem_euclid(1.0);
        let Some((offset_sum, offset_count)) =
            offsets.try_fold((first_offset, 1_u32), |(sum, count), offset| {
                let offset = offset?;
                let aligned = first_offset + (offset - first_offset + 0.5).rem_euclid(1.0) - 0.5;
                Some((sum + aligned, count + 1))
            })
        else {
            return Ok(None);
        };
        let phase_offset = (offset_sum / offset_count as f32).rem_euclid(1.0);
        Ok(Some(Self {
            direction,
            offset: CyclePhase::from_arithmetic(phase_offset)?,
        }))
    }

    /// The fallback and smoothing baseline retain their original operand order.
    fn fallback(self, physical: CyclePhase) -> AnimationValueResult<CyclePhase> {
        CyclePhase::from_arithmetic(if self.direction == TimelineDirection::Reversed {
            self.offset.cycles() - physical.cycles()
        } else {
            physical.cycles() + self.offset.cycles()
        })
    }

    fn smoothing_baseline(self, physical: CyclePhase) -> AnimationValueResult<CyclePhase> {
        CyclePhase::from_arithmetic(if self.direction == TimelineDirection::Reversed {
            self.offset.cycles() - physical.cycles()
        } else {
            self.offset.cycles() + physical.cycles()
        })
    }

    fn orient_segment(
        self,
        samples: &[AuthoredContactSample],
    ) -> AnimationValueResult<Vec<OrientedContactSample>> {
        let orient = |sample: &AuthoredContactSample| {
            let physical =
                CyclePhase::from_arithmetic(if self.direction == TimelineDirection::Reversed {
                    self.offset.cycles() - sample.authored.cycles()
                } else {
                    sample.authored.cycles() - self.offset.cycles()
                })?;
            Ok(OrientedContactSample {
                mapping: PhaseMappingSample {
                    physical,
                    authored: sample.authored,
                },
                position: sample.position,
            })
        };
        if self.direction == TimelineDirection::Reversed {
            samples.iter().rev().map(orient).collect()
        } else {
            samples.iter().map(orient).collect()
        }
    }
}

impl ContactPhaseFit {
    pub(super) fn fit(
        segments: &[Vec<AuthoredContactSample>],
        orientation: ContactOrientation,
        expected_stance_direction: StanceDirection,
    ) -> AnimationValueResult<Option<Self>> {
        let mut spans = Vec::new();
        let mut trajectories = Vec::new();
        for samples in segments {
            let oriented = orientation.orient_segment(samples)?;
            let Some(fitted) =
                FittedContactSpan::fit_distance(&oriented, expected_stance_direction)?
            else {
                return Ok(None);
            };
            spans.push(fitted.mapping);
            trajectories.push(fitted.trajectory);
        }
        Ok(Some(Self {
            curve: AuthoredPhaseCurve::fit_contact_spans(&spans, orientation)?,
            trajectories,
        }))
    }
}

impl ContactPhaseSpan {
    fn new(
        first: PhaseMappingSample,
        remaining: Vec<PhaseMappingSample>,
    ) -> AnimationValueResult<Self> {
        let mut previous = first.physical;
        for sample in &remaining {
            let width = sample.physical.checked_difference(previous)?;
            if width.cycles() <= 0.0 {
                return Err(AnimationValueError::NonPositive {
                    quantity: AnimationQuantity::CycleDelta,
                });
            }
            previous = sample.physical;
        }
        let samples = std::iter::once(first).chain(remaining).collect();
        Ok(Self { samples })
    }

    fn sample(
        &self,
        physical: CyclePhase,
        orientation: ContactOrientation,
    ) -> AnimationValueResult<Option<CyclePhase>> {
        for cycle_offset in [-1.0, 0.0, 1.0] {
            let authored_offset =
                CycleDelta::from_cycles(if orientation.direction == TimelineDirection::Reversed {
                    -cycle_offset
                } else {
                    cycle_offset
                })?;
            let cycle_offset = CycleDelta::from_cycles(cycle_offset)?;
            // Admission proves a first sample; each selected interval has positive width.
            let first = self.samples[0];
            let last = self.samples[self.samples.len() - 1];
            let start = first.physical.checked_advance(cycle_offset)?;
            let end = last.physical.checked_advance(cycle_offset)?;
            let before = CyclePhase::from_arithmetic(
                physical.cycles() + CONTACT_SPAN_ENDPOINT_ALLOWANCE_CYCLES,
            )?;
            let after = CyclePhase::from_arithmetic(
                physical.cycles() - CONTACT_SPAN_ENDPOINT_ALLOWANCE_CYCLES,
            )?;
            if before.cycles() < start.cycles() || after.cycles() > end.cycles() {
                continue;
            }
            // The shifted comparison cannot overflow: the ordered endpoints above
            // have already admitted this shift for every intermediate coordinate.
            let upper = self.samples.partition_point(|sample| {
                sample.physical.cycles() + cycle_offset.cycles() < physical.cycles()
            });
            if upper == 0 {
                return first.authored.checked_advance(authored_offset).map(Some);
            }
            if upper >= self.samples.len() {
                return last.authored.checked_advance(authored_offset).map(Some);
            }
            let lower = self.samples[upper - 1];
            let upper = self.samples[upper];
            let shifted_lower = lower.physical.checked_advance(cycle_offset)?;
            let query_delta = physical.checked_difference(shifted_lower)?;
            let width = upper.physical.checked_difference(lower.physical)?;
            // Project the admitted cycle distances only for this interpolation kernel.
            let amount = query_delta.cycles() / width.cycles();
            if !amount.is_finite() {
                return Err(AnimationValueError::Overflow {
                    quantity: AnimationQuantity::CyclePhase,
                });
            }
            let amount = amount.clamp(0.0, 1.0);
            let lower_authored = lower.authored.checked_advance(authored_offset)?;
            let upper_authored = upper.authored.checked_advance(authored_offset)?;
            return CyclePhase::from_arithmetic(
                lower_authored
                    .cycles()
                    .lerp(upper_authored.cycles(), amount),
            )
            .map(Some);
        }
        Ok(None)
    }
}

impl FittedContactSpan {
    /// Invert travel-axis metres to physical phase. The derivative floor is the
    /// existing blend with uniform phase speed, not an admission tolerance.
    fn fit_distance(
        samples: &[OrientedContactSample],
        expected_stance_direction: StanceDirection,
    ) -> AnimationValueResult<Option<Self>> {
        let (Some(first), Some(last)) = (samples.first(), samples.last()) else {
            return Ok(None);
        };
        let phase_span = last
            .mapping
            .physical
            .checked_difference(first.mapping.physical)?;
        if phase_span.cycles() <= 0.0 {
            return Ok(None);
        }
        let positive_deltas = samples
            .windows(2)
            .map(|pair| {
                let displacement = SignedLength::from_metres(
                    (pair[1].position.metres() - pair[0].position.metres())
                        * expected_stance_direction.sign(),
                )
                .map_err(|_| AnimationValueError::Overflow {
                    quantity: AnimationQuantity::CalibrationPosition,
                })?;
                NonnegativeLength::from_arithmetic(displacement.metres().max(0.0))
            })
            .collect::<AnimationValueResult<Vec<_>>>()?;
        let positive_total = positive_deltas
            .iter()
            .map(|delta| delta.metres())
            .sum::<f32>();
        if positive_total <= 0.01 {
            return Ok(None);
        }
        let derivative_floor = positive_total * 0.5 / positive_deltas.len() as f32;
        let weighted_total = positive_total + derivative_floor * positive_deltas.len() as f32;
        let mut physical = first.mapping.physical;
        let mut remaining = Vec::new();
        let mut trajectory = vec![PhysicalContactSample {
            physical,
            position: first.position,
        }];
        for (index, delta) in positive_deltas.into_iter().enumerate() {
            // Native distance-weighted phase inversion; only the completed
            // quotient is a cycle displacement.
            let weighted_delta = phase_span.cycles() * (delta.metres() + derivative_floor);
            let step = CycleDelta::from_arithmetic(weighted_delta / weighted_total)?;
            physical = physical.checked_advance(step)?;
            remaining.push(PhaseMappingSample {
                physical,
                authored: samples[index + 1].mapping.authored,
            });
            trajectory.push(PhysicalContactSample {
                physical,
                position: samples[index + 1].position,
            });
        }
        // Preserve exact endpoints before admitting strictly increasing spans.
        if let Some(endpoint) = remaining.last_mut() {
            endpoint.physical = last.mapping.physical;
        }
        if let Some(endpoint) = trajectory.last_mut() {
            endpoint.physical = last.mapping.physical;
        }
        Ok(Some(Self {
            mapping: ContactPhaseSpan::new(first.mapping, remaining)?,
            trajectory,
        }))
    }
}

/// Classify low-height samples inside the native sampled-position calibration kernel.
pub(super) fn low_height_contacts(
    samples: &[FootTrajectorySample],
) -> AnimationValueResult<Option<Vec<bool>>> {
    let minimum_height = samples
        .iter()
        .map(|sample| sample.position.y)
        .reduce(f32::min);
    let maximum_height = samples
        .iter()
        .map(|sample| sample.position.y)
        .reduce(f32::max);
    let (Some(minimum_height), Some(maximum_height)) = (minimum_height, maximum_height) else {
        return Ok(None);
    };
    let height_window = ((maximum_height - minimum_height)
        * pose_tuning().authored_contact_height_fraction.factor())
    .clamp(
        pose_tuning()
            .authored_contact_minimum_height_window_metres
            .metres(),
        pose_tuning()
            .authored_contact_maximum_height_window_metres
            .metres(),
    );
    if !height_window.is_finite() || !(minimum_height + height_window).is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::CalibrationPosition,
        });
    }
    let supported = samples
        .iter()
        .map(|sample| sample.position.y <= minimum_height + height_window)
        .collect::<Vec<_>>();
    Ok(Some(supported))
}

fn sample_authored_phase_spans(
    spans: &[ContactPhaseSpan],
    physical: CyclePhase,
    orientation: ContactOrientation,
) -> AnimationValueResult<CyclePhase> {
    for span in spans {
        if let Some(authored) = span.sample(physical, orientation)? {
            return Ok(authored);
        }
    }
    orientation.fallback(physical)
}

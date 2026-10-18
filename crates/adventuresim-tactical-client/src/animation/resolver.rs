//! Resolve semantic samples into authored clip plans.

use super::*;
use adventuresim_tactical_core::animation::AnimationPackId;

mod span;

#[derive(Clone, Copy)]
pub(super) struct PoseSampleResolver<'a> {
    pub(super) runtime: &'a AnimationRuntime,
    pub(super) catalog: &'a AnimationPackCatalog,
    pub(super) pack: &'a AnimationPackId,
    pub(super) locomotion_strides: &'a AuthoredLocomotionStrides,
}

pub(super) struct ResolvedBodyPlan {
    pub(super) clips: Vec<WeightedClip>,
    extrapolated_spans: Vec<ExtrapolatedSpan>,
    continuation_spans: Vec<ContinuationSpan>,
    whole_body_mirror: BlendWeight,
}

impl ResolvedBodyPlan {
    pub(super) fn into_playback(
        self,
        evaluation: &AnimationEvaluation,
    ) -> AnimationValueResult<PlaybackPose> {
        // Grip contributions join after body resolution. Validate the complete
        // nonnegative total before the native per-joint blending kernel runs.
        self.clips
            .iter()
            .map(|clip| clip.weight)
            .chain(self.extrapolated_spans.iter().map(|span| span.weight))
            .chain(self.continuation_spans.iter().map(|span| span.weight))
            .try_fold(BlendWeight::ZERO, |total, weight| total.checked_add(weight))?;
        Ok(PlaybackPose {
            pose_source: PoseSource::for_empty_plan(
                self.clips.is_empty()
                    && self.extrapolated_spans.is_empty()
                    && self.continuation_spans.is_empty(),
            ),
            whole_body_mirror: self.whole_body_mirror,
            foot_ik_weights: semantic_foot_ik_weights(evaluation)?,
            cadence: if evaluation.action.is_empty() {
                PoseSamplingCadence::Buffered
            } else {
                PoseSamplingCadence::Direct
            },
            clips: self.clips,
            extrapolated_spans: self.extrapolated_spans,
            continuation_spans: self.continuation_spans,
        })
    }
}

impl PoseSampleResolver<'_> {
    pub(super) fn resolve_body(
        &self,
        skeleton: &SkeletonState,
        evaluation: &AnimationEvaluation,
    ) -> AnimationValueResult<ResolvedBodyPlan> {
        let mut weighted = Vec::<WeightedClip>::new();
        let mut extrapolated_spans = Vec::<ExtrapolatedSpan>::new();
        let mut continuation_spans = Vec::<ContinuationSpan>::new();
        let samples = if evaluation.action.is_empty() {
            &evaluation.base
        } else {
            &evaluation.action
        };
        let split_combat_pelvis = combat_pelvis_is_component_blended(skeleton, evaluation);
        let base_layer = if split_combat_pelvis {
            ClipLayer::CombatUpper
        } else if !evaluation.lower_body.is_empty() {
            ClipLayer::Upper
        } else {
            ClipLayer::Whole
        };
        for sample in samples {
            self.append_layer(
                &mut weighted,
                &mut extrapolated_spans,
                &mut continuation_spans,
                *sample,
                base_layer,
            )?;
        }
        let lower_layer = if split_combat_pelvis {
            ClipLayer::CombatLower
        } else {
            ClipLayer::Lower
        };
        for sample in &evaluation.lower_body {
            self.append_layer(
                &mut weighted,
                &mut extrapolated_spans,
                &mut continuation_spans,
                *sample,
                lower_layer,
            )?;
        }
        let whole_body_mirror = mirror_fraction(
            weighted
                .iter()
                .map(|clip| (clip.weight, clip.mirrored_weight))
                .chain(
                    extrapolated_spans
                        .iter()
                        .map(|span| (span.weight, span.mirrored_weight)),
                )
                .chain(
                    continuation_spans
                        .iter()
                        .map(|span| (span.weight, span.mirrored_weight)),
                ),
        )?;
        Ok(ResolvedBodyPlan {
            whole_body_mirror,
            clips: weighted,
            extrapolated_spans,
            continuation_spans,
        })
    }

    pub(super) fn append_layer(
        &self,
        weighted: &mut Vec<WeightedClip>,
        extrapolated_spans: &mut Vec<ExtrapolatedSpan>,
        continuation_spans: &mut Vec<ContinuationSpan>,
        sample: PoseSample,
        layer: ClipLayer,
    ) -> AnimationValueResult<()> {
        let start = resolve_anchor(self.runtime, self.catalog, self.pack, sample.pose);
        let Some(start) = start.and_then(|resolved| {
            select_gait_endpoint_parity(
                self.runtime,
                resolved,
                ClipMirror::from_resolved_pose(sample.mirror_lower_body),
            )
        }) else {
            return Ok(());
        };
        match sample.sampling {
            PoseSampling::Anchor => {
                append_weighted_anchor(weighted, &start, start.anchor.frame, sample.weight, layer)
            }
            PoseSampling::Cycle { .. } | PoseSampling::Timeline { .. } => {
                self.append_timed_sample(weighted, &start, sample, layer)
            }
            PoseSampling::Span { .. } => {
                self.append_interpolated_span(weighted, &start, sample, layer)
            }
            PoseSampling::CurveSpan { .. } => {
                self.append_curve_span(weighted, extrapolated_spans, &start, sample, layer)
            }
            sampling @ PoseSampling::ContinuationSpan { .. } => append_continuation_span(
                self,
                weighted,
                continuation_spans,
                &start,
                sample,
                sampling,
                layer,
            ),
        }
    }

    fn append_timed_sample(
        &self,
        weighted: &mut Vec<WeightedClip>,
        start: &ResolvedAnchor<'_>,
        sample: PoseSample,
        layer: ClipLayer,
    ) -> AnimationValueResult<()> {
        match sample.sampling {
            PoseSampling::Cycle { phase } => {
                let phase = CyclePhase::from_cycles(phase)?;
                let sample_phase = self.locomotion_strides.sample_phase(sample.pose, phase)?;
                let clip = start.clip.at_anchor_layer(start.anchor.frame, layer);
                append_weighted_clip(
                    weighted,
                    &clip,
                    start.mirrored,
                    start.clip.duration_seconds.at_phase(sample_phase)?,
                    BlendWeight::from_factor(sample.weight)?,
                    matches!(
                        sample.pose,
                        SemanticPose::WalkContact | SemanticPose::RunContact
                    )
                    .then_some(phase.wrapped()),
                )
            }
            PoseSampling::Timeline { progress } => {
                let progress = CyclePhase::from_cycles(progress)?.cycles().clamp(0.0, 1.0);
                let timeline = if start.timeline_direction == TimelineDirection::Reversed {
                    1.0 - progress
                } else {
                    progress
                };
                let clip = start.clip.at_anchor_layer(start.anchor.frame, layer);
                append_weighted_clip(
                    weighted,
                    &clip,
                    start.mirrored,
                    start
                        .clip
                        .duration_seconds
                        .at_phase(CyclePhase::from_cycles(timeline)?)?,
                    BlendWeight::from_factor(sample.weight)?,
                    None,
                )
            }
            _ => Ok(()),
        }
    }
}

fn mirror_fraction(
    contributions: impl Iterator<Item = (BlendWeight, BlendWeight)>,
) -> AnimationValueResult<BlendWeight> {
    let mut total = BlendWeight::ZERO;
    let mut mirrored = BlendWeight::ZERO;
    for (weight, mirror) in contributions {
        total = total.checked_add(weight)?;
        mirrored = mirrored.checked_add(mirror)?;
    }
    if total.is_absent() {
        return Ok(BlendWeight::ZERO);
    }
    BlendWeight::from_factor((mirrored.factor() / total.factor()).clamp(0.0, 1.0))
}

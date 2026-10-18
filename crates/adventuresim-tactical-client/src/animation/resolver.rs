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
    pub(super) fn into_playback(self, evaluation: &AnimationEvaluation) -> PlaybackPose {
        PlaybackPose {
            pose_source: PoseSource::for_empty_plan(
                self.clips.is_empty()
                    && self.extrapolated_spans.is_empty()
                    && self.continuation_spans.is_empty(),
            ),
            whole_body_mirror: self.whole_body_mirror,
            foot_ik_weights: semantic_foot_ik_weights(evaluation),
            cadence: if evaluation.action.is_empty() {
                PoseSamplingCadence::Buffered
            } else {
                PoseSamplingCadence::Direct
            },
            clips: self.clips,
            extrapolated_spans: self.extrapolated_spans,
            continuation_spans: self.continuation_spans,
        }
    }
}

impl PoseSampleResolver<'_> {
    pub(super) fn resolve_body(
        &self,
        skeleton: &SkeletonState,
        evaluation: &AnimationEvaluation,
    ) -> ResolvedBodyPlan {
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
            );
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
            );
        }
        let (total_weight, mirrored_weight) = weighted
            .iter()
            .map(|clip| (clip.weight.factor(), clip.mirrored_weight.factor()))
            .chain(
                extrapolated_spans
                    .iter()
                    .map(|span| (span.weight, span.mirrored_weight)),
            )
            .chain(
                continuation_spans
                    .iter()
                    .map(|span| (span.weight, span.mirrored_weight)),
            )
            .fold((0.0, 0.0), |(total, mirrored), (weight, mirror)| {
                (total + weight, mirrored + mirror)
            });
        let whole_body_mirror = if total_weight > 0.0 {
            (mirrored_weight / total_weight).clamp(0.0, 1.0)
        } else {
            0.0
        };
        ResolvedBodyPlan {
            whole_body_mirror: BlendWeight::from_factor(whole_body_mirror),
            clips: weighted,
            extrapolated_spans,
            continuation_spans,
        }
    }

    pub(super) fn append_layer(
        &self,
        weighted: &mut Vec<WeightedClip>,
        extrapolated_spans: &mut Vec<ExtrapolatedSpan>,
        continuation_spans: &mut Vec<ContinuationSpan>,
        sample: PoseSample,
        layer: ClipLayer,
    ) {
        let start = resolve_anchor(self.runtime, self.catalog, self.pack, sample.pose);
        let Some(start) = start.and_then(|resolved| {
            select_gait_endpoint_parity(
                self.runtime,
                resolved,
                ClipMirror::from_resolved_pose(sample.mirror_lower_body),
            )
        }) else {
            return;
        };
        match sample.sampling {
            PoseSampling::Anchor => {
                append_weighted_anchor(weighted, &start, start.anchor.frame, sample.weight, layer)
            }
            PoseSampling::Cycle { .. } | PoseSampling::Timeline { .. } => {
                self.append_timed_sample(weighted, &start, sample, layer);
            }
            PoseSampling::Span { .. } => {
                self.append_interpolated_span(weighted, &start, sample, layer);
            }
            PoseSampling::CurveSpan { .. } => {
                self.append_curve_span(weighted, extrapolated_spans, &start, sample, layer);
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
    ) {
        match sample.sampling {
            PoseSampling::Cycle { phase } => {
                let sample_phase = self.locomotion_strides.sample_phase(sample.pose, phase);
                let clip = start.clip.at_anchor_layer(start.anchor.frame, layer);
                append_weighted_clip(
                    weighted,
                    &clip,
                    start.mirrored,
                    start
                        .clip
                        .duration_seconds
                        .at_phase(CyclePhase::from_cycles(sample_phase)),
                    BlendWeight::from_factor(sample.weight),
                    matches!(
                        sample.pose,
                        SemanticPose::WalkContact | SemanticPose::RunContact
                    )
                    .then_some(CyclePhase::from_cycles(phase.rem_euclid(1.0))),
                );
            }
            PoseSampling::Timeline { progress } => {
                let progress = progress.clamp(0.0, 1.0);
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
                        .at_phase(CyclePhase::from_cycles(timeline)),
                    BlendWeight::from_factor(sample.weight),
                    None,
                );
            }
            _ => {}
        }
    }
}

//! Select authored span endpoints and preserve reference-anchor priority.

use super::*;

struct ResolvedSpanEndpoint<'a> {
    anchor: ResolvedAnchor<'a>,
    frame: AuthoredAnimationFrame,
}

struct ResolvedSpan<'a> {
    start: ResolvedSpanEndpoint<'a>,
    end: ResolvedSpanEndpoint<'a>,
}

impl<'a> PoseSampleResolver<'a> {
    pub(super) fn append_interpolated_span(
        &self,
        weighted: &mut Vec<WeightedClip>,
        start: &ResolvedAnchor<'a>,
        sample: PoseSample,
        layer: ClipLayer,
    ) -> AnimationValueResult<()> {
        let PoseSampling::Span { end, progress } = sample.sampling else {
            return Ok(());
        };
        let progress = CyclePhase::from_cycles(progress)?.cycles().clamp(0.0, 1.0);
        let Some(span) = self.resolve_span(start, sample.pose, end) else {
            return append_weighted_anchor(
                weighted,
                start,
                start.anchor.frame,
                sample.weight,
                layer,
            );
        };
        append_weighted_anchor(
            weighted,
            &span.start.anchor,
            span.start.frame,
            sample.weight * (1.0 - progress),
            layer,
        )?;
        append_weighted_anchor(
            weighted,
            &span.end.anchor,
            span.end.frame,
            sample.weight * progress,
            layer,
        )
    }

    pub(super) fn append_curve_span(
        &self,
        weighted: &mut Vec<WeightedClip>,
        extrapolated_spans: &mut Vec<ExtrapolatedSpan>,
        start: &ResolvedAnchor<'a>,
        sample: PoseSample,
        layer: ClipLayer,
    ) -> AnimationValueResult<()> {
        let PoseSampling::CurveSpan { end, coordinate } = sample.sampling else {
            return Ok(());
        };
        let coordinate = CyclePhase::from_cycles(coordinate)?.cycles().clamp(
            -AttackCurve::maximum_drawback(),
            1.0 + AttackCurve::maximum_overshoot(),
        );
        let Some(span) = self.resolve_span(start, sample.pose, end) else {
            return append_weighted_anchor(
                weighted,
                start,
                start.anchor.frame,
                sample.weight,
                layer,
            );
        };
        extrapolated_spans.push(ExtrapolatedSpan {
            start: span
                .start
                .anchor
                .clip
                .at_anchor_layer(span.start.frame, layer),
            start_time_seconds: frame_seconds(span.start.frame)?,
            end: span.end.anchor.clip.at_anchor_layer(span.end.frame, layer),
            end_time_seconds: frame_seconds(span.end.frame)?,
            coordinate,
            weight: BlendWeight::from_factor(sample.weight)?,
            mirrored_weight: if span.start.anchor.mirrored == ClipMirror::Mirrored {
                BlendWeight::from_factor(sample.weight)?
            } else {
                BlendWeight::ZERO
            },
        });
        Ok(())
    }

    fn resolve_span(
        &self,
        start: &ResolvedAnchor<'a>,
        start_pose: SemanticPose,
        end_pose: SemanticPose,
    ) -> Option<ResolvedSpan<'a>> {
        let end = resolve_anchor(self.runtime, self.catalog, self.pack, end_pose)?;
        let mut span = ResolvedSpan {
            start: ResolvedSpanEndpoint {
                anchor: start.clone(),
                frame: start.anchor.frame,
            },
            end: ResolvedSpanEndpoint {
                anchor: end.clone(),
                frame: end.anchor.frame,
            },
        };
        if start.pack_id == end.pack_id && start.anchor.motion == end.anchor.motion {
            return Some(span);
        }
        if let Some(reference) = self.reference_for(&end, start_pose) {
            span.start = ResolvedSpanEndpoint {
                anchor: end,
                frame: reference.frame,
            };
        } else if let Some(reference) = self.reference_for(start, end_pose) {
            span.end = ResolvedSpanEndpoint {
                anchor: start.clone(),
                frame: reference.frame,
            };
        }
        Some(span)
    }

    fn reference_for(
        &self,
        anchor: &ResolvedAnchor<'a>,
        pose: SemanticPose,
    ) -> Option<&'a ReferenceAnchor> {
        self.catalog.packs[anchor.pack_id]
            .references
            .get(&anchor.anchor.motion)?
            .iter()
            .filter(|reference| reference.pose == pose)
            .min_by_key(|reference| reference.frame.abs_diff(anchor.anchor.frame))
    }
}

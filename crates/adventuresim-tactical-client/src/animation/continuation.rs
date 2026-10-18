use super::*;

#[derive(Debug, Clone)]
pub(super) struct ContinuationSpan {
    pub(super) start: LoadedClip,
    pub(super) start_time_seconds: ClipTime,
    pub(super) contact: LoadedClip,
    pub(super) contact_time_seconds: ClipTime,
    pub(super) end: LoadedClip,
    pub(super) end_time_seconds: ClipTime,
    pub(super) outgoing: LoadedClip,
    pub(super) outgoing_time_seconds: ClipTime,
    pub(super) finish: LoadedClip,
    pub(super) finish_time_seconds: ClipTime,
    pub(super) start_coordinate: f32,
    pub(super) incoming_tangent: f32,
    pub(super) ready_phase: f32,
    pub(super) progress: f32,
    pub(super) weight: BlendWeight,
    pub(super) mirrored_weight: BlendWeight,
}

pub(super) fn append_continuation_span(
    resolver: &PoseSampleResolver<'_>,
    weighted: &mut Vec<WeightedClip>,
    spans: &mut Vec<ContinuationSpan>,
    start: &ResolvedAnchor<'_>,
    sample: PoseSample,
    sampling: PoseSampling,
    layer: ClipLayer,
) -> AnimationValueResult<()> {
    let PoseSampleResolver {
        runtime,
        catalog,
        pack,
        ..
    } = *resolver;
    let PoseSampling::ContinuationSpan {
        contact,
        end,
        outgoing,
        finish,
        start_coordinate,
        incoming_tangent,
        ready_phase,
        progress,
    } = sampling
    else {
        unreachable!("continuation resolver requires continuation sampling");
    };
    let Some(contact) = resolve_anchor(runtime, catalog, pack, contact) else {
        return append_weighted_anchor(weighted, start, start.anchor.frame, sample.weight, layer);
    };
    let Some(end) = resolve_anchor(runtime, catalog, pack, end) else {
        return append_weighted_anchor(weighted, start, start.anchor.frame, sample.weight, layer);
    };
    let Some(outgoing) = resolve_anchor(runtime, catalog, pack, outgoing) else {
        return append_weighted_anchor(weighted, &end, end.anchor.frame, sample.weight, layer);
    };
    let Some(finish) = resolve_anchor(runtime, catalog, pack, finish) else {
        return append_weighted_anchor(
            weighted,
            &outgoing,
            outgoing.anchor.frame,
            sample.weight,
            layer,
        );
    };
    spans.push(ContinuationSpan {
        start: start.clip.at_anchor_layer(start.anchor.frame, layer),
        start_time_seconds: frame_seconds(start.anchor.frame)?,
        contact: contact.clip.at_anchor_layer(contact.anchor.frame, layer),
        contact_time_seconds: frame_seconds(contact.anchor.frame)?,
        end: end.clip.at_anchor_layer(end.anchor.frame, layer),
        end_time_seconds: frame_seconds(end.anchor.frame)?,
        outgoing: outgoing.clip.at_anchor_layer(outgoing.anchor.frame, layer),
        outgoing_time_seconds: frame_seconds(outgoing.anchor.frame)?,
        finish: finish.clip.at_anchor_layer(finish.anchor.frame, layer),
        finish_time_seconds: frame_seconds(finish.anchor.frame)?,
        start_coordinate: finite_sample_value(start_coordinate)?,
        incoming_tangent: finite_sample_value(incoming_tangent)?.max(0.0),
        ready_phase: finite_sample_value(ready_phase)?,
        progress: finite_sample_value(progress)?.clamp(0.0, 1.0),
        weight: sample.weight,
        mirrored_weight: if start.mirrored == super::ClipMirror::Mirrored {
            sample.weight
        } else {
            BlendWeight::ZERO
        },
    });
    Ok(())
}

//! Checked contributions coalesced at authored sample positions.

use super::*;

#[derive(Debug, Clone)]
pub(super) struct WeightedClip {
    pub(super) clip: LoadedClip,
    pub(super) weight: BlendWeight,
    pub(super) time_seconds: ClipTime,
    pub(super) mirrored_weight: BlendWeight,
    /// Unwarped physical phase for ordinary sparse-pose locomotion. Its
    /// presence selects the Bevy-side semantic-anchor sampler while
    /// `time_seconds` retains the measured authored phase coordinate.
    pub(super) locomotion_phase: Option<CyclePhase>,
}

pub(super) fn append_weighted_anchor(
    weighted: &mut Vec<WeightedClip>,
    resolved: &ResolvedAnchor,
    frame: u16,
    weight: f32,
    layer: ClipLayer,
) -> AnimationValueResult<()> {
    let clip = resolved.clip.at_anchor_layer(frame, layer);
    append_weighted_clip(
        weighted,
        &clip,
        resolved.mirrored,
        frame_seconds(frame)?,
        BlendWeight::from_factor(weight)?,
        None,
    )
}

pub(super) fn append_weighted_clip(
    weighted: &mut Vec<WeightedClip>,
    clip: &LoadedClip,
    mirrored: ClipMirror,
    time_seconds: ClipTime,
    weight: BlendWeight,
    locomotion_phase: Option<CyclePhase>,
) -> AnimationValueResult<()> {
    if weight.is_absent() {
        return Ok(());
    }
    if let Some(existing) = weighted.iter_mut().find(|existing| {
        existing.clip.handle.id() == clip.handle.id()
            && existing.clip.layer == clip.layer
            && existing.time_seconds.same_sample(time_seconds)
            && existing.locomotion_phase == locomotion_phase
    }) {
        let total = existing.weight.checked_add(weight)?;
        let mirrored_total = if mirrored == ClipMirror::Mirrored {
            existing.mirrored_weight.checked_add(weight)?
        } else {
            existing.mirrored_weight
        };
        existing.weight = total;
        existing.mirrored_weight = mirrored_total;
    } else {
        weighted.push(WeightedClip {
            clip: clip.clone(),
            weight,
            time_seconds,
            mirrored_weight: if mirrored == ClipMirror::Mirrored {
                weight
            } else {
                BlendWeight::ZERO
            },
            locomotion_phase,
        });
    }
    Ok(())
}

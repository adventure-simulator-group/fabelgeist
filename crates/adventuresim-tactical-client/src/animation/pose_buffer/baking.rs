//! Source-clip admission, fixed-rate bake grids and cached sampling.

use super::*;

#[derive(Clone)]
pub(super) struct BakedClip {
    pub(super) duration: ClipDuration,
    pub(super) frame_dt: AnimationElapsed,
    pub(super) frames: usize,
    pub(super) tracks: RigJointTable<BoneTrack>,
}

#[derive(Clone)]
pub(super) struct BoneTrack {
    pub(super) translations: Vec<Vec3>,
    pub(super) rotations: Vec<Quat>,
    pub(super) scales: Vec<Vec3>,
    pub(super) source: BoneTrackSource,
}

/// Whether the source clip owns this joint or sampling retained its bind transform.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BoneTrackSource {
    Authored,
    BindPose,
}

/// One joint's local transform in the pose buffer.
impl BakedClip {
    pub(super) fn sample(&self, joint: RigJointOrdinal, time_seconds: ClipTime) -> LocalPose {
        // Only this interpolation kernel projects seconds to a native grid index.
        let frame = if self.frames == 1 || self.frame_dt.is_zero() {
            0.0
        } else {
            (time_seconds.seconds().clamp(0.0, self.duration.seconds()) / self.frame_dt.seconds())
                .min((self.frames - 1) as f32)
        };
        let first = frame as usize;
        let second = (first + 1).min(self.frames - 1);
        let alpha = frame.fract();
        let track = &self.tracks[joint];
        LocalPose {
            translation: track.translations[first].lerp(track.translations[second], alpha),
            rotation: hemisphere_slerp(track.rotations[first], track.rotations[second], alpha),
            scale: track.scales[first].lerp(track.scales[second], alpha),
        }
    }

    pub(super) fn frame_time(&self, frame: usize) -> AnimationValueResult<ClipTime> {
        // A bake-grid ordinal becomes a source seconds coordinate here.
        ClipTime::from_seconds(frame as f32 * self.frame_dt.seconds())
    }

    fn memory_bytes(&self) -> usize {
        self.tracks
            .iter()
            .map(|track| {
                track.translations.len() * size_of::<Vec3>()
                    + track.rotations.len() * size_of::<Quat>()
                    + track.scales.len() * size_of::<Vec3>()
            })
            .sum()
    }
}

pub(super) fn get_or_bake(
    id: AssetId<AnimationClip>,
    handle: &Handle<AnimationClip>,
    definition: &RigDefinition,
    clips: &Assets<AnimationClip>,
    bank: &mut BakedClipBank,
    metrics: &mut PoseBufferMetrics,
) -> Option<Arc<BakedClip>> {
    let key = (definition.family.clone(), id);
    if let Some(clip) = bank.0.get(&key) {
        return Some(clip.clone());
    }
    let clip = clips.get(handle)?;
    let baked = match bake_clip(clip, definition) {
        Ok(baked) => Arc::new(baked),
        Err(error) => {
            warn!(%error, ?id, "Rejected animation bake grid");
            return None;
        }
    };
    metrics.baked_clip_bytes = metrics
        .baked_clip_bytes
        .saturating_add(baked.memory_bytes());
    metrics.baked_clip_count = metrics.baked_clip_count.saturating_add(1);
    bank.0.insert(key, baked.clone());
    Some(baked)
}

pub(super) fn bake_clip(
    clip: &AnimationClip,
    definition: &RigDefinition,
) -> AnimationValueResult<BakedClip> {
    let source_duration = ClipDuration::from_seconds(clip.duration())?;
    // Preserve the existing minimum sampling-grid extent, after source admission.
    let duration = ClipDuration::from_seconds(
        source_duration
            .seconds()
            .max(pose_tuning().minimum_baked_grid_seconds.seconds()),
    )?;
    let frame_dt = AnimationElapsed::from_seconds(pose_sample_seconds())?;
    let grid_span = duration.seconds() / frame_dt.seconds();
    if frame_dt.is_zero() || !grid_span.is_finite() || grid_span.ceil() >= usize::MAX as f32 {
        return Err(AnimationValueError::Overflow {
            quantity: adventuresim_tactical_core::animation::AnimationQuantity::ClipTime,
        });
    }
    let frames = grid_span.ceil() as usize + 1;
    if !((frames - 1) as f32 * frame_dt.seconds()).is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: adventuresim_tactical_core::animation::AnimationQuantity::ClipTime,
        });
    }
    let translation_field = animated_field!(Transform::translation);
    let rotation_field = animated_field!(Transform::rotation);
    let scale_field = animated_field!(Transform::scale);
    let tracks: RigJointTable<BoneTrack> = definition
        .joints
        .iter()
        .map(|joint| {
            let sample_time =
                |frame: usize| (frame as f32 * frame_dt.seconds()).min(duration.seconds());
            let translations = (0..frames)
                .map(|frame| {
                    clip.sample_clamped(translation_field.clone(), joint.target, sample_time(frame))
                        .unwrap_or(joint.bind.translation)
                })
                .collect::<Vec<_>>();
            let rotations = (0..frames)
                .map(|frame| {
                    clip.sample_clamped(rotation_field.clone(), joint.target, sample_time(frame))
                        .unwrap_or(joint.bind.rotation)
                })
                .collect::<Vec<_>>();
            let scales = (0..frames)
                .map(|frame| {
                    clip.sample_clamped(scale_field.clone(), joint.target, sample_time(frame))
                        .unwrap_or(joint.bind.scale)
                })
                .collect::<Vec<_>>();
            BoneTrack {
                translations,
                rotations,
                scales,
                source: if clip.curves().contains_key(&joint.target) {
                    BoneTrackSource::Authored
                } else {
                    BoneTrackSource::BindPose
                },
            }
        })
        .collect();
    Ok(BakedClip {
        duration,
        frame_dt,
        frames,
        tracks,
    })
}

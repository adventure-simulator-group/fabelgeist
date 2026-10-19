//! Source-clip admission, fixed-rate bake grids and cached sampling.

use super::*;
use adventuresim_tactical_core::animation::{AnimationQuantity, PositiveAnimationDuration};
use std::num::NonZeroUsize;

#[derive(Clone)]
pub(super) struct BakedClip {
    pub(super) duration: ClipDuration,
    frame_dt: PositiveAnimationDuration,
    frames: BakedFrameCount,
    tracks: RigJointTable<BoneTrack>,
}

/// Stored samples, including the endpoint: 64 intervals require 65 samples.
/// A one-sample clip is valid and has no interpolation interval.
#[derive(Clone, Copy)]
pub(super) struct BakedFrameCount(NonZeroUsize);

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

impl TryFrom<usize> for BakedFrameCount {
    type Error = AnimationValueError;

    fn try_from(samples: usize) -> AnimationValueResult<Self> {
        NonZeroUsize::new(samples)
            .map(Self)
            .ok_or(AnimationValueError::NonPositive {
                quantity: AnimationQuantity::BakedFrameCount,
            })
    }
}

impl BakedFrameCount {
    /// Native grid cardinality for allocation, indexing and calibration kernels.
    pub(super) fn samples(self) -> usize {
        self.0.get()
    }

    pub(super) fn intervals(self) -> usize {
        self.0.get() - 1
    }
}

impl BakedClip {
    /// Admit the complete grid before sampling can index any channel. Private
    /// storage and slice-only translation edits preserve these channel lengths.
    pub(super) fn new(
        duration: ClipDuration,
        frame_dt: PositiveAnimationDuration,
        frames: BakedFrameCount,
        tracks: RigJointTable<BoneTrack>,
    ) -> AnimationValueResult<Self> {
        for track in &tracks {
            for actual in [
                track.translations.len(),
                track.rotations.len(),
                track.scales.len(),
            ] {
                if actual != frames.samples() {
                    return Err(AnimationValueError::BakedChannelSampleCount {
                        expected: frames.samples(),
                        actual,
                    });
                }
            }
        }
        Ok(Self {
            duration,
            frame_dt,
            frames,
            tracks,
        })
    }

    pub(super) fn frame_count(&self) -> BakedFrameCount {
        self.frames
    }

    pub(super) fn track_source(&self, joint: RigJointOrdinal) -> BoneTrackSource {
        self.tracks[joint].source
    }

    pub(super) fn translations_mut(&mut self, joint: RigJointOrdinal) -> &mut [Vec3] {
        &mut self.tracks[joint].translations
    }

    pub(super) fn offset_translations(&mut self, offsets: &[Vec3]) {
        for (joint, offset) in self.tracks.ordinals().zip(offsets) {
            for translation in self.translations_mut(joint) {
                *translation += *offset;
            }
        }
    }

    /// One joint's local transform in the pose buffer.
    pub(super) fn sample(&self, joint: RigJointOrdinal, time_seconds: ClipTime) -> LocalPose {
        // Only this interpolation kernel projects seconds to a native grid index.
        let frame = if self.frames.samples() == 1 {
            0.0
        } else {
            (time_seconds.seconds().clamp(0.0, self.duration.seconds()) / self.frame_dt.seconds())
                .min(self.frames.intervals() as f32)
        };
        let first = frame as usize;
        let second = (first + 1).min(self.frames.intervals());
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
    let frame_dt = pose_sample_interval();
    let grid_span = duration.seconds() / frame_dt.seconds();
    if !grid_span.is_finite() || grid_span.ceil() >= usize::MAX as f32 {
        return Err(AnimationValueError::Overflow {
            quantity: adventuresim_tactical_core::animation::AnimationQuantity::ClipTime,
        });
    }
    let frames = BakedFrameCount::try_from(grid_span.ceil() as usize + 1)?;
    if !(frames.intervals() as f32 * frame_dt.seconds()).is_finite() {
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
            let translations = (0..frames.samples())
                .map(|frame| {
                    clip.sample_clamped(translation_field.clone(), joint.target, sample_time(frame))
                        .unwrap_or(joint.bind.translation)
                })
                .collect::<Vec<_>>();
            let rotations = (0..frames.samples())
                .map(|frame| {
                    clip.sample_clamped(rotation_field.clone(), joint.target, sample_time(frame))
                        .unwrap_or(joint.bind.rotation)
                })
                .collect::<Vec<_>>();
            let scales = (0..frames.samples())
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
    BakedClip::new(duration, frame_dt, frames, tracks)
}

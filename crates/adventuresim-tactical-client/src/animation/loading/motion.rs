use super::{
    AnimationAssetPath, AnimationTargetId, ClipDuration, ClipLayer, LoadedClip, MotionKey,
    MotionSource, animation_asset_path, clip_targets_match_base, frame_fits_clip, sole_animation,
};
use bevy::{
    asset::{AssetLoadError, LoadState, io::AssetReaderError},
    gltf::Gltf,
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};
use std::collections::HashSet;

/// Optional motions own their load diagnostics so expected gaps do not emit
/// Bevy's unconditional asset-load errors before pose fallback can run.
pub(crate) struct MotionLoad {
    path: AnimationAssetPath,
    state: MotionLoadState,
}

pub(super) enum MotionClipAdmission {
    Pending,
    Unavailable,
    Ready(LoadedClip),
}

enum MotionLoadState {
    Pending(Task<Option<Handle<Gltf>>>),
    Ready(Option<Handle<Gltf>>),
}

impl MotionLoad {
    pub(super) fn request(asset_server: &AssetServer, path: &AnimationAssetPath) -> Self {
        let requested = path.clone();
        let path = animation_asset_path(path);
        let asset_server = asset_server.clone();
        let task = IoTaskPool::get().spawn(async move {
            match asset_server.load_builder().load_untyped_async(&path).await {
                Ok(handle) => match handle.try_typed::<Gltf>() {
                    Ok(handle) => Some(handle),
                    Err(error) => {
                        error!(path = %path, %error, "Animation motion has the wrong asset type");
                        None
                    }
                },
                Err(AssetLoadError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                    info!(
                        path = %path,
                        "Animation motion is absent; using available pose fallbacks"
                    );
                    None
                }
                Err(error) => {
                    error!(path = %path, %error, "Animation motion failed to load");
                    None
                }
            }
        });
        Self {
            path: requested,
            state: MotionLoadState::Pending(task),
        }
    }

    /// Retain the source handle even while its current load state is failed.
    pub(in crate::animation) fn source(&self) -> Option<&Handle<Gltf>> {
        match &self.state {
            MotionLoadState::Ready(handle) => handle.as_ref(),
            MotionLoadState::Pending(_) => None,
        }
    }

    /// The outer option is pending; the inner option is motion availability.
    pub(super) fn poll(&mut self, asset_server: &AssetServer) -> Option<Option<Handle<Gltf>>> {
        if let MotionLoadState::Pending(task) = &mut self.state {
            self.state = MotionLoadState::Ready(future::block_on(future::poll_once(task))?);
        }
        match &self.state {
            MotionLoadState::Ready(Some(handle)) => match asset_server.load_state(handle.id()) {
                LoadState::Loaded => Some(Some(handle.clone())),
                LoadState::Failed(_) => Some(None),
                LoadState::NotLoaded | LoadState::Loading => None,
            },
            MotionLoadState::Ready(None) => Some(None),
            MotionLoadState::Pending(_) => None,
        }
    }

    /// Associate revisions by the actual source/path/label even when the first
    /// request failed before Bevy could return a typed handle. The AssetServer
    /// remains the handle authority; observing a revision does not issue a retry.
    pub(in crate::animation) fn observe_source_revision(
        &mut self,
        asset_server: &AssetServer,
        id: bevy::asset::AssetId<Gltf>,
    ) -> bool {
        let requested = animation_asset_path(&self.path);
        if !self.source().is_some_and(|source| source.id() == id)
            && !asset_server
                .get_path(id)
                .is_some_and(|path| path == requested)
        {
            return false;
        }
        if let Some(source) = asset_server.get_handle::<Gltf>(requested) {
            self.state = MotionLoadState::Ready(Some(source));
        }
        true
    }
}

impl MotionClipAdmission {
    /// Admit the complete single-motion clip before publishing any availability.
    pub(super) fn admit(
        key: &MotionKey,
        source: Option<&MotionSource>,
        gltf: &Gltf,
        asset_server: &AssetServer,
        clips: &Assets<AnimationClip>,
        canonical_targets: &HashSet<AnimationTargetId>,
    ) -> MotionClipAdmission {
        let Some(clip_handle) = sole_animation(&gltf.animations) else {
            warn!(
                pack = %key.pack,
                motion = %key.motion,
                count = gltf.animations.len(),
                "Motion must contain exactly one animation"
            );
            return MotionClipAdmission::Unavailable;
        };
        match asset_server.load_state(clip_handle.id()) {
            LoadState::Failed(_) => return MotionClipAdmission::Unavailable,
            LoadState::NotLoaded | LoadState::Loading => return MotionClipAdmission::Pending,
            LoadState::Loaded => {}
        }
        let Some(clip) = clips.get(clip_handle) else {
            return MotionClipAdmission::Pending;
        };
        if canonical_targets.is_empty() {
            return MotionClipAdmission::Pending;
        }
        if !clip_targets_match_base(clip, canonical_targets) {
            warn!(
                pack = %key.pack,
                motion = %key.motion,
                "Motion targets are incompatible with the base rig"
            );
            return MotionClipAdmission::Unavailable;
        }
        let Some(source) = source else {
            return MotionClipAdmission::Unavailable;
        };
        let duration_seconds = match ClipDuration::from_seconds(clip.duration()) {
            Ok(duration) => duration,
            Err(error) => {
                warn!(%error, motion = %key.motion, "Rejected animation duration");
                return MotionClipAdmission::Unavailable;
            }
        };
        if !frame_fits_clip(source.required_last_frame, duration_seconds) {
            warn!(
                pack = %key.pack,
                motion = %key.motion,
                last_frame = u16::from(source.required_last_frame),
                duration = clip.duration(),
                "Motion is shorter than its catalog frames"
            );
            return MotionClipAdmission::Unavailable;
        }
        MotionClipAdmission::Ready(LoadedClip {
            handle: clip_handle.clone(),
            duration_seconds,
            layer: ClipLayer::Whole,
        })
    }
}

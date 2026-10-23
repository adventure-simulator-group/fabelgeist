//! Native asset changes and load failures entering revision invalidation.

use super::AnimationRuntime;
use bevy::{
    asset::{Asset, AssetEvent, AssetId, AssetLoadFailedEvent},
    ecs::system::SystemParam,
    gltf::Gltf,
    prelude::*,
};
use std::collections::HashSet;

#[derive(SystemParam)]
pub(in crate::animation) struct RevisionEvents<'w, 's> {
    clip_assets: Res<'w, Assets<AnimationClip>>,
    source_assets: Res<'w, Assets<Gltf>>,
    clips: MessageReader<'w, 's, AssetEvent<AnimationClip>>,
    sources: MessageReader<'w, 's, AssetEvent<Gltf>>,
    clip_failures: MessageReader<'w, 's, AssetLoadFailedEvent<AnimationClip>>,
    source_failures: MessageReader<'w, 's, AssetLoadFailedEvent<Gltf>>,
}

pub(super) struct ChangedAnimationAssets {
    pub(super) clips: HashSet<AssetId<AnimationClip>>,
    pub(super) sources: HashSet<AssetId<Gltf>>,
}

impl RevisionEvents<'_, '_> {
    pub(super) fn read(&mut self, runtime: &AnimationRuntime) -> ChangedAnimationAssets {
        let mut changed = ChangedAnimationAssets {
            clips: self
                .clips
                .read()
                .map(asset_event_id)
                .chain(self.clip_failures.read().map(|failure| failure.id))
                .collect(),
            sources: self
                .sources
                .read()
                .map(asset_event_id)
                .chain(self.source_failures.read().map(|failure| failure.id))
                .collect(),
        };
        // Native contents are committed in PreUpdate, before their ordinary
        // Added/Modified/Removed messages are flushed in PostUpdate.
        if self.clip_assets.is_changed() {
            changed.clips.extend(self.clip_assets.ids());
            changed.clips.extend(
                runtime
                    .clips
                    .values()
                    .chain(runtime.grips.values())
                    .map(|clip| clip.handle.id()),
            );
        }
        if self.source_assets.is_changed() {
            changed.sources.extend(self.source_assets.ids());
            changed.sources.extend(
                runtime
                    .requested_motions
                    .values()
                    .filter_map(|load| load.source())
                    .map(|source| source.id())
                    .chain(runtime.requested_grips.values().map(|source| source.id())),
            );
        }
        changed
    }
}

fn asset_event_id<A: Asset>(event: &AssetEvent<A>) -> AssetId<A> {
    match event {
        AssetEvent::Added { id }
        | AssetEvent::Modified { id }
        | AssetEvent::Removed { id }
        | AssetEvent::Unused { id }
        | AssetEvent::LoadedWithDependencies { id } => *id,
    }
}

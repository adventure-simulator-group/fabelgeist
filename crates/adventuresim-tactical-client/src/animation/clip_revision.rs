//! Asset revision invalidation before loaded availability and pose sampling.

use super::*;
use bevy::asset::AssetEvent;
use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub(super) struct RevisionCaches<'w, 's> {
    bank: ResMut<'w, pose_buffer::BakedClipBank>,
    strides: ResMut<'w, AuthoredLocomotionStrides>,
    rigs: Query<'w, 's, &'static mut pose_buffer::PoseBufferRig>,
    characters: Query<'w, 's, &'static mut pose_buffer::CharacterLocomotionStrides>,
}

pub(super) fn refresh_animation_revisions(
    mut clip_events: MessageReader<AssetEvent<AnimationClip>>,
    mut source_events: MessageReader<AssetEvent<Gltf>>,
    gltfs: Res<Assets<Gltf>>,
    catalog: Res<AnimationPackCatalog>,
    mut runtime: ResMut<AnimationRuntime>,
    mut caches: RevisionCaches,
) {
    let mut changed = clip_events
        .read()
        .map(|event| match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::Removed { id }
            | AssetEvent::Unused { id }
            | AssetEvent::LoadedWithDependencies { id } => *id,
        })
        .collect::<HashSet<_>>();
    let sources = source_events
        .read()
        .map(|event| match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::Removed { id }
            | AssetEvent::Unused { id }
            | AssetEvent::LoadedWithDependencies { id } => *id,
        })
        .collect::<HashSet<_>>();
    if changed.is_empty() && sources.is_empty() {
        return;
    }
    let motions = runtime
        .requested_motions
        .iter()
        .filter(|(key, load)| {
            let source = load.source();
            source.is_some_and(|source| sources.contains(&source.id()))
                || runtime
                    .clips
                    .get(*key)
                    .is_some_and(|clip| changed.contains(&clip.handle.id()))
                || source
                    .and_then(|source| gltfs.get(source))
                    .is_some_and(|gltf| {
                        gltf.animations
                            .iter()
                            .any(|clip| changed.contains(&clip.id()))
                    })
        })
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let grips = runtime
        .requested_grips
        .iter()
        .filter(|(grip, source)| {
            sources.contains(&source.id())
                || runtime
                    .grips
                    .get(*grip)
                    .is_some_and(|clip| changed.contains(&clip.handle.id()))
                || gltfs.get(*source).is_some_and(|gltf| {
                    gltf.animations
                        .iter()
                        .any(|clip| changed.contains(&clip.id()))
                })
        })
        .map(|(&grip, _)| grip)
        .collect::<Vec<_>>();
    for key in &motions {
        if let Some(clip) = runtime.clips.remove(key) {
            changed.insert(clip.handle.id());
        }
        runtime.unavailable_motions.remove(key);
    }
    for grip in grips {
        if let Some(clip) = runtime.grips.remove(&grip) {
            changed.insert(clip.handle.id());
        }
        runtime.processed_grips.remove(&grip);
        runtime.unavailable_grips.remove(&grip);
    }
    caches.bank.invalidate_assets(&changed);
    caches.strides.invalidate_assets(&changed);
    for mut character in &mut caches.characters {
        character.measurements.invalidate_assets(&changed);
    }
    for mut rig in &mut caches.rigs {
        rig.invalidate_clip_revisions(&changed);
    }
    if !motions.is_empty() {
        runtime.rebuild_loaded_library(&catalog);
    }
}

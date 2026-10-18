use super::*;

pub(super) fn install_requests(app: &mut App) {
    app.add_systems(Startup, request_animation_packs);
    #[cfg(target_family = "wasm")]
    app.add_systems(
        Update,
        request_animation_packs
            .run_if(resource_changed::<crate::browser_runtime::BrowserMode>)
            .before(collect_loaded_packs),
    );
}

pub(super) fn request_animation_packs(
    catalog: Res<AnimationPackCatalog>,
    asset_server: Res<AssetServer>,
    mut runtime: ResMut<AnimationRuntime>,
    #[cfg(target_family = "wasm")] mode: Res<crate::browser_runtime::BrowserMode>,
) {
    if runtime.requested_base.is_none() {
        runtime.requested_base = Some(asset_server.load(animation_asset_path(&BIPED_BASE_GLB)));
    }
    for grip in WeaponGrip::ALL {
        runtime
            .requested_grips
            .entry(grip)
            .or_insert_with(|| asset_server.load(animation_asset_path(grip.path())));
    }
    for (pack_id, pack) in &catalog.packs {
        for (motion_id, source) in &pack.motions {
            // Strategic portraits and street characters stand in relaxed idle.
            // Combat and locomotion clips join the persistent asset cache when
            // the player enters tactical play.
            #[cfg(target_family = "wasm")]
            if *mode == crate::browser_runtime::BrowserMode::Strategic
                && !pack
                    .poses
                    .get(&SemanticPose::IdleRelaxed)
                    .is_some_and(|anchor| anchor.motion == *motion_id)
            {
                continue;
            }
            runtime
                .requested_motions
                .entry(MotionKey::new(pack_id, motion_id))
                .or_insert_with(|| asset_server.load(animation_asset_path(&source.path)));
        }
    }
    if let Err(error) = runtime.library.validate_structure() {
        error!(
            ?error,
            "Rejected structurally invalid animation pack library"
        );
    }
}

pub(super) fn collect_loaded_packs(
    catalog: Res<AnimationPackCatalog>,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut runtime: ResMut<AnimationRuntime>,
) {
    let mut changed = false;
    if runtime.base_load == BaseRigLoad::Pending {
        let Some(handle) = runtime.requested_base.as_ref() else {
            return;
        };
        if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
            runtime.base_load = BaseRigLoad::Failed;
            warn!(path = %BIPED_BASE_GLB, "Authored base rig is unavailable");
        } else if let Some(gltf) = gltfs.get(handle) {
            runtime.base_scene = gltf
                .default_scene
                .clone()
                .or_else(|| gltf.scenes.first().cloned());
            if runtime.base_scene.is_none() {
                runtime.base_load = BaseRigLoad::Failed;
                warn!(path = %BIPED_BASE_GLB, "Authored base rig has no scene");
            } else {
                runtime.base_load = BaseRigLoad::Ready;
            }
        }
    }

    let requested = runtime
        .requested_motions
        .iter()
        .map(|(key, handle)| (key.clone(), handle.clone()))
        .collect::<Vec<_>>();
    for (key, handle) in requested {
        if runtime.processed_motions.contains(&key) || runtime.unavailable_motions.contains(&key) {
            continue;
        }
        if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
            runtime.unavailable_motions.insert(key.clone());
            changed = true;
            continue;
        }
        let Some(gltf) = gltfs.get(&handle) else {
            continue;
        };
        let Some(clip_handle) = sole_animation(&gltf.animations) else {
            warn!(
                pack = %key.pack,
                motion = %key.motion,
                count = gltf.animations.len(),
                "Motion must contain exactly one animation"
            );
            runtime.unavailable_motions.insert(key.clone());
            changed = true;
            continue;
        };
        let Some(clip) = clips.get(clip_handle) else {
            continue;
        };
        if runtime.canonical_targets.is_empty() {
            continue;
        }
        if !clip_targets_match_base(clip, &runtime.canonical_targets) {
            warn!(
                pack = %key.pack,
                motion = %key.motion,
                "Motion targets are incompatible with the base rig"
            );
            runtime.unavailable_motions.insert(key.clone());
            changed = true;
            continue;
        }
        let Some(source) = catalog
            .packs
            .get(&key.pack)
            .and_then(|pack| pack.motions.get(&key.motion))
        else {
            runtime.unavailable_motions.insert(key.clone());
            changed = true;
            continue;
        };
        if !frame_fits_clip(source.required_last_frame, clip.duration()) {
            warn!(
                pack = %key.pack,
                motion = %key.motion,
                last_frame = source.required_last_frame,
                duration = clip.duration(),
                "Motion is shorter than its catalog frames"
            );
            runtime.unavailable_motions.insert(key.clone());
            changed = true;
            continue;
        }
        runtime
            .clip_handles
            .insert(key.clone(), clip_handle.clone());
        runtime.processed_motions.insert(key);
        changed = true;
    }

    if !changed {
        collect_loaded_grips(&asset_server, &gltfs, &clips, &mut runtime);
        return;
    }
    runtime.library = AnimationPackLibrary::default();
    for (pack_id, pack) in &catalog.packs {
        let available = pack
            .poses
            .iter()
            .filter(|(_, anchor)| {
                let key = MotionKey::new(pack_id, &anchor.motion);
                runtime.processed_motions.contains(&key)
                    && runtime
                        .clip_handles
                        .get(&key)
                        .and_then(|handle| clips.get(handle))
                        .is_some_and(|clip| frame_fits_clip(anchor.frame, clip.duration()))
            })
            .map(|(&pose, _)| pose)
            .collect();
        if let Err(error) = runtime.library.insert(AnimationPack {
            id: pack_id.clone(),
            skeleton_family: pack.skeleton_family.clone(),
            fallback: pack.fallback.clone(),
            clips: available,
        }) {
            error!(?error, pack = %pack_id, "Rejected animation pack");
        }
    }
    runtime.clips = runtime
        .clip_handles
        .iter()
        .map(|(key, handle)| (key.clone(), handle.clone()))
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(key, handle)| {
            let duration_seconds = clips.get(&handle).map_or(0.0, AnimationClip::duration);
            (
                key,
                LoadedClip {
                    handle,
                    duration_seconds: ClipDuration::from_seconds(duration_seconds),
                    layer: ClipLayer::Whole,
                },
            )
        })
        .collect();
    collect_loaded_grips(&asset_server, &gltfs, &clips, &mut runtime);
}

pub(super) fn sole_animation(
    animations: &[Handle<AnimationClip>],
) -> Option<&Handle<AnimationClip>> {
    (animations.len() == 1).then(|| &animations[0])
}

pub(super) fn frame_fits_clip(frame: u16, duration: f32) -> bool {
    frame_seconds(frame) <= duration + 0.5 / animation_frames_per_second()
}

pub(super) fn frame_seconds(frame: u16) -> f32 {
    frame as f32 / animation_frames_per_second()
}

pub(super) fn clip_targets_match_base(
    clip: &AnimationClip,
    canonical_targets: &HashSet<AnimationTargetId>,
) -> bool {
    targets_match_base(clip.curves().keys(), canonical_targets)
}

pub(super) fn targets_match_base<'a>(
    mut targets: impl Iterator<Item = &'a AnimationTargetId>,
    canonical_targets: &HashSet<AnimationTargetId>,
) -> bool {
    let Some(first) = targets.next() else {
        return false;
    };
    canonical_targets.contains(first) && targets.all(|target| canonical_targets.contains(target))
}

pub(super) fn attach_loaded_rig_scenes(
    mut commands: Commands,
    runtime: Res<AnimationRuntime>,
    players: Query<(Entity, &SkeletonState, Has<AnimationRigAttached>), AnimatedActors>,
) {
    for (player, _skeleton, attached) in &players {
        if attached {
            continue;
        }
        let Some(scene) = runtime.base_scene.as_ref() else {
            continue;
        };
        commands.entity(player).with_children(|parent| {
            parent.spawn((
                Name::new("Authored animation rig"),
                AnimationRigScene(player),
                WorldAssetRoot(scene.clone()),
                Transform::from_xyz(0.0, player_visual_y_offset_metres(), 0.0),
                Visibility::Hidden,
            ));
        });
        commands
            .entity(player)
            .insert((AnimationPlayback::default(), AnimationRigAttached));
    }
}

/// A zero-animation base glTF receives none of Bevy's usual animation wiring.
/// Recreate the loader's stable name-path target IDs so independently exported
/// motion files can animate the canonical `Skeleton` hierarchy.
pub(super) fn establish_animation_targets(
    mut commands: Commands,
    mut runtime: ResMut<AnimationRuntime>,
    roots: Query<(Entity, &AnimationRigScene), Without<RigAnimationTargetsBound>>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    for (rig_root, _owner) in &roots {
        let Some(skeleton_root) = find_named_descendant(rig_root, "Skeleton", &children, &names)
        else {
            continue;
        };
        bind_animation_target_paths(
            &mut commands,
            skeleton_root,
            Vec::new(),
            &children,
            &names,
            &mut runtime,
        );
        commands.entity(rig_root).insert(RigAnimationTargetsBound);
    }
}

pub(super) fn descendants_including(root: Entity, children: &Query<&Children>) -> Vec<Entity> {
    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(entity) = pending.pop() {
        found.push(entity);
        if let Ok(entity_children) = children.get(entity) {
            pending.extend(entity_children.iter());
        }
    }
    found
}

pub(super) fn find_named_descendant(
    root: Entity,
    target: &str,
    children: &Query<&Children>,
    names: &Query<&Name>,
) -> Option<Entity> {
    descendants_including(root, children)
        .into_iter()
        .find(|entity| names.get(*entity).is_ok_and(|name| name.as_str() == target))
}

pub(super) fn bind_animation_target_paths(
    commands: &mut Commands,
    entity: Entity,
    mut path: Vec<Name>,
    children: &Query<&Children>,
    names: &Query<&Name>,
    runtime: &mut AnimationRuntime,
) {
    let Ok(name) = names.get(entity) else {
        return;
    };
    path.push(name.clone());
    let target = AnimationTargetId::from_names(path.iter());
    runtime.canonical_targets.insert(target);
    // `Name` is the native scene boundary; downstream rig consumers retain
    // this admitted identity and its cached animation classification.
    commands
        .entity(entity)
        .insert((target, AuthoredRigJoint::new(name.as_str().into())));
    if let Ok(entity_children) = children.get(entity) {
        for child in entity_children.iter() {
            bind_animation_target_paths(commands, child, path.clone(), children, names, runtime);
        }
    }
}

fn collect_loaded_grips(
    asset_server: &AssetServer,
    gltfs: &Assets<Gltf>,
    clips: &Assets<AnimationClip>,
    runtime: &mut AnimationRuntime,
) {
    let requested = runtime
        .requested_grips
        .iter()
        .map(|(&grip, handle)| (grip, handle.clone()))
        .collect::<Vec<_>>();
    for (grip, handle) in requested {
        if runtime.processed_grips.contains(&grip) || runtime.unavailable_grips.contains(&grip) {
            continue;
        }
        if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
            runtime.unavailable_grips.insert(grip);
            warn!(path = %grip.path(), "Authored weapon grip is unavailable");
            continue;
        }
        let Some(gltf) = gltfs.get(&handle) else {
            continue;
        };
        let Some(clip_handle) = sole_animation(&gltf.animations) else {
            warn!(
                path = %grip.path(),
                count = gltf.animations.len(),
                "Weapon grip must contain exactly one animation"
            );
            runtime.unavailable_grips.insert(grip);
            continue;
        };
        let Some(clip) = clips.get(clip_handle) else {
            continue;
        };
        if runtime.canonical_targets.is_empty() {
            continue;
        }
        if !clip_targets_match_base(clip, &runtime.canonical_targets) {
            warn!(
                path = %grip.path(),
                "Weapon grip targets are incompatible with the base rig"
            );
            runtime.unavailable_grips.insert(grip);
            continue;
        }
        runtime.grips.insert(
            grip,
            LoadedClip {
                handle: clip_handle.clone(),
                duration_seconds: ClipDuration::from_seconds(clip.duration()),
                layer: ClipLayer::MainHand,
            },
        );
        runtime.processed_grips.insert(grip);
    }
}

fn animation_asset_path(path: &AnimationAssetPath) -> String {
    #[cfg(not(target_family = "wasm"))]
    {
        format!("workspace://{path}")
    }
    #[cfg(target_family = "wasm")]
    {
        path.relative().to_owned()
    }
}

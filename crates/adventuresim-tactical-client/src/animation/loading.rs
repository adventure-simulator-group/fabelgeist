use super::*;
use bevy::asset::AssetPath;
use fabelgeist_rig::RigJointName;
pub(super) use motion::MotionLoad;

mod motion;

impl AnimationRuntime {
    pub(super) fn rebuild_loaded_library(&mut self, catalog: &AnimationPackCatalog) {
        self.library = AnimationPackLibrary::default();
        for (pack_id, pack) in &catalog.packs {
            let available = pack
                .poses
                .iter()
                .filter(|(_, anchor)| {
                    self.clips
                        .get(&MotionKey::new(pack_id, &anchor.motion))
                        .is_some_and(|clip| frame_fits_clip(anchor.frame, clip.duration_seconds))
                })
                .map(|(&pose, _)| pose)
                .collect();
            if let Err(error) = self.library.insert(AnimationPack {
                id: pack_id.clone(),
                skeleton_family: pack.skeleton_family.clone(),
                fallback: pack.fallback.clone(),
                clips: available,
            }) {
                error!(?error, pack = %pack_id, "Rejected animation pack");
            }
        }
    }
}

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
    if matches!(runtime.base_rig, BaseRigLoad::Unrequested) {
        runtime.base_rig = BaseRigLoad::Loading {
            source: asset_server.load(animation_asset_path(&BIPED_BASE_GLB)),
        };
    }
    for grip in WeaponGrip::ALL {
        runtime
            .requested_grips
            .entry(grip)
            .or_insert_with(|| asset_server.load(animation_asset_path(&grip.path())));
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
                .or_insert_with(|| MotionLoad::request(&asset_server, &source.path));
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
    if let BaseRigLoad::Loading { source } = &runtime.base_rig {
        let source = source.clone();
        if matches!(asset_server.load_state(source.id()), LoadState::Failed(_)) {
            runtime.base_rig = BaseRigLoad::Failed { source };
            warn!(path = %BIPED_BASE_GLB, "Authored base rig is unavailable");
        } else if let Some(gltf) = gltfs.get(&source) {
            if let Some(scene) = gltf
                .default_scene
                .clone()
                .or_else(|| gltf.scenes.first().cloned())
            {
                runtime.base_rig = BaseRigLoad::Ready { source, scene };
            } else {
                runtime.base_rig = BaseRigLoad::Failed { source };
                warn!(path = %BIPED_BASE_GLB, "Authored base rig has no scene");
            }
        }
    }

    let requested = runtime
        .requested_motions
        .iter_mut()
        .filter_map(|(key, load)| load.poll(&asset_server).map(|handle| (key.clone(), handle)))
        .collect::<Vec<_>>();
    for (key, handle) in requested {
        if runtime.clips.contains_key(&key) || runtime.unavailable_motions.contains(&key) {
            continue;
        }
        let Some(handle) = handle else {
            runtime.unavailable_motions.insert(key.clone());
            changed = true;
            continue;
        };
        let Some(gltf) = gltfs.get(&handle) else {
            continue;
        };
        let source = catalog
            .packs
            .get(&key.pack)
            .and_then(|pack| pack.motions.get(&key.motion));
        match motion::MotionClipAdmission::admit(
            &key,
            source,
            gltf,
            &asset_server,
            &clips,
            &runtime.canonical_targets,
        ) {
            motion::MotionClipAdmission::Pending => continue,
            motion::MotionClipAdmission::Unavailable => {
                runtime.unavailable_motions.insert(key);
            }
            motion::MotionClipAdmission::Ready(clip) => {
                runtime.clips.insert(key, clip);
            }
        }
        changed = true;
    }

    if !changed {
        collect_loaded_grips(&asset_server, &gltfs, &clips, &mut runtime);
        return;
    }
    runtime.rebuild_loaded_library(&catalog);
    collect_loaded_grips(&asset_server, &gltfs, &clips, &mut runtime);
}

pub(super) fn sole_animation(
    animations: &[Handle<AnimationClip>],
) -> Option<&Handle<AnimationClip>> {
    (animations.len() == 1).then(|| &animations[0])
}

pub(super) fn frame_fits_clip(frame: AuthoredAnimationFrame, duration: ClipDuration) -> bool {
    frame_seconds(frame).is_ok_and(|time| {
        time.seconds() <= duration.seconds() + 0.5 / animation_frames_per_second()
    })
}

pub(super) fn frame_seconds(frame: AuthoredAnimationFrame) -> AnimationValueResult<ClipTime> {
    // Catalog frames enter the engine's seconds coordinate here.
    ClipTime::from_seconds(f32::from(u16::from(frame)) / animation_frames_per_second())
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
        let Some(scene) = runtime.base_rig.scene() else {
            continue;
        };
        commands.entity(player).with_children(|parent| {
            parent.spawn((
                Name::new("Authored animation rig"),
                AnimationRigScene(player),
                WorldAssetRoot(scene.clone()),
                Transform::from_xyz(0.0, player_visual_y_offset().metres(), 0.0),
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
        let Some(skeleton_root) =
            find_named_descendant(rig_root, RigJointName::SKELETON.as_ref(), &children, &names)
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
        match asset_server.load_state(handle.id()) {
            LoadState::Failed(_) => {
                runtime.unavailable_grips.insert(grip);
                warn!(path = %grip.path(), "Authored weapon grip is unavailable");
                continue;
            }
            LoadState::NotLoaded | LoadState::Loading => continue,
            LoadState::Loaded => {}
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
        match asset_server.load_state(clip_handle.id()) {
            LoadState::Failed(_) => {
                runtime.unavailable_grips.insert(grip);
                continue;
            }
            LoadState::NotLoaded | LoadState::Loading => continue,
            LoadState::Loaded => {}
        }
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
        let duration_seconds = match ClipDuration::from_seconds(clip.duration()) {
            Ok(duration) => duration,
            Err(error) => {
                warn!(%error, path = %grip.path(), "Rejected grip duration");
                runtime.unavailable_grips.insert(grip);
                continue;
            }
        };
        runtime.grips.insert(
            grip,
            LoadedClip {
                handle: clip_handle.clone(),
                duration_seconds,
                layer: ClipLayer::MainHand,
            },
        );
        runtime.processed_grips.insert(grip);
    }
}

fn animation_asset_path(path: &AnimationAssetPath) -> AssetPath<'static> {
    #[cfg(not(target_family = "wasm"))]
    {
        AssetPath::from(path.as_ref().to_owned()).with_source(WORKSPACE_ASSET_SOURCE)
    }
    #[cfg(target_family = "wasm")]
    {
        AssetPath::from(path.as_ref().to_owned())
    }
}

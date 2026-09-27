//! One census per resident scene, outside steady rendering and navigation.
use super::{RetainedScene, cached_views::CachedViews, status::SceneAssetsReady};
use adventuresim_tactical_core::prelude::{SceneDoor, SceneFurniture, SceneWindow};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashSet};

const MAX_CENSUS_GROUPS: usize = 20;

#[derive(Default, serde::Serialize)]
struct Counts {
    instances: usize,
    visible: usize,
    visible_cuboids: usize,
    #[serde(skip)]
    meshes: HashSet<AssetId<Mesh>>,
    unique_meshes: usize,
}

pub(super) fn census(
    scene: Res<RetainedScene>,
    ready: Res<SceneAssetsReady>,
    cache: Res<CachedViews>,
    mut recorded: Local<Option<Entity>>,
    meshes: Res<Assets<Mesh>>,
    parts: Query<(Entity, &Mesh3d, Option<&ViewVisibility>)>,
    hierarchy: Query<(
        Option<&Name>,
        Option<&ChildOf>,
        Has<SceneDoor>,
        Has<SceneWindow>,
        Has<SceneFurniture>,
        Has<crate::presentation::PresentedSign>,
    )>,
) {
    if !ready.0 || !cache.is_ready() || scene.root.is_none() || scene.root == *recorded {
        return;
    }
    *recorded = scene.root;
    let mut groups: BTreeMap<String, Counts> = BTreeMap::new();
    for (entity, mesh, visible) in &parts {
        let mut cursor = entity;
        let label = loop {
            let Ok((name, parent, door, window, furniture, sign)) = hierarchy.get(cursor) else {
                break "unnamed";
            };
            if door {
                break "operable doors";
            }
            if window {
                break "operable windows";
            }
            if furniture {
                break "furniture";
            }
            if sign {
                break "shop signs";
            }
            if let Some(name) = name {
                break name.as_str();
            }
            let Some(parent) = parent else {
                break "unnamed";
            };
            cursor = parent.parent();
        };
        let count = groups.entry(label.to_owned()).or_default();
        count.instances += 1;
        count.meshes.insert(mesh.id());
        if visible.is_some_and(|visible| visible.get()) {
            count.visible += 1;
            count.visible_cuboids +=
                usize::from(meshes.get(mesh).is_some_and(|mesh| {
                    mesh.try_indices().is_ok_and(|indices| indices.len() == 36)
                }));
        }
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_by_key(|(_, count)| std::cmp::Reverse(count.visible));
    for (_, count) in &mut groups {
        count.unique_meshes = count.meshes.len();
    }
    // Bounded telemetry, not a frame-by-frame traversal or diagnostic overlay.
    for (name, count) in groups.into_iter().take(MAX_CENSUS_GROUPS) {
        info!(
            "City mesh census {name}: {}",
            serde_json::to_string(&count).unwrap()
        );
    }
}

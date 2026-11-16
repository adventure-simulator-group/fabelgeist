//! Behaviour-tree files that drive server-controlled combatants.
//!
//! Trees are RON files in `content/tactical/ai/`, named by enemy fixtures
//! (`behavior_tree: always_dodge.bt.ron`). The server parses and validates
//! every tree it can spawn at startup, so spawning only looks trees up and a
//! broken file stops the server before the mission begins.

use std::{collections::HashMap, path::PathBuf};

use bevy::reflect::TypeRegistryArc;
use kisya_edbt::core::tree::BehaviourTree;
use serde::de::DeserializeSeed;

use super::*;

/// Tree for ordinary enemies when no fixture names one.
pub(crate) const STANDARD_COMBAT_TREE: &str = "standard_combat.bt.ron";
/// Tree for enemies that should stay inert (`--enemy-combat-scale-bps 0`).
pub(crate) const PASSIVE_TREE: &str = "passive.bt.ron";
const TREE_DIRECTORY: &str = "content/tactical/ai";
const TREE_EXTENSION: &str = "bt.ron";

/// Parsed trees by file name.
#[derive(Resource, Default)]
pub(crate) struct BotTrees(HashMap<String, Handle<BehaviourTree>>);

#[derive(Debug, thiserror::Error)]
pub enum BotTreeError {
    #[error("behaviour tree {file} was not loaded at startup")]
    NotLoaded { file: String },
    #[error("could not read behaviour tree {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("behaviour tree {file} is not valid RON: {source}")]
    Syntax {
        file: String,
        source: Box<ron::error::SpannedError>,
    },
    #[error("behaviour tree {file} does not deserialize: {source}")]
    Deserialize {
        file: String,
        source: Box<ron::error::Error>,
    },
    #[error("behaviour tree {file} has invalid nodes:\n{diagnostics}")]
    Validation { file: String, diagnostics: String },
}

impl BotTrees {
    /// The `Behaviour` that runs a tree loaded at startup.
    pub(crate) fn behaviour(&self, file: &str) -> Result<Behaviour, BotTreeError> {
        self.0
            .get(file)
            .map(|tree| Behaviour { tree: tree.clone() })
            .ok_or_else(|| BotTreeError::NotLoaded {
                file: file.to_owned(),
            })
    }

    fn load(
        &mut self,
        file: &str,
        assets: &mut Assets<BehaviourTree>,
        registry: &TypeRegistryArc,
    ) -> Result<Behaviour, BotTreeError> {
        if !self.0.contains_key(file) {
            let tree = assets.add(parse_tree(file, registry)?);
            self.0.insert(file.to_owned(), tree);
        }
        self.behaviour(file)
    }
}

/// Loads the fallback trees and every tree the enemy fixture names.
pub(super) fn preload_bot_trees(world: &mut World) -> Result {
    let fixture_trees: Vec<String> = world
        .get_resource::<crate::Args>()
        .and_then(|args| args.enemy_fixture.as_ref())
        .map(|fixture| {
            fixture
                .enemies()
                .iter()
                .map(|enemy| enemy.behavior_tree.as_str().to_owned())
                .collect()
        })
        .unwrap_or_default();
    for file in [STANDARD_COMBAT_TREE, PASSIVE_TREE]
        .into_iter()
        .chain(fixture_trees.iter().map(String::as_str))
    {
        behaviour_in_world(world, file)?;
    }
    Ok(())
}

/// The `Behaviour` that runs `file`, loading the tree if needed.
pub(crate) fn behaviour_in_world(world: &mut World, file: &str) -> Result<Behaviour, BotTreeError> {
    let registry = world.resource::<AppTypeRegistry>().0.clone();
    world.resource_scope(|world, mut trees: Mut<BotTrees>| {
        let mut assets = world.resource_mut::<Assets<BehaviourTree>>();
        trees.load(file, &mut assets, &registry)
    })
}

/// Starts `file` on every bot restored from a world dump. Dumps keep which
/// entities are bots, but not the tree each ran, so restored bots run the tree
/// a fresh spawn would.
#[cfg(feature = "debug")]
pub(crate) fn start_restored_bots(world: &mut World, file: &str) -> Result<(), BotTreeError> {
    let restored: Vec<Entity> = world
        .query_filtered::<Entity, (With<MissionEnemy>, Without<Behaviour>)>()
        .iter(world)
        .collect();
    if restored.is_empty() {
        return Ok(());
    }
    let tree = behaviour_in_world(world, file)?.tree;
    for bot in restored {
        world
            .entity_mut(bot)
            .insert(Behaviour { tree: tree.clone() });
    }
    Ok(())
}

fn parse_tree(file: &str, registry: &TypeRegistryArc) -> Result<BehaviourTree, BotTreeError> {
    let stem = file
        .strip_suffix(TREE_EXTENSION)
        .and_then(|stem| stem.strip_suffix('.'))
        .unwrap_or(file);
    let path = resolve_fixture_path(stem, TREE_DIRECTORY, TREE_EXTENSION);
    let source =
        std::fs::read_to_string(&path).map_err(|source| BotTreeError::Read { path, source })?;
    let mut deserializer =
        ron::de::Deserializer::from_str(&source).map_err(|source| BotTreeError::Syntax {
            file: file.to_owned(),
            source: Box::new(source),
        })?;
    let tree = BehaviourTreeDeserializer::new(registry)
        .deserialize(&mut deserializer)
        .map_err(|source| BotTreeError::Deserialize {
            file: file.to_owned(),
            source: Box::new(source),
        })?;
    let diagnostics = tree.validate(&registry.read());
    if diagnostics.is_empty() {
        return Ok(tree);
    }
    Err(BotTreeError::Validation {
        file: file.to_owned(),
        diagnostics: diagnostics
            .iter()
            .map(|item| format!("{}: {}", item.format_path_one_line(&tree), item.error))
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

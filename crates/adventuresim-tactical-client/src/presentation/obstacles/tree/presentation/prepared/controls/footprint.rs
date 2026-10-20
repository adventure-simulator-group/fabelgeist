//! Complete live asset-slot sets and residency accounting around local controls.
use super::*;
use crate::presentation::obstacles::tree::presentation::{
    TreeContentRequest, grown::GrownCatalogueAssets,
};
use adventuresim_environment::render::{
    LevelSelection,
    grown::{GrownLeafMaterial, GrownWoodMaterial},
};
use bevy::render::storage::ShaderBuffer;
use std::{
    collections::{BTreeSet, HashSet},
    sync::Arc,
};

#[derive(PartialEq)]
pub(super) struct Footprint {
    assets: AssetFootprint,
    roots: BTreeSet<Entity>,
}

#[derive(PartialEq)]
struct AssetFootprint {
    meshes: BTreeSet<AssetId<Mesh>>,
    bark: BTreeSet<AssetId<TacticalTreeBarkMaterial>>,
    aggregate: BTreeSet<AssetId<TacticalTreeAggregateBarkMaterial>>,
    leaves: BTreeSet<AssetId<TacticalTreeLeafCardMaterial>>,
    buds: BTreeSet<AssetId<StandardMaterial>>,
    cards: BTreeSet<AssetId<TacticalTreeImpostorMaterial>>,
    images: BTreeSet<AssetId<Image>>,
    cache: CacheFootprint,
    active_scene: Option<Entity>,
    request: Option<TreeContentRequest>,
    grown_wood: Option<BTreeSet<AssetId<GrownWoodMaterial>>>,
    grown_leaves: Option<BTreeSet<AssetId<GrownLeafMaterial>>>,
    source_buffers: Option<BTreeSet<AssetId<ShaderBuffer>>>,
    residency: serde_json::Value,
}

enum CacheFootprint {
    Unbound,
    Recipe {
        binding: TreeSceneBinding,
        variants: HashSet<Seed>,
    },
    Grown {
        binding: TreeSceneBinding,
        catalogue: Arc<GrownCatalogueAssets>,
        selection: LevelSelection,
    },
}

impl PartialEq for CacheFootprint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unbound, Self::Unbound) => true,
            (
                Self::Recipe {
                    binding: left,
                    variants: left_keys,
                },
                Self::Recipe {
                    binding: right,
                    variants: right_keys,
                },
            ) => left == right && left_keys == right_keys,
            (
                Self::Grown {
                    binding: left,
                    catalogue: left_assets,
                    selection: left_level,
                },
                Self::Grown {
                    binding: right,
                    catalogue: right_assets,
                    selection: right_level,
                },
            ) => {
                left == right && left_level == right_level && Arc::ptr_eq(left_assets, right_assets)
            }
            _ => false,
        }
    }
}

impl CacheFootprint {
    fn capture(cache: &TreePresentationCache) -> Self {
        let Some(active) = &cache.active else {
            return Self::Unbound;
        };
        match &active.contents {
            TreeCacheContents::Recipe(recipe) => Self::Recipe {
                binding: active.binding.clone(),
                variants: recipe.variants.keys().copied().collect(),
            },
            TreeCacheContents::Grown(grown) => Self::Grown {
                binding: active.binding.clone(),
                catalogue: grown.catalogue.clone(),
                selection: grown.selection,
            },
        }
    }
}

impl Footprint {
    pub(super) fn capture(world: &mut World) -> Result<Self> {
        Ok(Self {
            assets: AssetFootprint::capture(world)?,
            roots: world
                .query_filtered::<Entity, With<PresentedTree>>()
                .iter(world)
                .collect(),
        })
    }

    pub(super) fn verify(&self, world: &mut World) -> Result<()> {
        ensure!(
            *self == Self::capture(world)?,
            "root admission control changed assets/roots/accounting"
        );
        Ok(())
    }

    pub(super) fn verify_assets(&self, world: &World) -> Result<()> {
        ensure!(
            self.assets == AssetFootprint::capture(world)?,
            "root attachment inserted assets or doubled residency accounting"
        );
        Ok(())
    }
}

impl AssetFootprint {
    fn capture(world: &World) -> Result<Self> {
        Ok(Self {
            meshes: world.resource::<Assets<Mesh>>().ids().collect(),
            bark: world
                .resource::<Assets<TacticalTreeBarkMaterial>>()
                .ids()
                .collect(),
            aggregate: world
                .resource::<Assets<TacticalTreeAggregateBarkMaterial>>()
                .ids()
                .collect(),
            leaves: world
                .resource::<Assets<TacticalTreeLeafCardMaterial>>()
                .ids()
                .collect(),
            buds: world.resource::<Assets<StandardMaterial>>().ids().collect(),
            cards: world
                .resource::<Assets<TacticalTreeImpostorMaterial>>()
                .ids()
                .collect(),
            images: world.resource::<Assets<Image>>().ids().collect(),
            cache: CacheFootprint::capture(world.resource::<TreePresentationCache>()),
            active_scene: world.resource::<ActiveTacticalScene>().entity,
            request: world
                .resource::<ActiveTacticalScene>()
                .entity
                .and_then(|scene| world.get::<TreeContentRequest>(scene))
                .cloned(),
            grown_wood: world
                .get_resource::<Assets<GrownWoodMaterial>>()
                .map(|assets| assets.ids().collect()),
            grown_leaves: world
                .get_resource::<Assets<GrownLeafMaterial>>()
                .map(|assets| assets.ids().collect()),
            source_buffers: world
                .get_resource::<Assets<ShaderBuffer>>()
                .map(|assets| assets.ids().collect()),
            residency: serde_json::to_value(
                world.resource::<RecipeTreeAssetResidencyDiagnostics>(),
            )?,
        })
    }
}

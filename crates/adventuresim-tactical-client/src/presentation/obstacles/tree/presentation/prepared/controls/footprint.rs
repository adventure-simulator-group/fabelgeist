//! Complete live asset-slot sets and residency accounting around local controls.
use super::*;
use std::collections::{BTreeSet, HashSet};

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
    variants: HashSet<Seed>,
    residency: serde_json::Value,
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
            variants: world
                .resource::<TreePresentationCache>()
                .variants
                .keys()
                .copied()
                .collect(),
            residency: serde_json::to_value(world.resource::<TreeAssetResidencyDiagnostics>())?,
        })
    }
}

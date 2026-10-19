//! Complete admitted assets use the production cache, materials and streamer.
use super::super::prepared_cache::{PreparedCluster, PreparedTreeCache, PreparedVariant};
use super::*;
use anyhow::{Context, Result, ensure};
use bevy::ecs::system::{SystemParam, SystemState};
use std::sync::Arc;

pub(super) use assets::require_resident;
use assets::{AssetStores, VariantAdmission};
pub(crate) use controls::RootAdmissionReport;
pub(crate) use dependencies::material_image_ids;
pub(crate) use inspection::TreeCacheInspection;
pub(crate) use roots::TreeRootBatch;
pub(crate) use selection::TreeCacheRepresentation;

mod assets;
mod controls;
mod dependencies;
mod inspection;
mod installation;
mod roots;
mod selection;

const COMPLETE_STREAM_MASK: u8 = 0b11_1111;

pub(crate) struct InstalledTreeSpecimen {
    root: Entity,
    variant: Arc<InstalledTreeVariant>,
}

/// Scene-bound reference to the complete variant owned by the production cache.
pub(crate) struct InstalledTreeVariant {
    scene: Entity,
    environment: SceneEnvironment,
    species: TreePresentationSpecies,
    cache_key: Seed,
    origin: TreeAssetOrigin,
    parts: Vec<InstalledTreePart>,
    images: std::collections::HashSet<AssetId<Image>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InstalledTreePart {
    pub(crate) role: InstalledTreePartRole,
    pub(crate) mesh: AssetId<Mesh>,
    pub(crate) material: InstalledTreeMaterial,
}

enum VariantSource {
    Generated(super::super::authoring::specimens::SourceSpecimen),
    Prepared(Box<PreparedVariant>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InstalledTreePartRole {
    DetailedTrunk,
    MiddleTrunk,
    Wood { primary_group: u8 },
    Buds { primary_group: u8 },
    CamberedLeaves { primary_group: u8 },
    FlatLeaves { primary_group: u8 },
    AggregateWood { lod: super::super::AggregateTreeLod },
    CanopyCard { lod: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum InstalledTreeMaterial {
    Bark(AssetId<TacticalTreeBarkMaterial>),
    Aggregate(AssetId<TacticalTreeAggregateBarkMaterial>),
    Leaves(AssetId<TacticalTreeLeafCardMaterial>),
    Buds(AssetId<StandardMaterial>),
    Card {
        material: AssetId<TacticalTreeImpostorMaterial>,
        image: AssetId<Image>,
    },
}

impl PreparedTreeCache {
    pub(crate) fn spawn(
        self,
        world: &mut World,
        roots: TreeRootBatch,
    ) -> Result<Vec<InstalledTreeSpecimen>> {
        roots.verify(world)?;
        let variant = self.into_variant(&roots.destination.environment)?;
        InstalledTreeVariant::install(world, roots, VariantSource::Prepared(Box::new(variant)))
    }
}

impl InstalledTreeSpecimen {
    pub(crate) fn root(&self) -> Entity {
        self.root
    }

    pub(crate) fn variant(&self) -> &Arc<InstalledTreeVariant> {
        &self.variant
    }

    /// Diagnostic prewarming uses the production generated cache owner.
    pub(crate) fn from_generated(
        world: &mut World,
        source: fabelgeist_botany::identity::PlantId,
        roots: TreeRootBatch,
    ) -> Result<Vec<Self>> {
        roots.verify(world)?;
        let specimen = super::super::authoring::specimens::SourceSpecimen::find(source)?;
        InstalledTreeVariant::install(world, roots, VariantSource::Generated(specimen))
    }
}

impl InstalledTreeVariant {
    pub(crate) fn cache_key(&self) -> Seed {
        self.cache_key
    }

    pub(crate) fn origin(&self) -> &TreeAssetOrigin {
        &self.origin
    }

    pub(crate) fn parts(&self) -> &[InstalledTreePart] {
        &self.parts
    }
}

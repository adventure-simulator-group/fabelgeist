//! Complete admitted assets use the production cache, materials and streamer.
use super::super::prepared_cache::{
    PreparedCluster, PreparedTreeCache, PreparedVariant, TreeCacheContentId,
};
use super::*;
use anyhow::{Context, Result, ensure};
use bevy::ecs::system::{SystemParam, SystemState};

use assets::AssetStores;
pub(super) use assets::require_resident;
pub(crate) use inspection::TreeCacheInspection;
pub(crate) use selection::TreeCacheRepresentation;

mod assets;
mod inspection;
mod selection;

const COMPLETE_STREAM_MASK: u8 = 0b11_1111;

pub(crate) struct InstalledTreeSpecimen {
    pub(crate) root: Entity,
    pub(crate) cache_key: Seed,
    pub(crate) origin: InstalledTreeOrigin,
    pub(crate) parts: Vec<InstalledTreePart>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InstalledTreePart {
    pub(crate) role: InstalledTreePartRole,
    pub(crate) mesh: AssetId<Mesh>,
    pub(crate) material: InstalledTreeMaterial,
}

struct Destination {
    environment: SceneEnvironment,
    terrain: SceneTerrain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InstalledTreeOrigin {
    Generated,
    Prepared(TreeCacheContentId),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InstalledTreeMaterial {
    Bark(AssetId<TacticalTreeBarkMaterial>),
    Aggregate(AssetId<TacticalTreeAggregateBarkMaterial>),
    Leaves(AssetId<TacticalTreeLeafCardMaterial>),
    Buds(AssetId<StandardMaterial>),
    Card {
        material: bevy::asset::UntypedAssetId,
        image: AssetId<Image>,
    },
}

impl PreparedTreeCache {
    /// Root is actual ground in area-world metres, not collider-centred mesh space.
    pub(crate) fn spawn(self, world: &mut World, root: Vec3) -> Result<InstalledTreeSpecimen> {
        let destination = Destination::at(world, root)?;
        let variant = self.into_variant(&destination.environment)?;
        InstalledTreeSpecimen::install(
            world,
            root,
            destination,
            VariantSource::Prepared(Box::new(variant)),
        )
    }
}

impl Destination {
    fn at(world: &World, root: Vec3) -> Result<Self> {
        let scene = world
            .resource::<ActiveTacticalScene>()
            .entity
            .context("active scene for review tree")?;
        let environment = world
            .get::<SceneEnvironment>(scene)
            .context("active tree environment")?
            .clone();
        let terrain = world
            .get::<SceneTerrain>(scene)
            .context("active tree terrain")?
            .clone();
        ensure!(
            root.is_finite()
                && terrain
                    .height_at(Vec2::new(root.x, root.z))
                    .is_some_and(|height| height.is_finite() && height == root.y),
            "review root requires actual destination ground"
        );
        Ok(Self {
            environment,
            terrain,
        })
    }
}

impl InstalledTreeSpecimen {
    /// Diagnostic prewarming uses the production generated cache owner, never decoded meshes.
    pub(crate) fn from_generated(
        world: &mut World,
        source: fabelgeist_botany::identity::PlantId,
        root: Vec3,
    ) -> Result<Self> {
        let specimen = super::super::authoring::specimens::SourceSpecimen::find(source)?;
        let destination = Destination::at(world, root)?;
        Self::install(world, root, destination, VariantSource::Generated(specimen))
    }

    fn install(
        world: &mut World,
        root: Vec3,
        destination: Destination,
        source: VariantSource,
    ) -> Result<Self> {
        let (species, origin) = match &source {
            VariantSource::Generated(specimen) => {
                (specimen.species, InstalledTreeOrigin::Generated)
            }
            VariantSource::Prepared(variant) => (
                variant.species,
                InstalledTreeOrigin::Prepared(variant.content),
            ),
        };
        let mut state = SystemState::<(
            ResMut<TreePresentationCache>,
            ResMut<RetainedTreePresentations>,
            ResMut<TreeAssetResidencyDiagnostics>,
            AssetStores,
        )>::new(world);
        let (key, parts) = {
            let (mut cache, mut retained, mut diagnostics, mut stores) = state.get_mut(world)?;
            retained.activate(
                &destination.environment,
                &destination.terrain,
                &mut cache,
                &mut diagnostics,
                &mut stores.images,
            );
            let key = match source {
                VariantSource::Prepared(variant) => {
                    cache.admit_variant(*variant, &destination.terrain, &mut stores)?
                }
                VariantSource::Generated(specimen) => cache.generate_review_variant(
                    &specimen,
                    &destination.environment,
                    &destination.terrain,
                    &mut stores,
                )?,
            };
            let installed = cache
                .variants
                .get(&key)
                .context("installed tree cache entry")?;
            diagnostics.add_resident_assets(&installed.resident_assets);
            let parts = inspection::installed_parts(installed, &stores.cards)?;
            retained.update_diagnostics(&cache, &mut diagnostics);
            (key, parts)
        };
        state.apply(world);
        let entity = world
            .spawn((
                Transform::from_translation(root + super::super::source::source_normalization()),
                Visibility::Inherited,
            ))
            .id();
        spawn_cached_tree(&mut world.commands(), entity, key, species.name());
        world.flush();
        Ok(Self {
            root: entity,
            cache_key: key,
            origin,
            parts,
        })
    }
}

//! Complete admitted assets use the production cache, materials and streamer.
use super::super::prepared_cache::{PreparedCluster, PreparedTreeCache, PreparedVariant};
use super::*;
use anyhow::{Context, Result, ensure};
use bevy::ecs::system::{SystemParam, SystemState};

mod assets;
mod inspection;
use assets::AssetStores;
pub(super) use assets::require_resident;
pub(crate) use inspection::PreparedCacheInspection;

const FULL_STREAM_MASK: u8 = 0b11;
const COMPLETE_STREAM_MASK: u8 = 0b11_1111;

pub(crate) struct InstalledCacheSpecimen {
    pub(crate) root: Entity,
    pub(crate) cache_key: Seed,
    pub(crate) content_id: blake3::Hash,
    pub(crate) parts: Vec<InstalledTreePart>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InstalledTreePart {
    pub(crate) role: InstalledTreePartRole,
    pub(crate) mesh: AssetId<Mesh>,
    pub(crate) material: InstalledTreeMaterial,
}

impl PreparedTreeCache {
    /// Root is actual ground in area-world metres, not collider-centred mesh space.
    pub(crate) fn spawn(self, world: &mut World, root: Vec3) -> Result<InstalledCacheSpecimen> {
        let scene = world
            .resource::<ActiveTacticalScene>()
            .entity
            .context("active scene for prepared tree")?;
        let environment = world
            .get::<SceneEnvironment>(scene)
            .context("active scene environment for prepared tree")?
            .clone();
        let terrain = world
            .get::<SceneTerrain>(scene)
            .context("active scene terrain for prepared tree")?
            .clone();
        ensure!(
            root.is_finite()
                && terrain
                    .height_at(Vec2::new(root.x, root.z))
                    .is_some_and(|height| height.is_finite() && height == root.y),
            "prepared root requires actual destination ground"
        );
        let variant = self.into_variant(&environment)?;
        let content_id = variant.content;
        let species = variant.species;
        let mut state = SystemState::<(
            ResMut<TreePresentationCache>,
            ResMut<RetainedTreePresentations>,
            ResMut<TreeAssetResidencyDiagnostics>,
            AssetStores,
        )>::new(world);
        let (key, parts) = {
            let (mut cache, mut retained, mut diagnostics, mut stores) = state.get_mut(world)?;
            retained.activate(
                &environment,
                &terrain,
                &mut cache,
                &mut diagnostics,
                &mut stores.images,
            );
            let key = cache.admit_variant(variant, &terrain, &mut stores)?;
            let installed = cache
                .variants
                .get(&key)
                .context("admitted tree cache entry")?;
            diagnostics.add_resident_assets(&installed.resident_assets);
            ensure!(
                matches!(&installed.origin, TreeAssetOrigin::Prepared(content) if *content == content_id),
                "installed prepared tree content identity"
            );
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
        Ok(InstalledCacheSpecimen {
            root: entity,
            cache_key: key,
            content_id,
            parts,
        })
    }
}

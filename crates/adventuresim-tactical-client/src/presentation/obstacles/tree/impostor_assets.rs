//! Canonical software-baked tree cards loaded only by fixed prepared scenes.

use super::TreePresentationSpecies;
#[cfg(test)]
use super::impostor::{BEECH_TREE_BAKE_STYLE, OAK_TREE_BAKE_STYLE, bake_tree_lod_with_style};
use super::impostor::{
    TREE_IMPOSTOR_BAKE_VERSION, TREE_IMPOSTOR_RENDER_METHOD, TreeImpostorBakeRecord,
    TreeImpostorProvenance, TreeLodBake, TreeLodClusterBake, tree_source_geometry_hash,
};
#[cfg(test)]
use super::source::{
    canopy_competition, playable_tree_source, tree_species_for_site, vista_tree_source,
};
#[cfg(test)]
use super::specimen::oak_variant_for_site;
use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    ecs::system::SystemParam,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use fabelgeist_determinism::Seed;
use flate2::read::GzDecoder;
#[cfg(test)]
use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::io::Read;

pub(super) mod bounded;
mod records;
pub(super) use records::PreparedTreeLodBake;

const MAX_CARD_VERTICES: usize = 100_000;
const MAX_CARD_INDICES: usize = 600_000;
const MAX_CARD_PIXELS: usize = 64 * 1024 * 1024;
const MAX_CARD_RECORDS: usize = 4096;
const MAX_CARD_GROUPS: usize = super::TREE_PRIMARY_GROUP_COUNT as usize;
#[cfg(test)]
use std::io::Write;

#[derive(Asset, TypePath)]
pub(crate) struct PreparedTreeImpostorAsset {
    bakes: Vec<PreparedTreeLodBake>,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub(in crate::presentation) enum PreparedTreeImpostorUsage {
    Playable,
    Vista,
}

fn species_tag(species: TreePresentationSpecies) -> u8 {
    match species {
        TreePresentationSpecies::EnglishOak => 0,
        TreePresentationSpecies::CommonBeech => 1,
    }
}

impl PreparedTreeImpostorAsset {
    fn from_compressed_bytes(compressed: &[u8]) -> std::io::Result<Self> {
        let mut bytes = Vec::new();
        GzDecoder::new(compressed).read_to_end(&mut bytes)?;
        let bakes = postcard::from_bytes(&bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        Ok(Self { bakes })
    }

    pub(super) fn resolve(
        &self,
        usage: PreparedTreeImpostorUsage,
        species: TreePresentationSpecies,
        seed: Seed,
        lod: u8,
    ) -> Option<TreeLodBake> {
        self.bakes
            .iter()
            .find(|bake| bake.matches(usage, species, seed, lod))
            .map(PreparedTreeLodBake::runtime)
    }

    #[cfg(test)]
    pub(crate) fn compressed_bytes(&self) -> Vec<u8> {
        let encoded = postcard::to_allocvec(&self.bakes).expect("tree impostor asset serializes");
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&encoded).expect("gzip accepts postcard");
        encoder.finish().expect("gzip tree impostor asset")
    }

    #[cfg(test)]
    pub(crate) fn bake_count(&self) -> usize {
        self.bakes.len()
    }

    #[cfg(test)]
    pub(crate) fn decode(compressed: &[u8]) -> Self {
        Self::from_compressed_bytes(compressed).expect("prepared tree impostor asset decodes")
    }

    #[cfg(test)]
    pub(crate) fn matches_art_demo(
        &self,
        position: Vec3,
        environment: &crate::presentation::SceneEnvironment,
    ) -> bool {
        let competition = canopy_competition(environment.canopy_bps);
        let species = tree_species_for_site(position, environment);
        let (variant_index, variant_seed) = oak_variant_for_site(position);
        let (branches, leaves) = playable_tree_source(
            species,
            variant_seed,
            variant_index,
            competition,
            environment,
        );
        let source_hash =
            tree_source_geometry_hash(&branches, &leaves) ^ u64::from(TREE_IMPOSTOR_BAKE_VERSION);
        if !(1..=4).all(|lod| {
            self.resolve(
                PreparedTreeImpostorUsage::Playable,
                species,
                variant_seed,
                lod,
            )
            .is_some_and(|bake| bake.provenance.source_geometry_hash == source_hash)
        }) {
            return false;
        }
        let vista_seed = crate::presentation::obstacles::tree::specimen::oak_variant_seed(0);
        [
            TreePresentationSpecies::EnglishOak,
            TreePresentationSpecies::CommonBeech,
        ]
        .into_iter()
        .all(|vista_species| {
            let (branches, leaves) = vista_tree_source(vista_seed, 0.5, vista_species);
            let source_hash = tree_source_geometry_hash(&branches, &leaves)
                ^ u64::from(TREE_IMPOSTOR_BAKE_VERSION);
            self.resolve(
                PreparedTreeImpostorUsage::Vista,
                vista_species,
                vista_seed,
                4,
            )
            .is_some_and(|bake| bake.provenance.source_geometry_hash == source_hash)
        })
    }
}

#[cfg(test)]
pub(crate) fn prepare_art_demo_tree_impostor_asset(
    position: Vec3,
    environment: &crate::presentation::SceneEnvironment,
) -> PreparedTreeImpostorAsset {
    let competition = canopy_competition(environment.canopy_bps);
    let species = tree_species_for_site(position, environment);
    let (variant_index, variant_seed) = oak_variant_for_site(position);
    let (branches, leaves) = playable_tree_source(
        species,
        variant_seed,
        variant_index,
        competition,
        environment,
    );
    let mut bakes = (1..=4)
        .map(|lod| {
            let bake = bake_tree_lod_with_style(
                variant_seed,
                &branches,
                &leaves,
                lod,
                match species {
                    TreePresentationSpecies::EnglishOak => OAK_TREE_BAKE_STYLE,
                    TreePresentationSpecies::CommonBeech => BEECH_TREE_BAKE_STYLE,
                },
            );
            PreparedTreeLodBake::from_runtime(&bake, PreparedTreeImpostorUsage::Playable, species)
                .expect("authored playable bake")
        })
        .collect::<Vec<_>>();
    let vista_seed = crate::presentation::obstacles::tree::specimen::oak_variant_seed(0);
    for vista_species in [
        TreePresentationSpecies::EnglishOak,
        TreePresentationSpecies::CommonBeech,
    ] {
        let (branches, leaves) = vista_tree_source(vista_seed, 0.5, vista_species);
        let bake = bake_tree_lod_with_style(
            vista_seed,
            &branches,
            &leaves,
            4,
            match vista_species {
                TreePresentationSpecies::EnglishOak => OAK_TREE_BAKE_STYLE,
                TreePresentationSpecies::CommonBeech => BEECH_TREE_BAKE_STYLE,
            },
        );
        bakes.push(
            PreparedTreeLodBake::from_runtime(
                &bake,
                PreparedTreeImpostorUsage::Vista,
                vista_species,
            )
            .expect("authored vista bake"),
        );
    }
    PreparedTreeImpostorAsset { bakes }
}

#[derive(Default, TypePath)]
pub(in crate::presentation) struct PreparedTreeImpostorLoader;

impl AssetLoader for PreparedTreeImpostorLoader {
    type Asset = PreparedTreeImpostorAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _context: &mut LoadContext<'_>,
    ) -> std::io::Result<Self::Asset> {
        let mut compressed = Vec::new();
        reader.read_to_end(&mut compressed).await?;
        PreparedTreeImpostorAsset::from_compressed_bytes(&compressed)
    }

    fn extensions(&self) -> &[&str] {
        &["tree-impostors"]
    }
}

#[derive(Default, Resource)]
pub(crate) struct PreparedTreeImpostorAssets {
    handle: Option<Handle<PreparedTreeImpostorAsset>>,
}

impl PreparedTreeImpostorAssets {
    pub(crate) fn request(&mut self, server: &AssetServer) {
        self.handle
            .get_or_insert_with(|| server.load("art-demo/oak.tree-impostors"));
    }

    pub(crate) fn release(&mut self) {
        self.handle = None;
    }

    pub(crate) fn handle(&self) -> Option<&Handle<PreparedTreeImpostorAsset>> {
        self.handle.as_ref()
    }

    pub(in crate::presentation) fn get<'a>(
        &self,
        assets: &'a Assets<PreparedTreeImpostorAsset>,
    ) -> Option<&'a PreparedTreeImpostorAsset> {
        self.handle.as_ref().and_then(|handle| assets.get(handle))
    }
}

#[derive(SystemParam)]
pub(in crate::presentation) struct PreparedTreeImpostors<'w> {
    residency: Res<'w, PreparedTreeImpostorAssets>,
    assets: Res<'w, Assets<PreparedTreeImpostorAsset>>,
}

impl<'w> PreparedTreeImpostors<'w> {
    pub(in crate::presentation) fn get(&self) -> Option<&PreparedTreeImpostorAsset> {
        self.residency.get(&self.assets)
    }
}

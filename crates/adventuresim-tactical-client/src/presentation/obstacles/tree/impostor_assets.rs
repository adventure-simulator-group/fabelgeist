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
use flate2::read::GzDecoder;
#[cfg(test)]
use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::io::Read;
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
        seed: u64,
        lod: u8,
    ) -> Option<TreeLodBake> {
        self.bakes
            .iter()
            .find(|bake| {
                bake.usage == usage
                    && bake.species == species_tag(species)
                    && bake.seed == seed
                    && bake.lod == lod
            })
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
        let vista_seed = fabelgeist_determinism::splitmix64(0x6f61_6b00);
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
        })
        .collect::<Vec<_>>();
    let vista_seed = fabelgeist_determinism::splitmix64(0x6f61_6b00);
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
        bakes.push(PreparedTreeLodBake::from_runtime(
            &bake,
            PreparedTreeImpostorUsage::Vista,
            vista_species,
        ));
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

#[derive(Serialize, Deserialize)]
struct PreparedTreeLodBake {
    usage: PreparedTreeImpostorUsage,
    species: u8,
    seed: u64,
    lod: u8,
    bake_version: u32,
    source_geometry_hash: u64,
    atlas_width: u32,
    atlas_height: u32,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    clusters: Vec<PreparedTreeLodCluster>,
    pixels: Vec<u8>,
    records: Vec<PreparedTreeImpostorRecord>,
}

impl PreparedTreeLodBake {
    #[cfg(test)]
    fn from_runtime(
        bake: &TreeLodBake,
        usage: PreparedTreeImpostorUsage,
        species: TreePresentationSpecies,
    ) -> Self {
        let attribute = |name| bake.mesh.attribute(name).expect("tree card attribute");
        let VertexAttributeValues::Float32x3(positions) = attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("tree card positions use Float32x3")
        };
        let VertexAttributeValues::Float32x3(normals) = attribute(Mesh::ATTRIBUTE_NORMAL) else {
            panic!("tree card normals use Float32x3")
        };
        let VertexAttributeValues::Float32x2(uvs) = attribute(Mesh::ATTRIBUTE_UV_0) else {
            panic!("tree card UVs use Float32x2")
        };
        let indices = match bake.mesh.indices().expect("tree card indices") {
            Indices::U16(indices) => indices.iter().map(|index| u32::from(*index)).collect(),
            Indices::U32(indices) => indices.clone(),
        };
        Self {
            usage,
            species: species_tag(species),
            seed: bake.provenance.seed,
            lod: bake.lod,
            bake_version: bake.provenance.bake_version,
            source_geometry_hash: bake.provenance.source_geometry_hash,
            atlas_width: bake.provenance.atlas_width,
            atlas_height: bake.provenance.atlas_height,
            positions: positions.clone(),
            normals: normals.clone(),
            uvs: uvs.clone(),
            indices,
            clusters: bake
                .clusters
                .iter()
                .map(PreparedTreeLodCluster::from)
                .collect(),
            pixels: bake.image.data.clone().expect("tree atlas retains pixels"),
            records: bake
                .provenance
                .records
                .iter()
                .map(PreparedTreeImpostorRecord::from)
                .collect(),
        }
    }

    fn runtime(&self) -> TreeLodBake {
        assert_eq!(self.bake_version, TREE_IMPOSTOR_BAKE_VERSION);
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone());
        mesh.insert_indices(Indices::U32(self.indices.clone()));
        TreeLodBake {
            lod: self.lod,
            mesh,
            clusters: self.clusters.iter().map(TreeLodClusterBake::from).collect(),
            image: Image::new(
                Extent3d {
                    width: self.atlas_width,
                    height: self.atlas_height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                self.pixels.clone(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            ),
            provenance: TreeImpostorProvenance {
                seed: self.seed,
                lod: self.lod,
                bake_version: self.bake_version,
                source_geometry_hash: self.source_geometry_hash,
                render_method: TREE_IMPOSTOR_RENDER_METHOD,
                atlas_width: self.atlas_width,
                atlas_height: self.atlas_height,
                records: self
                    .records
                    .iter()
                    .map(TreeImpostorBakeRecord::from)
                    .collect(),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PreparedTreeLodCluster {
    primary_group: u8,
    center: [f32; 3],
    radius: f32,
}

impl From<&TreeLodClusterBake> for PreparedTreeLodCluster {
    fn from(cluster: &TreeLodClusterBake) -> Self {
        Self {
            primary_group: cluster.primary_group,
            center: cluster.center.to_array(),
            radius: cluster.radius,
        }
    }
}

impl From<&PreparedTreeLodCluster> for TreeLodClusterBake {
    fn from(cluster: &PreparedTreeLodCluster) -> Self {
        Self {
            primary_group: cluster.primary_group,
            center: Vec3::from_array(cluster.center),
            radius: cluster.radius,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PreparedTreeImpostorRecord {
    source_group: u16,
    source_leaf_count: u16,
    source_branch_count: u16,
    view_direction: [f32; 3],
    projected_bounds: [f32; 4],
    atlas_region: [u32; 4],
    opaque_pixel_count: u32,
    silhouette_centroid: [f32; 2],
}

impl From<&TreeImpostorBakeRecord> for PreparedTreeImpostorRecord {
    fn from(record: &TreeImpostorBakeRecord) -> Self {
        Self {
            source_group: record.source_group,
            source_leaf_count: record.source_leaf_count,
            source_branch_count: record.source_branch_count,
            view_direction: record.view_direction.to_array(),
            projected_bounds: record.projected_bounds.to_array(),
            atlas_region: record.atlas_region.to_array(),
            opaque_pixel_count: record.opaque_pixel_count,
            silhouette_centroid: record.silhouette_centroid.to_array(),
        }
    }
}

impl From<&PreparedTreeImpostorRecord> for TreeImpostorBakeRecord {
    fn from(record: &PreparedTreeImpostorRecord) -> Self {
        Self {
            source_group: record.source_group,
            source_leaf_count: record.source_leaf_count,
            source_branch_count: record.source_branch_count,
            view_direction: Vec3::from_array(record.view_direction),
            projected_bounds: Vec4::from_array(record.projected_bounds),
            atlas_region: UVec4::from_array(record.atlas_region),
            opaque_pixel_count: record.opaque_pixel_count,
            silhouette_centroid: Vec2::from_array(record.silhouette_centroid),
        }
    }
}

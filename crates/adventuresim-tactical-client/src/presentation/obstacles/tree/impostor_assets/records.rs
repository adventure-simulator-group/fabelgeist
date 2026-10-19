//! Production atlas channels and provenance at the serialized asset boundary.

use super::*;

#[derive(Serialize, Deserialize)]
pub(in crate::presentation::obstacles::tree) struct PreparedTreeLodBake {
    usage: PreparedTreeImpostorUsage,
    species: u8,
    seed: Seed,
    lod: u8,
    bake_version: u32,
    source_geometry_hash: u64,
    atlas_width: u32,
    atlas_height: u32,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_CARD_VERTICES>")]
    positions: Vec<[f32; 3]>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_CARD_VERTICES>")]
    normals: Vec<[f32; 3]>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_CARD_VERTICES>")]
    uvs: Vec<[f32; 2]>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_CARD_INDICES>")]
    indices: Vec<u32>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_CARD_GROUPS>")]
    clusters: Vec<PreparedTreeLodCluster>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_CARD_PIXELS>")]
    pixels: Vec<u8>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_CARD_RECORDS>")]
    records: Vec<PreparedTreeImpostorRecord>,
}

impl PreparedTreeLodBake {
    pub(super) fn matches(
        &self,
        usage: PreparedTreeImpostorUsage,
        species: TreePresentationSpecies,
        seed: Seed,
        lod: u8,
    ) -> bool {
        self.usage == usage
            && self.species == species_tag(species)
            && self.seed == seed
            && self.lod == lod
    }

    #[cfg(any(test, all(feature = "foliage-review", not(target_family = "wasm"))))]
    pub(in crate::presentation::obstacles::tree) fn from_runtime(
        bake: &TreeLodBake,
        usage: PreparedTreeImpostorUsage,
        species: TreePresentationSpecies,
    ) -> anyhow::Result<Self> {
        use anyhow::Context;
        let attribute = |name| bake.mesh.attribute(name).context("tree card attribute");
        let VertexAttributeValues::Float32x3(positions) = attribute(Mesh::ATTRIBUTE_POSITION)?
        else {
            anyhow::bail!("tree card positions use Float32x3")
        };
        let VertexAttributeValues::Float32x3(normals) = attribute(Mesh::ATTRIBUTE_NORMAL)? else {
            anyhow::bail!("tree card normals use Float32x3")
        };
        let VertexAttributeValues::Float32x2(uvs) = attribute(Mesh::ATTRIBUTE_UV_0)? else {
            anyhow::bail!("tree card UVs use Float32x2")
        };
        let indices = match bake.mesh.indices().context("tree card indices")? {
            Indices::U16(indices) => indices.iter().map(|index| u32::from(*index)).collect(),
            Indices::U32(indices) => indices.clone(),
        };
        Ok(Self {
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
            pixels: bake
                .image
                .data
                .clone()
                .context("tree atlas retains pixels")?,
            records: bake
                .provenance
                .records
                .iter()
                .map(PreparedTreeImpostorRecord::from)
                .collect(),
        })
    }

    #[cfg(all(feature = "foliage-review", not(target_family = "wasm")))]
    pub(in crate::presentation::obstacles::tree) fn maximum_storage_bytes() -> usize {
        MAX_CARD_VERTICES * (std::mem::size_of::<[f32; 3]>() * 2 + std::mem::size_of::<[f32; 2]>())
            + MAX_CARD_INDICES * std::mem::size_of::<u32>()
            + MAX_CARD_PIXELS
            + MAX_CARD_RECORDS * std::mem::size_of::<PreparedTreeImpostorRecord>()
            + MAX_CARD_GROUPS * std::mem::size_of::<PreparedTreeLodCluster>()
    }

    #[cfg(all(feature = "foliage-review", not(target_family = "wasm")))]
    pub(in crate::presentation::obstacles::tree) fn validate_full_dependency(
        &self,
        species: TreePresentationSpecies,
        seed: Seed,
    ) -> anyhow::Result<()> {
        use anyhow::ensure;
        ensure!(
            self.usage == PreparedTreeImpostorUsage::Playable
                && self.species == species_tag(species)
                && self.seed == seed
                && self.lod == 1
                && self.bake_version == TREE_IMPOSTOR_BAKE_VERSION,
            "Full cache card identity"
        );
        let pixels = u64::from(self.atlas_width)
            .checked_mul(u64::from(self.atlas_height))
            .and_then(|count| count.checked_mul(4));
        ensure!(
            self.atlas_width > 0
                && self.atlas_height > 0
                && pixels == Some(self.pixels.len() as u64)
                && self.pixels.len() <= MAX_CARD_PIXELS,
            "card image dimensions"
        );
        ensure!(
            !self.positions.is_empty()
                && self.positions.len() == self.normals.len()
                && self.positions.len() == self.uvs.len(),
            "card channel cardinality"
        );
        ensure!(
            !self.indices.is_empty()
                && self.indices.len().is_multiple_of(3)
                && self
                    .indices
                    .iter()
                    .all(|index| (*index as usize) < self.positions.len()),
            "card triangle indices"
        );
        ensure!(
            self.positions
                .iter()
                .flatten()
                .chain(self.normals.iter().flatten())
                .chain(self.uvs.iter().flatten())
                .all(|value| value.is_finite()),
            "nonfinite card mesh"
        );
        let mut groups = std::collections::BTreeSet::new();
        ensure!(
            !self.clusters.is_empty() && !self.records.is_empty() && self.source_geometry_hash != 0,
            "card cluster/provenance missing"
        );
        for cluster in &self.clusters {
            ensure!(
                cluster.primary_group < super::super::TREE_PRIMARY_GROUP_COUNT
                    && groups.insert(cluster.primary_group)
                    && Vec3::from_array(cluster.center).is_finite()
                    && cluster.radius.is_finite()
                    && cluster.radius > 0.0,
                "card cluster shape/group"
            );
        }
        for record in &self.records {
            let [x, y, width, height] = record.atlas_region;
            ensure!(
                width > 0
                    && height > 0
                    && x.checked_add(width)
                        .is_some_and(|end| end <= self.atlas_width)
                    && y.checked_add(height)
                        .is_some_and(|end| end <= self.atlas_height),
                "card atlas region"
            );
            ensure!(
                record.source_leaf_count > 0
                    && record.source_branch_count > 0
                    && record.opaque_pixel_count > 0
                    && u64::from(record.opaque_pixel_count) <= u64::from(width) * u64::from(height),
                "card coverage counts"
            );
            ensure!(
                record
                    .view_direction
                    .iter()
                    .chain(&record.projected_bounds)
                    .chain(&record.silhouette_centroid)
                    .all(|value| value.is_finite()),
                "nonfinite card provenance"
            );
        }
        Ok(())
    }

    pub(in crate::presentation::obstacles::tree) fn runtime(&self) -> TreeLodBake {
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

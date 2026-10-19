//! Complete production tree cache payload for native content and LOD trials.
use super::{
    AggregateTreeLod, TreePresentationSpecies,
    authoring::specimens::SourceSpecimen,
    impostor::{TreeLodBake, bake_tree_lod_with_style},
    impostor_assets::{PreparedTreeImpostorUsage, PreparedTreeLodBake, bounded},
    procedural_oak_bud_group_mesh, procedural_oak_leaf_card_group_mesh,
    procedural_oak_textured_leaf_group_mesh, procedural_tree_branch_group_mesh,
    procedural_tree_branch_mesh, procedural_woody_mid_trunk_mesh,
    source::playable_cache_key,
};
use crate::presentation::SceneEnvironment;
use adventuresim_tactical_core::scene_input::TacticalSceneInput;
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;
use fabelgeist_determinism::Seed;
use flate2::bufread::GzDecoder;
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::Path};

mod author;
pub(crate) use author::{inspect_tree_cache, write_tree_cache};
mod mesh;
mod runtime;
pub(super) use runtime::{PreparedCluster, PreparedVariant};
mod source;
mod validation;

const FORMAT: &[u8; 8] = b"TREECPU2";
const MAX_PAYLOAD_BYTES: u64 = 128 * 1024 * 1024;
const MAX_COMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_BINDING_BYTES: usize = 32_768;
const MAX_CONTAINER_OVERHEAD_BYTES: usize = 2 * 1024 * 1024;
const MAX_ADMISSION_BYTES: u64 = 512 * 1024 * 1024;
const RECIPE_VERSION: u32 = 1;
const MAX_DECODED_ELEMENT_EXPANSION: u64 = 4;
const MAX_GROUPS: usize = super::TREE_PRIMARY_GROUP_COUNT as usize;

#[derive(Serialize, Deserialize)]
struct Binding {
    source: u64,
    seed: Seed,
    species: TreePresentationSpecies,
    cache_key: Seed,
    recipe_version: u32,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_BINDING_BYTES>")]
    input: Vec<u8>,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_BINDING_BYTES>")]
    environment: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    binding: Binding,
    source: source::Source,
    detailed_trunk: mesh::PreparedMesh,
    mid_trunk: mesh::PreparedMesh,
    #[serde(deserialize_with = "bounded::sequence::<_, _, MAX_GROUPS>")]
    groups: Vec<Group>,
    aggregate_wood: [mesh::PreparedMesh; 2],
    cards: [PreparedTreeLodBake; 4],
}

#[derive(Serialize, Deserialize)]
struct Group {
    primary_group: u8,
    center: [f32; 3],
    radius: f32,
    wood: mesh::PreparedMesh,
    buds: mesh::PreparedMesh,
    leaves: mesh::PreparedMesh,
    flat_leaves: mesh::PreparedMesh,
}

/// Only checked decoded CPU data can cross into the live asset stores.
pub(crate) struct PreparedTreeCache {
    payload: Payload,
    digest: blake3::Hash,
}

impl PreparedTreeCache {
    pub(crate) fn load(
        path: &Path,
        input: &TacticalSceneInput,
        environment: &SceneEnvironment,
        id: PlantId,
    ) -> Result<Self> {
        let mut file = fs::File::open(path)?;
        let length = file.metadata()?.len();
        ensure!(
            length <= MAX_COMPRESSED_BYTES,
            "compressed tree cache limit"
        );
        let mut bytes = vec![0; usize::try_from(length)?];
        file.read_exact(&mut bytes)?;
        ensure!(
            file.read(&mut [0])? == 0,
            "tree cache changed while reading"
        );
        let checked = Self::decode(&bytes, input, environment, id)?;
        checked.verify_source(environment, id)?;
        Ok(checked)
    }

    fn decode(
        bytes: &[u8],
        input: &TacticalSceneInput,
        environment: &SceneEnvironment,
        id: PlantId,
    ) -> Result<Self> {
        let header_bytes = FORMAT.len() + blake3::OUT_LEN + std::mem::size_of::<u64>();
        ensure!(
            bytes.len() as u64 <= MAX_COMPRESSED_BYTES && bytes.len() >= header_bytes,
            "tree cache encoded limit/header"
        );
        ensure!(&bytes[..FORMAT.len()] == FORMAT, "tree cache format");
        let (digest, body) = bytes[FORMAT.len()..].split_at(blake3::OUT_LEN);
        let (length, compressed) = body.split_at(std::mem::size_of::<u64>());
        let length = u64::from_le_bytes(length.try_into()?);
        ensure!(length <= MAX_PAYLOAD_BYTES, "decoded tree cache limit");
        checked_allocation_bound(length, bytes.len() as u64)?;
        // The declared size is bounded before one exact allocation. Reading one
        // extra byte checks EOF/CRC without allowing growth beyond that buffer.
        let mut decoded = vec![0; usize::try_from(length)?];
        let mut decoder = GzDecoder::new(compressed);
        decoder.read_exact(&mut decoded)?;
        ensure!(
            decoder.read(&mut [0])? == 0,
            "tree cache decompressed size mismatch"
        );
        ensure!(
            decoder.into_inner().is_empty(),
            "tree cache compressed trailing data"
        );
        let payload = decoded.as_slice();
        let actual = blake3::hash(payload);
        ensure!(actual.as_bytes() == digest, "tree cache digest");
        let (payload, tail): (Payload, _) = postcard::take_from_bytes(payload)?;
        ensure!(tail.is_empty(), "tree cache trailing payload");
        payload.validate(input, environment, id)?;
        Ok(Self {
            payload,
            digest: actual,
        })
    }

    pub(crate) fn content_id(&self) -> String {
        self.digest.to_hex().to_string()
    }

    pub(crate) fn verify_source(&self, environment: &SceneEnvironment, id: PlantId) -> Result<()> {
        let specimen = SourceSpecimen::find(id)?;
        let (branches, leaves) = specimen.records(environment);
        ensure!(
            postcard::to_allocvec(&source::Source::from_records(&branches, &leaves))?
                == postcard::to_allocvec(&self.payload.source)?,
            "exact selected source record mismatch"
        );
        for card in &self.payload.cards {
            card.validate_source(&branches, &leaves)?;
        }
        for group in &self.payload.groups {
            let expected = procedural_oak_bud_group_mesh(&branches, group.primary_group);
            ensure!(
                group.buds.vertices() == expected.count_vertices(),
                "source bud completeness"
            );
        }
        Ok(())
    }
}

fn checked_allocation_bound(payload: u64, compressed: u64) -> Result<u64> {
    // For complete postcard collections u32 indices are the tightest element
    // expansion (four native bytes from one wire byte). Float arrays and the
    // fixed source/card records have smaller ratios. Fixed-count nested Vec
    // headers and decoder/container overhead fit the separate allowance.
    let pending = PreparedTreeLodBake::maximum_pending_collection_bytes();
    let mesh_copy =
        mesh::MAX_PART_VERTICES * 48 + mesh::MAX_PART_INDICES * std::mem::size_of::<u32>();
    ensure!(
        source::Source::maximum_storage_bytes() <= pending && mesh_copy <= pending,
        "tree cache pending-collection bound"
    );
    // A malformed final collection may reserve its full permitted capacity
    // before its truncated contents fail. All preceding complete collections
    // are covered by the wire expansion. Validation clones one card and one
    // mesh at a time; never keep four validation bakes simultaneously.
    let scratch = PreparedTreeLodBake::maximum_storage_bytes() + mesh_copy;
    let bound = payload
        .checked_mul(MAX_DECODED_ELEMENT_EXPANSION)
        .and_then(|bytes| bytes.checked_add(payload))
        .and_then(|bytes| bytes.checked_add(compressed))
        .and_then(|bytes| {
            bytes.checked_add((pending + scratch + MAX_CONTAINER_OVERHEAD_BYTES) as u64)
        })
        .context("tree cache allocation bound overflow")?;
    ensure!(
        bound <= MAX_ADMISSION_BYTES,
        "tree cache aggregate allocation budget"
    );
    Ok(bound)
}

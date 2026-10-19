//! Fixed-Full production cache payload: native content trial, not adaptive-ready.
use super::{
    TreePresentationSpecies,
    authoring::specimens::SourceSpecimen,
    impostor::{TreeLodBake, bake_tree_lod_with_style},
    impostor_assets::{PreparedTreeImpostorUsage, PreparedTreeLodBake, bounded},
    procedural_oak_bud_group_mesh, procedural_oak_textured_leaf_group_mesh,
    procedural_tree_branch_group_mesh, procedural_tree_branch_mesh,
    procedural_woody_mid_trunk_mesh,
    source::playable_cache_key,
};
use crate::presentation::SceneEnvironment;
use adventuresim_tactical_core::scene_input::TacticalSceneInput;
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;
use fabelgeist_determinism::Seed;
use flate2::{Compression, bufread::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

mod mesh;
mod source;
mod validation;

const FORMAT: &[u8; 8] = b"TREECPU1";
const MAX_PAYLOAD_BYTES: u64 = 128 * 1024 * 1024;
const MAX_COMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_BINDING_BYTES: usize = 32_768;
const MAX_CONTAINER_OVERHEAD_BYTES: usize = 2 * 1024 * 1024;
const MAX_ADMISSION_BYTES: u64 = 512 * 1024 * 1024;
const MAX_GROUPS: usize = super::TREE_PRIMARY_GROUP_COUNT as usize;

#[derive(Serialize, Deserialize)]
struct Binding {
    source: u64,
    seed: Seed,
    species: TreePresentationSpecies,
    cache_key: Seed,
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
    card: PreparedTreeLodBake,
}

#[derive(Serialize, Deserialize)]
struct Group {
    primary_group: u8,
    center: [f32; 3],
    radius: f32,
    wood: mesh::PreparedMesh,
    buds: mesh::PreparedMesh,
    leaves: mesh::PreparedMesh,
}

/// Only checked decoded CPU data can cross into the live asset stores.
pub(crate) struct PreparedTreeCache {
    payload: Payload,
    digest: blake3::Hash,
}

impl Payload {
    fn author(
        input: &TacticalSceneInput,
        environment: &SceneEnvironment,
        id: PlantId,
    ) -> Result<Self> {
        let specimen = SourceSpecimen::find(id)?;
        let (branches, leaves) = specimen.records(environment);
        let bake = bake_tree_lod_with_style(
            specimen.seed,
            &branches,
            &leaves,
            1,
            specimen.species.bake_style(),
        );
        let groups = bake
            .clusters
            .iter()
            .map(|cluster| {
                Ok(Group {
                    primary_group: cluster.primary_group,
                    center: cluster.center.to_array(),
                    radius: cluster.radius,
                    wood: mesh::PreparedMesh::from_mesh(&procedural_tree_branch_group_mesh(
                        &branches,
                        3,
                        cluster.primary_group,
                    ))?,
                    buds: mesh::PreparedMesh::from_mesh(&procedural_oak_bud_group_mesh(
                        &branches,
                        cluster.primary_group,
                    ))?,
                    leaves: mesh::PreparedMesh::from_mesh(
                        &procedural_oak_textured_leaf_group_mesh(&leaves, cluster.primary_group),
                    )?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            binding: Binding {
                source: id.word(),
                seed: specimen.seed,
                species: specimen.species,
                cache_key: playable_cache_key(specimen.species, specimen.seed, environment),
                input: input.digest()?.into_bytes(),
                environment: serde_json::to_vec(environment)?,
            },
            source: source::Source::from_records(&branches, &leaves),
            detailed_trunk: mesh::PreparedMesh::from_mesh(&procedural_tree_branch_mesh(
                &branches, 0,
            ))?,
            mid_trunk: mesh::PreparedMesh::from_mesh(&procedural_woody_mid_trunk_mesh(&branches))?,
            card: PreparedTreeLodBake::from_runtime(
                &bake,
                PreparedTreeImpostorUsage::Playable,
                specimen.species,
            )?,
            groups,
        })
    }
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
        // Explicit worst-case bound, checked before any payload collection allocation.
        // Postcard provides exact lengths; the bounded visitor reserves them once.
        let near_parts = 2 + MAX_GROUPS * 3;
        let part_bytes = mesh::MAX_PART_VERTICES
            .checked_mul(48)
            .and_then(|bytes| {
                bytes.checked_add(mesh::MAX_PART_INDICES * std::mem::size_of::<u32>())
            })
            .context("tree mesh budget overflow")?;
        let decoded_storage = near_parts
            .checked_mul(part_bytes)
            .and_then(|bytes| bytes.checked_add(PreparedTreeLodBake::maximum_storage_bytes()))
            .and_then(|bytes| {
                bytes.checked_add(source::Source::maximum_storage_bytes() + MAX_BINDING_BYTES * 2)
            })
            .context("tree cache aggregate budget overflow")?;
        // Validation reconstructs the complete card (including atlas/provenance),
        // then copies its mesh channels. Reserve additional container overhead.
        let validation_copies = PreparedTreeLodBake::maximum_storage_bytes() + part_bytes;
        let admission = (decoded_storage + validation_copies + MAX_CONTAINER_OVERHEAD_BYTES) as u64
            + MAX_PAYLOAD_BYTES
            + MAX_COMPRESSED_BYTES;
        ensure!(
            admission <= MAX_ADMISSION_BYTES,
            "tree cache aggregate allocation budget"
        );
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

pub(crate) fn write_tree_cache(input_path: &Path, output: &Path, source_word: u64) -> Result<()> {
    ensure!(!output.exists(), "fresh cache output directory required");
    let input = TacticalSceneInput::load(input_path)?;
    let generated = input.generate()?;
    let environment = input.environment_snapshot(generated.digest);
    let id = PlantId::from_word(source_word);
    let payload = Payload::author(&input, &environment, id)?;
    payload.validate(&input, &environment, id)?;
    let encoded = postcard::to_allocvec(&payload)?;
    let digest = blake3::hash(&encoded);
    ensure!(
        encoded.len() as u64 <= MAX_PAYLOAD_BYTES,
        "authored payload limit"
    );
    let mut compressor = GzEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(&encoded)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(FORMAT);
    bytes.extend_from_slice(digest.as_bytes());
    bytes.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&compressor.finish()?);
    ensure!(
        bytes.len() as u64 <= MAX_COMPRESSED_BYTES,
        "compressed tree cache limit"
    );
    let decoded = PreparedTreeCache::decode(&bytes, &input, &environment, id)?;
    ensure!(
        postcard::to_allocvec(&decoded.payload)? == encoded,
        "lossless tree cache round trip"
    );
    decoded.verify_source(&environment, id)?;
    let report = serde_json::json!({"format": "production-cache-full-v1", "content_id": decoded.content_id(), "source_id": source_word,
        "species": payload.binding.species, "seed": payload.binding.seed.to_u64(), "cache_key": payload.binding.cache_key.to_u64(),
        "input_digest": input.digest()?, "environment": environment, "compressed_bytes": bytes.len(), "payload_bytes": encoded.len(),
        "source_frame": "collider-centred metres; placement adds half-trunk normalization once", "capability": "fixed Full with cambered leaves and hidden LOD1 dependency", "groups": payload.groups.len(),
        "detailed_trunk_vertices": payload.detailed_trunk.vertices(), "mid_trunk_vertices": payload.mid_trunk.vertices(),
        "group_vertices": payload.groups.iter().map(|group| serde_json::json!({"group":group.primary_group,"wood":group.wood.vertices(),"buds":group.buds.vertices(),"leaves":group.leaves.vertices()})).collect::<Vec<_>>(),
        "exact_payload_round_trip": true, "exact_source_records":true, "limits":"offline authoring only; no material, image, adaptive, arrival or performance acceptance"});
    fs::create_dir(output).with_context(|| format!("create fresh cache {}", output.display()))?;
    fs::write(output.join("variant.treecache"), bytes)?;
    fs::write(
        output.join("cache.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// CPU-only admission control, including exact selected-source validation.
pub(crate) fn inspect_tree_cache(
    input_path: &Path,
    cache_path: &Path,
    source_word: u64,
) -> Result<()> {
    let input = TacticalSceneInput::load(input_path)?;
    let generated = input.generate()?;
    let environment = input.environment_snapshot(generated.digest);
    let checked = PreparedTreeCache::load(
        cache_path,
        &input,
        &environment,
        PlantId::from_word(source_word),
    )?;
    println!(
        "{}",
        serde_json::json!({"content_id": checked.content_id(), "source_id":source_word,"fixed_full_admitted": true,"scope":"CPU payload/source admission only"})
    );
    Ok(())
}

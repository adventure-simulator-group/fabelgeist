//! Complete offline production cache authoring and exact source inspection.
use super::*;
use flate2::{Compression, write::GzEncoder};
use std::io::Write;

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
                    flat_leaves: mesh::PreparedMesh::from_mesh(
                        &procedural_oak_leaf_card_group_mesh(&leaves, cluster.primary_group),
                    )?,
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
                recipe_version: RECIPE_VERSION,
                input: input.digest()?.into_bytes(),
                environment: serde_json::to_vec(environment)?,
            },
            source: source::Source::from_records(&branches, &leaves),
            detailed_trunk: mesh::PreparedMesh::from_mesh(&procedural_tree_branch_mesh(
                &branches, 0,
            ))?,
            mid_trunk: mesh::PreparedMesh::from_mesh(&procedural_woody_mid_trunk_mesh(&branches))?,
            aggregate_wood: [
                mesh::PreparedMesh::from_mesh(&AggregateTreeLod::Near.mesh(&branches))?,
                mesh::PreparedMesh::from_mesh(&AggregateTreeLod::Middle.mesh(&branches))?,
            ],
            cards: [
                PreparedTreeLodBake::from_runtime(
                    &bake,
                    PreparedTreeImpostorUsage::Playable,
                    specimen.species,
                )?,
                Self::author_card(&specimen, &branches, &leaves, 2)?,
                Self::author_card(&specimen, &branches, &leaves, 3)?,
                Self::author_card(&specimen, &branches, &leaves, 4)?,
            ],
            groups,
        })
    }
    fn author_card(
        specimen: &SourceSpecimen,
        branches: &[super::super::TreeBranchSegment],
        leaves: &[super::super::TreeLeaf],
        lod: u8,
    ) -> Result<PreparedTreeLodBake> {
        PreparedTreeLodBake::from_runtime(
            &bake_tree_lod_with_style(
                specimen.seed,
                branches,
                leaves,
                lod,
                specimen.species.bake_style(),
            ),
            PreparedTreeImpostorUsage::Playable,
            specimen.species,
        )
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
    let report = serde_json::json!({"format": "production-cache-adaptive-v2", "content_id": decoded.content_id(), "source_id": source_word,
        "species": payload.binding.species, "seed": payload.binding.seed.to_u64(), "cache_key": payload.binding.cache_key.to_u64(),
        "input_digest": input.digest()?, "environment": environment, "compressed_bytes": bytes.len(), "payload_bytes": encoded.len(), "decode_validation_allocation_bound_bytes": checked_allocation_bound(encoded.len() as u64, bytes.len() as u64)?,
        "source_frame": "collider-centred metres; placement adds half-trunk normalization once", "capability": "complete five-tier production closure, cambered and thinned flat leaves", "recipe_version": RECIPE_VERSION, "groups": payload.groups.len(),
        "detailed_trunk_vertices": payload.detailed_trunk.vertices(), "mid_trunk_vertices": payload.mid_trunk.vertices(),
        "group_vertices": payload.groups.iter().map(|group| serde_json::json!({"group":group.primary_group,"wood":group.wood.vertices(),"buds":group.buds.vertices(),"leaves":group.leaves.vertices(),"flat_leaves":group.flat_leaves.vertices()})).collect::<Vec<_>>(),
        "aggregate_vertices": payload.aggregate_wood.iter().map(mesh::PreparedMesh::vertices).collect::<Vec<_>>(), "cards": payload.cards.iter().map(PreparedTreeLodBake::summary).collect::<Vec<_>>(),
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
        serde_json::json!({"content_id": checked.content_id(), "source_id":source_word,"complete_tiers_admitted": true,"scope":"CPU payload/source admission only"})
    );
    Ok(())
}

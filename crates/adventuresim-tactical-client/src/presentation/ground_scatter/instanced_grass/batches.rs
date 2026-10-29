//! Construct chunk-instanced batches from the procedural tuft lattice.
use super::*;
use fabelgeist_determinism::Seed;

use crate::presentation::chunk_instancing::{ChunkSize, InstanceBatch};

/// Ground cells of the dense near tiers, which fade out within ~20 m.
const NEAR_CHUNK_SIZE: ChunkSize = ChunkSize(16.0);
/// Ground cells of the sparse far tiers, which reach ~70 m.
const FAR_CHUNK_SIZE: ChunkSize = ChunkSize(32.0);

/// Turns filled instance batches into chunk entities per (tier, species),
/// each carrying `marker` on top of the chunk-instanced draw components.
pub(in crate::presentation) fn spawn_tuft_batches(
    world: GrassWorld<'_, '_, '_>,
    batches: &mut TierSpeciesBatches,
    label: &str,
    marker: impl Bundle + Clone,
    base_seed: Seed,
    pigment: TuftPigment,
    grass: &crate::presentation::config::GrassConfig,
) {
    let GrassWorld {
        commands,
        meshes,
        materials,
    } = world;
    for lod in TIERS {
        let material = materials.add(diagnostics::material(
            lod,
            grass,
            pigment.density,
            pigment.dryness,
            pigment.wind_scale,
        ));
        for species in GrassSpecies::ALL {
            let instances = std::mem::take(&mut batches[lod.tier_index()][species.index()]);
            if instances.is_empty() {
                continue;
            }
            let (mesh, triangle_count) = diagnostics::add_mesh(
                meshes,
                grass_tuft_mesh(
                    pigment.color,
                    lod,
                    pigment.density,
                    species,
                    streams::TUFT_MESH.seed(
                        base_seed,
                        &[species.index() as u64, lod.tier_index() as u64],
                    ),
                    grass,
                ),
            );
            InstanceBatch {
                mesh,
                material: material.clone(),
                instances,
                band: configured_tier_band(lod, grass),
                reach: ground_scatter_reach(configured_tuft_footprint_metres(lod, grass)),
                chunk_size: match lod {
                    GrassMeshLod::Near | GrassMeshLod::NearEdge => NEAR_CHUNK_SIZE,
                    GrassMeshLod::Far | GrassMeshLod::Vista => FAR_CHUNK_SIZE,
                },
            }
            .spawn(commands, |chunk| {
                (
                    Name::new(format!(
                        "{label} {species:?} {lod:?} tufts ({})",
                        chunk.len()
                    )),
                    GroundScatterLayer::Grass,
                    triangle_count,
                    marker.clone(),
                )
            });
        }
    }
}

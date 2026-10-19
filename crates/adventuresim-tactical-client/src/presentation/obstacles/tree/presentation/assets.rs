//! Demand construction and asset-store insertion for shared tree variants.

use super::*;

pub(super) struct TreeMaterialBindings {
    pub(super) bark: Handle<TacticalTreeBarkMaterial>,
    pub(super) aggregate_bark: Handle<TacticalTreeAggregateBarkMaterial>,
    pub(super) leaf: Handle<TacticalTreeLeafCardMaterial>,
    pub(super) bud: Handle<StandardMaterial>,
}

impl CachedTreePresentation {
    /// Existing procedural kernel outputs retain their collider-centred frame.
    pub(super) fn from_generated(
        species: TreePresentationSpecies,
        variant_seed: Seed,
        branches: Vec<super::super::TreeBranchSegment>,
        leaves: Vec<super::super::TreeLeaf>,
        materials: TreeMaterialBindings,
    ) -> Self {
        let resident_assets = TreeAssetResidencyDiagnostics {
            variants: 1,
            source_branches: branches.len(),
            source_leaves: leaves.len(),
            ..default()
        };
        Self {
            origin: TreeAssetOrigin::Generated,
            variant_seed,
            species,
            species_name: species.name(),
            branches: branches.into(),
            leaves: leaves.into(),
            trunk_mesh: None,
            mid_trunk_mesh: None,
            cluster_layout: None,
            clusters: None,
            aggregate_branch_meshes: [None, None],
            lod_cards: [None, None, None, None],
            bark_material: materials.bark,
            aggregate_bark_material: materials.aggregate_bark,
            leaf_material: materials.leaf,
            bud_material: materials.bud,
            resident_assets,
        }
    }
}

fn bake_tree_card_for_cached(
    cached: &CachedTreePresentation,
    lod: u8,
    prepared: Option<&PreparedTreeImpostorAsset>,
) -> TreeLodBake {
    if let Some(prepared) = prepared {
        return prepared
            .resolve(
                PreparedTreeImpostorUsage::Playable,
                cached.species,
                cached.variant_seed,
                lod,
            )
            .unwrap_or_else(|| {
                panic!("prepared tree impostors do not match the requested playable tree")
            });
    }
    if cached.species == TreePresentationSpecies::EnglishOak {
        bake_tree_lod(cached.variant_seed, &cached.branches, &cached.leaves, lod)
    } else {
        bake_tree_lod_with_style(
            cached.variant_seed,
            &cached.branches,
            &cached.leaves,
            lod,
            cached.species.bake_style(),
        )
    }
}

fn ensure_tree_card_resident(
    cached: &mut CachedTreePresentation,
    lod: u8,
    prepared: Option<&PreparedTreeImpostorAsset>,
    meshes: &mut Assets<Mesh>,
    tree_materials: &mut Assets<TacticalTreeImpostorMaterial>,
    images: &mut Assets<Image>,
    diagnostics: &mut TreeAssetResidencyDiagnostics,
) {
    let index = lod as usize - 1;
    if cached.lod_cards[index].is_some() {
        return;
    }
    let bake = bake_tree_card_for_cached(cached, lod, prepared);
    validate_tree_bake_provenance(&bake.provenance);
    if lod == 1 {
        cached.cluster_layout = Some(
            bake.clusters
                .iter()
                .map(|cluster| (cluster.primary_group, cluster.center, cluster.radius))
                .collect(),
        );
    }
    let vertex_count = bake.mesh.count_vertices();
    if lod == 1 {
        diagnostics.lod1_impostor_cards += bake.provenance.records.len();
        diagnostics.lod1_impostor_vertices += vertex_count;
    }
    diagnostics.impostor_vertices += vertex_count;
    let texture_bytes = bake.image.data.as_ref().map_or(0, |pixels| pixels.len());
    diagnostics.impostor_texture_bytes_by_lod[index] += texture_bytes;
    diagnostics.impostor_texture_bytes += texture_bytes;
    diagnostics.generated_lod_mask |= 1 << (lod + 1);
    let texture = images.add(bake.image);
    cached.lod_cards[index] = Some(CachedTreeCardPresentation {
        mesh: meshes.add(bake.mesh),
        material: tree_materials.add(tree_impostor_material(
            cached.variant_seed,
            bake.lod,
            texture,
        )),
        provenance: bake.provenance,
    });
}

fn ensure_detailed_tree_assets_resident(
    cached: &mut CachedTreePresentation,
    selected_leaf: Option<TreeLeafRepresentation>,
    meshes: &mut Assets<Mesh>,
    diagnostics: &mut TreeAssetResidencyDiagnostics,
) {
    if cached.clusters.is_none() {
        cached.clusters = Some(
            cached
                .cluster_layout
                .as_ref()
                .expect("LOD1 bake supplies detailed crown cluster bounds")
                .iter()
                .map(
                    |&(primary_group, center, radius)| CachedTreeClusterPresentation {
                        primary_group,
                        center,
                        radius,
                        detailed_branch_mesh: None,
                        cambered_leaf_mesh: None,
                        leaf_card_mesh: None,
                        bud_mesh: None,
                    },
                )
                .collect(),
        );
    }
    for cluster in cached
        .clusters
        .as_mut()
        .expect("detailed crown cache was initialized")
    {
        if cluster.detailed_branch_mesh.is_none() {
            let mesh =
                procedural_tree_branch_group_mesh(&cached.branches, 3, cluster.primary_group);
            diagnostics.detailed_branch_vertices += mesh.count_vertices();
            cluster.detailed_branch_mesh = Some(meshes.add(mesh));
        }
        if cluster.bud_mesh.is_none() {
            let mesh = procedural_oak_bud_group_mesh(&cached.branches, cluster.primary_group);
            diagnostics.bud_vertices += mesh.count_vertices();
            cluster.bud_mesh = Some(meshes.add(mesh));
        }
        if selected_leaf != Some(TreeLeafRepresentation::AlphaCard)
            && cluster.cambered_leaf_mesh.is_none()
        {
            let mesh =
                procedural_oak_textured_leaf_group_mesh(&cached.leaves, cluster.primary_group);
            diagnostics.cambered_leaf_vertices += mesh.count_vertices();
            cluster.cambered_leaf_mesh = Some(meshes.add(mesh));
        }
        if selected_leaf != Some(TreeLeafRepresentation::TexturedMesh)
            && cluster.leaf_card_mesh.is_none()
        {
            let mesh = procedural_oak_leaf_card_group_mesh(&cached.leaves, cluster.primary_group);
            diagnostics.leaf_card_source_leaves += cached
                .leaves
                .iter()
                .filter(|leaf| leaf.primary_group == cluster.primary_group)
                .count();
            diagnostics.leaf_card_retained_leaves += mesh.count_vertices() / 4;
            diagnostics.leaf_card_vertices += mesh.count_vertices();
            cluster.leaf_card_mesh = Some(meshes.add(mesh));
        }
    }
    diagnostics.generated_lod_mask |= 1 << 1;
}

#[expect(
    clippy::too_many_arguments,
    reason = "tree streaming updates independently borrowed asset stores and accounting"
)]
pub(super) fn ensure_tree_assets_resident(
    cached: &mut CachedTreePresentation,
    mask: u8,
    selected_leaf: Option<TreeLeafRepresentation>,
    prepared: Option<&PreparedTreeImpostorAsset>,
    meshes: &mut Assets<Mesh>,
    tree_materials: &mut Assets<TacticalTreeImpostorMaterial>,
    images: &mut Assets<Image>,
    diagnostics: &mut TreeAssetResidencyDiagnostics,
) -> bevy::prelude::Result {
    #[cfg(all(feature = "foliage-review", not(target_family = "wasm")))]
    if let TreeAssetOrigin::PreparedFull(_) = &cached.origin {
        return prepared::require_full(cached, mask, selected_leaf).map_err(Into::into);
    }
    let mut added = TreeAssetResidencyDiagnostics::default();
    generate_tree_assets_resident(
        cached,
        mask,
        selected_leaf,
        prepared,
        meshes,
        tree_materials,
        images,
        &mut added,
    );
    cached.resident_assets.add_resident_assets(&added);
    diagnostics.add_resident_assets(&added);
    diagnostics.generation_milliseconds += added.generation_milliseconds;
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "tree residency updates independently borrowed mesh, material, image, and diagnostics stores"
)]
fn generate_tree_assets_resident(
    cached: &mut CachedTreePresentation,
    mask: u8,
    selected_leaf: Option<TreeLeafRepresentation>,
    prepared: Option<&PreparedTreeImpostorAsset>,
    meshes: &mut Assets<Mesh>,
    tree_materials: &mut Assets<TacticalTreeImpostorMaterial>,
    images: &mut Assets<Image>,
    diagnostics: &mut TreeAssetResidencyDiagnostics,
) {
    let started = web_time::Instant::now();
    if mask & 1 != 0 && cached.trunk_mesh.is_none() {
        let mesh = procedural_tree_branch_mesh(&cached.branches, 0);
        diagnostics.detailed_trunk_vertices += mesh.count_vertices();
        cached.trunk_mesh = Some(meshes.add(mesh));
    }
    if mask & 1 != 0 && cached.mid_trunk_mesh.is_none() {
        let mesh = procedural_woody_mid_trunk_mesh(&cached.branches);
        diagnostics.mid_trunk_vertices += mesh.count_vertices();
        cached.mid_trunk_mesh = Some(meshes.add(mesh));
    }
    if mask & 1 != 0 {
        diagnostics.generated_lod_mask |= 1;
    }
    if mask & (1 << 1) != 0 {
        ensure_tree_card_resident(
            cached,
            1,
            prepared,
            meshes,
            tree_materials,
            images,
            diagnostics,
        );
        ensure_detailed_tree_assets_resident(cached, selected_leaf, meshes, diagnostics);
    }
    for lod in 1..=4 {
        if mask & (1 << (lod + 1)) == 0 {
            continue;
        }
        ensure_tree_card_resident(
            cached,
            lod,
            prepared,
            meshes,
            tree_materials,
            images,
            diagnostics,
        );
        if lod <= 2 && cached.aggregate_branch_meshes[lod as usize - 1].is_none() {
            let (depth, quality) = if lod == 1 {
                (2, WoodyBranchMeshQuality::AggregateLod1)
            } else {
                (1, WoodyBranchMeshQuality::AggregateLod2)
            };
            let mesh = procedural_woody_crown_mesh(&cached.branches, depth, quality);
            let aggregate_index = lod as usize - 1;
            let vertices = mesh.count_vertices();
            let triangles = mesh
                .indices()
                .expect("aggregate branch mesh is indexed")
                .len()
                / 3;
            diagnostics.aggregate_branch_vertices += vertices;
            diagnostics.aggregate_branch_vertices_by_lod[aggregate_index] += vertices;
            diagnostics.aggregate_branch_triangles_by_lod[aggregate_index] += triangles;
            cached.aggregate_branch_meshes[lod as usize - 1] = Some(meshes.add(mesh));
        }
    }
    let elapsed = started.elapsed().as_millis();
    diagnostics.generation_milliseconds += elapsed;
    if elapsed > 0 {
        info!(
            species = cached.species_name,
            requested_mask = mask,
            elapsed_ms = elapsed,
            resident_mask =
                cached.resident_assets.generated_lod_mask | diagnostics.generated_lod_mask,
            "Generated demand-driven tactical tree assets"
        );
    }
}

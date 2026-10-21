//! Complete decoded variant insertion into existing production asset stores.
use super::*;

#[derive(SystemParam)]
pub(super) struct AssetStores<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    bark: ResMut<'w, Assets<TacticalTreeBarkMaterial>>,
    aggregate_bark: ResMut<'w, Assets<TacticalTreeAggregateBarkMaterial>>,
    leaves: ResMut<'w, Assets<TacticalTreeLeafCardMaterial>>,
    pub(super) cards: ResMut<'w, Assets<TacticalTreeImpostorMaterial>>,
    pub(super) images: ResMut<'w, Assets<Image>>,
    textures: Res<'w, ProceduralTextureAssets>,
}

pub(super) struct VariantAdmission {
    pub(super) key: Seed,
    pub(super) assets_added: bool,
}

impl AssetStores<'_> {
    pub(super) fn verify_parts(
        &self,
        parts: &[InstalledTreePart],
    ) -> Result<std::collections::HashSet<AssetId<Image>>> {
        dependencies::PartDependencies {
            meshes: &self.meshes,
            materials: &self.materials,
            bark: &self.bark,
            aggregate_bark: &self.aggregate_bark,
            leaves: &self.leaves,
            cards: &self.cards,
            images: &self.images,
        }
        .verify(parts)
    }

    fn bind_materials(
        &mut self,
        cache: &mut RecipeTreeCache,
        terrain: &SceneTerrain,
        species: TreePresentationSpecies,
    ) -> Result<super::super::assets::TreeMaterialBindings> {
        let heightmap = cache.terrain_heightmap.clone();
        let bark =
            cache.bark_for_terrain(species, &self.textures, heightmap, terrain, &mut self.bark);
        let aggregate_bark = match species {
            TreePresentationSpecies::EnglishOak => {
                self.aggregate_bark.add(oak_aggregate_bark_material())
            }
            TreePresentationSpecies::CommonBeech => {
                self.aggregate_bark.add(beech_aggregate_bark_material())
            }
        };
        let leaf = self.leaves.add(match species {
            TreePresentationSpecies::EnglishOak => oak_leaf_material(&self.textures),
            TreePresentationSpecies::CommonBeech => beech_leaf_material(&self.textures),
        });
        let buds = self
            .materials
            .add(super::super::super::materials::tree_bud_material(species));
        Ok(super::super::assets::TreeMaterialBindings {
            bark,
            aggregate_bark,
            leaf,
            bud: buds,
        })
    }
}

impl RecipeTreeCache {
    pub(super) fn generate_review_variant(
        &mut self,
        specimen: &super::super::super::authoring::specimens::SourceSpecimen,
        environment: &SceneEnvironment,
        terrain: &SceneTerrain,
        stores: &mut AssetStores,
    ) -> Result<Seed> {
        let key = super::super::super::source::playable_cache_key(
            specimen.species,
            specimen.seed,
            environment,
        );
        ensure!(
            !self.variants.contains_key(&key),
            "generated review cache key already occupied"
        );
        let materials = stores.bind_materials(self, terrain, specimen.species)?;
        let (branches, leaves) = specimen.records(environment);
        let mut cached = CachedTreePresentation::from_generated(
            specimen.species,
            specimen.seed,
            branches,
            leaves,
            materials,
        );
        // Diagnostic eager prewarming only. The ordinary streamer still creates
        // the selected child closure; these generation costs are not arrival evidence.
        let mut diagnostic = RecipeTreeAssetResidencyDiagnostics::default();
        ensure_tree_assets_resident(
            &mut cached,
            COMPLETE_STREAM_MASK,
            None,
            None,
            &mut stores.meshes,
            &mut stores.cards,
            &mut stores.images,
            &mut diagnostic,
        )
        .map_err(|error| anyhow::anyhow!("generated review residency: {error}"))?;
        require_resident(&cached, COMPLETE_STREAM_MASK)?;
        self.variants.insert(key, cached);
        Ok(key)
    }

    pub(super) fn admit_variant(
        &mut self,
        variant: PreparedVariant,
        terrain: &SceneTerrain,
        stores: &mut AssetStores,
    ) -> Result<VariantAdmission> {
        if let Some(cached) = self.variants.get(&variant.key) {
            ensure!(
                cached.origin == TreeAssetOrigin::Prepared(variant.identity.clone()),
                "prepared tree cache key has another origin/binding"
            );
            require_resident(cached, COMPLETE_STREAM_MASK)?;
            let parts = inspection::installed_parts(cached, &stores.cards)?;
            stores.verify_parts(&parts)?;
            return Ok(VariantAdmission {
                key: variant.key,
                assets_added: false,
            });
        }
        let bindings = stores.bind_materials(self, terrain, variant.species)?;
        let mut diagnostics = RecipeTreeAssetResidencyDiagnostics {
            variants: 1,
            source_branches: variant.branches.len(),
            source_leaves: variant.leaves.len(),
            detailed_trunk_vertices: variant.detailed_trunk.count_vertices(),
            mid_trunk_vertices: variant.mid_trunk.count_vertices(),
            generated_lod_mask: COMPLETE_STREAM_MASK,
            ..default()
        };
        let clusters = variant
            .groups
            .into_iter()
            .map(|group| {
                let source_leaves = variant
                    .leaves
                    .iter()
                    .filter(|leaf| leaf.primary_group == group.primary_group)
                    .count();
                CachedTreeClusterPresentation::from_prepared(
                    group,
                    source_leaves,
                    &mut stores.meshes,
                    &mut diagnostics,
                )
            })
            .collect();
        let layout = variant.cards[0]
            .clusters
            .iter()
            .map(|group| (group.primary_group, group.center, group.radius))
            .collect();
        let cards = variant
            .cards
            .map(|card| Some(stores.insert_card(card, variant.seed, &mut diagnostics)));
        let [near, middle] = variant.aggregate_wood;
        let aggregate_branch_meshes = [
            Some(stores.insert_aggregate(
                near,
                super::super::super::AggregateTreeLod::Near,
                &mut diagnostics,
            )?),
            Some(stores.insert_aggregate(
                middle,
                super::super::super::AggregateTreeLod::Middle,
                &mut diagnostics,
            )?),
        ];
        let cached = CachedTreePresentation {
            origin: TreeAssetOrigin::Prepared(variant.identity),
            variant_seed: variant.seed,
            species: variant.species,
            species_name: variant.species.name(),
            branches: variant.branches,
            leaves: variant.leaves,
            trunk_mesh: Some(stores.meshes.add(variant.detailed_trunk)),
            mid_trunk_mesh: Some(stores.meshes.add(variant.mid_trunk)),
            cluster_layout: Some(layout),
            clusters: Some(clusters),
            aggregate_branch_meshes,
            lod_cards: cards,
            bark_material: bindings.bark,
            aggregate_bark_material: bindings.aggregate_bark,
            leaf_material: bindings.leaf,
            bud_material: bindings.bud,
            resident_assets: diagnostics,
        };
        require_resident(&cached, COMPLETE_STREAM_MASK)?;
        self.variants.insert(variant.key, cached);
        Ok(VariantAdmission {
            key: variant.key,
            assets_added: true,
        })
    }
}

impl AssetStores<'_> {
    fn insert_aggregate(
        &mut self,
        mesh: Mesh,
        lod: super::super::super::AggregateTreeLod,
        diagnostics: &mut RecipeTreeAssetResidencyDiagnostics,
    ) -> Result<Handle<Mesh>> {
        let index = lod.index();
        let vertices = mesh.count_vertices();
        let triangles = mesh.indices().context("admitted aggregate indices")?.len() / 3;
        diagnostics.aggregate_branch_vertices += vertices;
        diagnostics.aggregate_branch_vertices_by_lod[index] += vertices;
        diagnostics.aggregate_branch_triangles_by_lod[index] += triangles;
        Ok(self.meshes.add(mesh))
    }

    fn insert_card(
        &mut self,
        card: TreeLodBake,
        seed: Seed,
        diagnostics: &mut RecipeTreeAssetResidencyDiagnostics,
    ) -> CachedTreeCardPresentation {
        let index = card.lod as usize - 1;
        let vertices = card.mesh.count_vertices();
        if card.lod == 1 {
            diagnostics.lod1_impostor_cards = card.provenance.records.len();
            diagnostics.lod1_impostor_vertices = vertices;
        }
        diagnostics.impostor_vertices += vertices;
        let bytes = card.image.data.as_ref().map_or(0, Vec::len);
        diagnostics.impostor_texture_bytes += bytes;
        diagnostics.impostor_texture_bytes_by_lod[index] = bytes;
        let texture = self.images.add(card.image);
        CachedTreeCardPresentation {
            mesh: self.meshes.add(card.mesh),
            material: self
                .cards
                .add(tree_impostor_material(seed, card.lod, texture)),
            provenance: card.provenance,
        }
    }
}

impl CachedTreeClusterPresentation {
    fn from_prepared(
        group: PreparedCluster,
        source_leaves: usize,
        meshes: &mut Assets<Mesh>,
        diagnostics: &mut RecipeTreeAssetResidencyDiagnostics,
    ) -> Self {
        diagnostics.detailed_branch_vertices += group.wood.count_vertices();
        diagnostics.bud_vertices += group.buds.count_vertices();
        diagnostics.cambered_leaf_vertices += group.leaves.count_vertices();
        diagnostics.leaf_card_source_leaves += source_leaves;
        diagnostics.leaf_card_retained_leaves += group.flat_leaves.count_vertices() / 4;
        diagnostics.leaf_card_vertices += group.flat_leaves.count_vertices();
        Self {
            primary_group: group.primary_group,
            center: group.center,
            radius: group.radius,
            detailed_branch_mesh: Some(meshes.add(group.wood)),
            bud_mesh: Some(meshes.add(group.buds)),
            cambered_leaf_mesh: Some(meshes.add(group.leaves)),
            leaf_card_mesh: Some(meshes.add(group.flat_leaves)),
        }
    }
}

pub(in super::super) fn require_resident(cached: &CachedTreePresentation, mask: u8) -> Result<()> {
    ensure!(
        mask & !COMPLETE_STREAM_MASK == 0,
        "unsupported prepared stream mask"
    );
    ensure!(
        cached.trunk_mesh.is_some()
            && cached.mid_trunk_mesh.is_some()
            && cached.lod_cards.iter().all(Option::is_some)
            && cached.aggregate_branch_meshes.iter().all(Option::is_some),
        "prepared tree tier dependency missing"
    );
    let clusters = cached
        .clusters
        .as_ref()
        .context("prepared crown groups missing")?;
    ensure!(
        !clusters.is_empty()
            && clusters
                .iter()
                .all(|group| group.detailed_branch_mesh.is_some()
                    && group.bud_mesh.is_some()
                    && group.cambered_leaf_mesh.is_some()
                    && group.leaf_card_mesh.is_some()),
        "prepared tree group dependency missing"
    );
    Ok(())
}

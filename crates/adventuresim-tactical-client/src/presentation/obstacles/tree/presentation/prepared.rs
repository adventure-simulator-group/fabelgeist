//! Admit immutable Full assets through the existing production cache hierarchy.
use super::super::prepared_cache::{FullCluster, FullVariant, PreparedTreeCache};
use super::*;
use anyhow::{Context, Result, ensure};
use bevy::ecs::system::{SystemParam, SystemState};

const FULL_STREAM_MASK: u8 = 0b11;
const FULL_RESIDENT_MASK: u8 = 0b111;

#[derive(SystemParam)]
struct AssetStores<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    bark: ResMut<'w, Assets<TacticalTreeBarkMaterial>>,
    aggregate_bark: ResMut<'w, Assets<TacticalTreeAggregateBarkMaterial>>,
    leaves: ResMut<'w, Assets<TacticalTreeLeafCardMaterial>>,
    cards: ResMut<'w, Assets<TacticalTreeImpostorMaterial>>,
    images: ResMut<'w, Assets<Image>>,
    textures: Res<'w, ProceduralTextureAssets>,
}

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
    MidTrunkDependency,
    Wood { primary_group: u8 },
    Buds { primary_group: u8 },
    CamberedLeaves { primary_group: u8 },
    CardDependency,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InstalledTreeMaterial {
    Bark(AssetId<TacticalTreeBarkMaterial>),
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

impl AssetStores<'_> {
    fn bind_materials(
        &mut self,
        cache: &mut TreePresentationCache,
        terrain: &SceneTerrain,
        species: TreePresentationSpecies,
    ) -> Result<assets::TreeMaterialBindings> {
        let heightmap = cache
            .terrain_heightmap
            .clone()
            .context("active destination tree heightmap")?;
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
            .add(super::super::materials::tree_bud_material(species));
        Ok(assets::TreeMaterialBindings {
            bark,
            aggregate_bark,
            leaf,
            bud: buds,
        })
    }
}

impl TreePresentationCache {
    fn admit_variant(
        &mut self,
        variant: FullVariant,
        terrain: &SceneTerrain,
        stores: &mut AssetStores,
    ) -> Result<Seed> {
        ensure!(
            !self.variants.contains_key(&variant.key),
            "prepared tree cache key already occupied"
        );
        let bindings = stores.bind_materials(self, terrain, variant.species)?;
        let mut diagnostics = TreeAssetResidencyDiagnostics {
            variants: 1,
            source_branches: variant.branches.len(),
            source_leaves: variant.leaves.len(),
            detailed_trunk_vertices: variant.detailed_trunk.count_vertices(),
            mid_trunk_vertices: variant.mid_trunk.count_vertices(),
            generated_lod_mask: FULL_RESIDENT_MASK,
            ..default()
        };
        let clusters = variant
            .groups
            .into_iter()
            .map(|group| {
                CachedTreeClusterPresentation::from_prepared(
                    group,
                    &mut stores.meshes,
                    &mut diagnostics,
                )
            })
            .collect();
        let card = variant.card;
        let layout = card
            .clusters
            .iter()
            .map(|group| (group.primary_group, group.center, group.radius))
            .collect();
        diagnostics.lod1_impostor_cards = card.provenance.records.len();
        diagnostics.lod1_impostor_vertices = card.mesh.count_vertices();
        diagnostics.impostor_vertices = diagnostics.lod1_impostor_vertices;
        diagnostics.impostor_texture_bytes = card.image.data.as_ref().map_or(0, Vec::len);
        diagnostics.impostor_texture_bytes_by_lod[0] = diagnostics.impostor_texture_bytes;
        let texture = stores.images.add(card.image);
        let cached = CachedTreePresentation {
            origin: TreeAssetOrigin::PreparedFull(variant.content),
            variant_seed: variant.seed,
            species: variant.species,
            species_name: variant.species.name(),
            branches: variant.branches,
            leaves: variant.leaves,
            trunk_mesh: Some(stores.meshes.add(variant.detailed_trunk)),
            mid_trunk_mesh: Some(stores.meshes.add(variant.mid_trunk)),
            cluster_layout: Some(layout),
            clusters: Some(clusters),
            aggregate_branch_meshes: [None, None],
            lod_cards: [
                Some(CachedTreeCardPresentation {
                    mesh: stores.meshes.add(card.mesh),
                    material: stores
                        .cards
                        .add(tree_impostor_material(variant.seed, 1, texture)),
                    provenance: card.provenance,
                }),
                None,
                None,
                None,
            ],
            bark_material: bindings.bark,
            aggregate_bark_material: bindings.aggregate_bark,
            leaf_material: bindings.leaf,
            bud_material: bindings.bud,
            resident_assets: diagnostics,
        };
        require_full(
            &cached,
            FULL_STREAM_MASK,
            Some(TreeLeafRepresentation::TexturedMesh),
        )?;
        self.variants.insert(variant.key, cached);
        Ok(variant.key)
    }
}

impl CachedTreeClusterPresentation {
    fn from_prepared(
        group: FullCluster,
        meshes: &mut Assets<Mesh>,
        diagnostics: &mut TreeAssetResidencyDiagnostics,
    ) -> Self {
        diagnostics.detailed_branch_vertices += group.wood.count_vertices();
        diagnostics.bud_vertices += group.buds.count_vertices();
        diagnostics.cambered_leaf_vertices += group.leaves.count_vertices();
        Self {
            primary_group: group.primary_group,
            center: group.center,
            radius: group.radius,
            detailed_branch_mesh: Some(meshes.add(group.wood)),
            bud_mesh: Some(meshes.add(group.buds)),
            cambered_leaf_mesh: Some(meshes.add(group.leaves)),
            leaf_card_mesh: None,
        }
    }
}

pub(super) fn require_full(
    cached: &CachedTreePresentation,
    mask: u8,
    leaf: Option<TreeLeafRepresentation>,
) -> Result<()> {
    ensure!(
        mask == FULL_STREAM_MASK && leaf == Some(TreeLeafRepresentation::TexturedMesh),
        "prepared tree supports fixed Full/cambered only"
    );
    ensure!(
        cached.trunk_mesh.is_some()
            && cached.mid_trunk_mesh.is_some()
            && cached.lod_cards[0].is_some(),
        "prepared Full dependencies missing"
    );
    let clusters = cached
        .clusters
        .as_ref()
        .context("prepared Full groups missing")?;
    ensure!(
        !clusters.is_empty()
            && clusters
                .iter()
                .all(|group| group.detailed_branch_mesh.is_some()
                    && group.bud_mesh.is_some()
                    && group.cambered_leaf_mesh.is_some()),
        "prepared Full group parts missing"
    );
    Ok(())
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
                matches!(&installed.origin, TreeAssetOrigin::PreparedFull(content) if *content == content_id),
                "installed prepared tree content identity"
            );
            let parts = installed_parts(installed, &stores.cards)?;
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

fn installed_parts(
    cached: &CachedTreePresentation,
    cards: &Assets<TacticalTreeImpostorMaterial>,
) -> Result<Vec<InstalledTreePart>> {
    let mut parts = vec![
        InstalledTreePart {
            role: InstalledTreePartRole::DetailedTrunk,
            mesh: cached
                .trunk_mesh
                .as_ref()
                .context("installed detailed trunk")?
                .id(),
            material: InstalledTreeMaterial::Bark(cached.bark_material.id()),
        },
        InstalledTreePart {
            role: InstalledTreePartRole::MidTrunkDependency,
            mesh: cached
                .mid_trunk_mesh
                .as_ref()
                .context("installed mid trunk")?
                .id(),
            material: InstalledTreeMaterial::Bark(cached.bark_material.id()),
        },
    ];
    let groups = cached.clusters.as_ref().context("installed groups")?;
    for group in groups {
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::Wood {
                primary_group: group.primary_group,
            },
            mesh: group
                .detailed_branch_mesh
                .as_ref()
                .context("installed crown wood")?
                .id(),
            material: InstalledTreeMaterial::Bark(cached.bark_material.id()),
        });
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::Buds {
                primary_group: group.primary_group,
            },
            mesh: group.bud_mesh.as_ref().context("installed buds")?.id(),
            material: InstalledTreeMaterial::Buds(cached.bud_material.id()),
        });
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::CamberedLeaves {
                primary_group: group.primary_group,
            },
            mesh: group
                .cambered_leaf_mesh
                .as_ref()
                .context("installed cambered leaves")?
                .id(),
            material: InstalledTreeMaterial::Leaves(cached.leaf_material.id()),
        });
    }
    let card = cached.lod_cards[0]
        .as_ref()
        .context("installed card dependency")?;
    let material = cards
        .get(&card.material)
        .context("installed card material")?;
    parts.push(InstalledTreePart {
        role: InstalledTreePartRole::CardDependency,
        mesh: card.mesh.id(),
        material: InstalledTreeMaterial::Card {
            material: card.material.id().untyped(),
            image: material.baked_color.id(),
        },
    });
    Ok(parts)
}

/// Read-only cache inspection, including the actual screenshot observer.
#[derive(SystemParam)]
pub(crate) struct PreparedCacheInspection<'w, 's> {
    cache: Res<'w, TreePresentationCache>,
    streams: Query<'w, 's, &'static StreamedTreePresentation>,
    meshes: Res<'w, Assets<Mesh>>,
    cards: Res<'w, Assets<TacticalTreeImpostorMaterial>>,
    images: Res<'w, Assets<Image>>,
    bark: Res<'w, Assets<TacticalTreeBarkMaterial>>,
    leaves: Res<'w, Assets<TacticalTreeLeafCardMaterial>>,
    buds: Res<'w, Assets<StandardMaterial>>,
}

impl PreparedCacheInspection<'_, '_> {
    pub(crate) fn verify(&self, installed: &InstalledCacheSpecimen) -> Result<()> {
        let stream = self.streams.get(installed.root)?;
        installed.verify_records(&self.cache, &self.cards, stream)?;
        for part in &installed.parts {
            ensure!(
                self.meshes.contains(part.mesh),
                "retained prepared mesh missing"
            );
            let present = match part.material {
                InstalledTreeMaterial::Bark(id) => self.bark.contains(id),
                InstalledTreeMaterial::Leaves(id) => self.leaves.contains(id),
                InstalledTreeMaterial::Buds(id) => self.buds.contains(id),
                InstalledTreeMaterial::Card { material, image } => {
                    self.cards
                        .contains(material.try_typed::<TacticalTreeImpostorMaterial>()?)
                        && self.images.contains(image)
                }
            };
            ensure!(present, "retained prepared material/image missing");
        }
        Ok(())
    }
}

impl InstalledCacheSpecimen {
    /// Inspect the actual retained cache and streamer without generating assets.
    fn verify_records(
        &self,
        cache: &TreePresentationCache,
        cards: &Assets<TacticalTreeImpostorMaterial>,
        stream: &StreamedTreePresentation,
    ) -> Result<()> {
        let cached = cache
            .variants
            .get(&self.cache_key)
            .context("retained prepared entry")?;
        ensure!(
            matches!(cached.origin, TreeAssetOrigin::PreparedFull(content) if content == self.content_id),
            "prepared origin/content drift"
        );
        require_full(
            cached,
            FULL_STREAM_MASK,
            Some(TreeLeafRepresentation::TexturedMesh),
        )?;
        ensure!(
            installed_parts(cached, cards)? == self.parts,
            "retained cache part binding drift"
        );
        ensure!(
            stream.cache_key == self.cache_key
                && stream.resident_mask == FULL_STREAM_MASK
                && stream.active_mask == FULL_STREAM_MASK
                && stream.resident_leaf_mask == 1
                && stream.active_leaf == Some(TreeLeafRepresentation::TexturedMesh),
            "actual cache root is not fixed Full/cambered"
        );
        Ok(())
    }
}

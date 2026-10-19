use super::impostor::{
    BEECH_TREE_BAKE_STYLE, OAK_TREE_BAKE_STYLE, TREE_LEAF_HANDOFF_END, TREE_LEAF_HANDOFF_START,
    TreeImpostorProvenance, TreeLodBake, bake_tree_lod, bake_tree_lod_with_style,
    tree_impostor_material, tree_leaf_visibility, tree_lod_name, tree_lod_visibility,
    tree_mid_trunk_visibility, tree_trunk_visibility, validate_tree_bake_provenance,
};
use super::source::{
    TreePresentationSpecies, canopy_competition, oak_gnarling_for_site, playable_cache_key,
    playable_tree_source, tree_species_for_site, vista_tree_source,
};
use super::{
    COMMON_BEECH_PARAMETERS, OAK_GNARLING_SHOWCASE, OakGnarlingParameters,
    PlayableTreeAggregateWood, PlayableTreeBuds, PlayableTreeCanopyCard,
    PlayableTreeDetailedLeaves, PlayableTreeDetailedTrunk, PlayableTreeDetailedWood,
    PlayableTreeMidTrunk, PlayableTreeTrunk, PreparedTreeImpostorAsset, PreparedTreeImpostorUsage,
    PreparedTreeImpostors, TacticalTreeAggregateBarkMaterial, TacticalTreeBarkMaterial,
    TacticalTreeImpostorMaterial, TacticalTreeLeafCardMaterial, TreeLeafRepresentation, TreeLod,
    TreeLodCluster, TreeLodRenderOverride, TreeTrunkLod, WoodyBranchMeshQuality,
    beech_aggregate_bark_material, beech_bark_material, beech_leaf_material,
    oak_aggregate_bark_material, oak_bark_material, oak_leaf_material,
    procedural_oak_bud_group_mesh, procedural_oak_leaf_card_group_mesh, procedural_oak_leaves,
    procedural_oak_skeleton_with_gnarling, procedural_oak_textured_leaf_group_mesh,
    procedural_tree_branch_group_mesh, procedural_tree_branch_mesh, procedural_tree_skeleton,
    procedural_woody_crown_mesh, procedural_woody_mid_trunk_mesh, procedural_woody_plant_leaves,
    procedural_woody_plant_skeleton,
};
use crate::presentation::TacticalGameplayCamera;
use crate::presentation::{
    ActiveTacticalScene, ActiveVistaSurface, ProceduralTextureAssets, SceneEnvironment,
};
use adventuresim_tactical_core::prelude::SceneTerrain;
use bevy::{
    camera::{primitives::Aabb, visibility::NoFrustumCulling},
    light::NotShadowCaster,
    prelude::*,
};
use fabelgeist_determinism::Seed;
use fabelgeist_determinism::StreamId;

pub(crate) use diagnostics::TreeAssetResidencyDiagnostics;
pub(in crate::presentation) use residency::RetainedTreePresentations;

#[cfg(test)]
use super::{TREE_PRIMARY_GROUP_COUNT, source::oak_site_key};

mod assets;
mod diagnostics;
#[cfg(all(feature = "foliage-review", not(target_family = "wasm")))]
pub(super) mod prepared;

use assets::ensure_tree_assets_resident;
mod residency;
mod vista;
pub(in crate::presentation) use vista::ensure_vista_tree_variant;

#[derive(Component)]
pub(in crate::presentation) struct PendingTreePresentation;

/// Exact submitted triangle count for one streamed leaf-cluster entity.
/// Stored when the CPU mesh is built because production meshes intentionally
/// relinquish their main-world vertex data after render extraction.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct TreeLeafTriangleCount(pub(crate) usize);

#[derive(Resource, Default)]
pub(in crate::presentation) struct TreePresentationCache {
    variants: std::collections::HashMap<Seed, CachedTreePresentation>,
    oak_bark_material: Option<Handle<TacticalTreeBarkMaterial>>,
    beech_bark_material: Option<Handle<TacticalTreeBarkMaterial>>,
    oak_aggregate_bark_material: Option<Handle<TacticalTreeAggregateBarkMaterial>>,
    beech_aggregate_bark_material: Option<Handle<TacticalTreeAggregateBarkMaterial>>,
    terrain_scene_digest: Option<String>,
    terrain_heightmap: Option<Handle<Image>>,
    terrain_height_range: Vec2,
}

impl TreePresentationCache {
    pub(in crate::presentation) fn weather_occlusion_branches(
        &self,
        cache_key: Seed,
    ) -> Option<&[super::TreeBranchSegment]> {
        self.variants
            .get(&cache_key)
            .map(|cached| cached.branches.as_ref())
    }
}

#[derive(Resource, Default)]
pub(in crate::presentation) struct VistaTreePresentationCache {
    variants: std::collections::HashMap<Seed, CachedVistaTreePresentation>,
}

#[derive(Clone)]
pub(in crate::presentation) struct CachedVistaTreePresentation {
    pub(in crate::presentation) mesh: Handle<Mesh>,
    pub(in crate::presentation) material: Handle<TacticalTreeImpostorMaterial>,
    pub(in crate::presentation) provenance: TreeImpostorProvenance,
}

#[derive(Clone)]
struct CachedTreePresentation {
    origin: TreeAssetOrigin,
    variant_seed: Seed,
    species: TreePresentationSpecies,
    species_name: &'static str,
    branches: std::sync::Arc<[super::TreeBranchSegment]>,
    leaves: std::sync::Arc<[super::TreeLeaf]>,
    trunk_mesh: Option<Handle<Mesh>>,
    mid_trunk_mesh: Option<Handle<Mesh>>,
    cluster_layout: Option<Vec<(u8, Vec3, f32)>>,
    clusters: Option<Vec<CachedTreeClusterPresentation>>,
    aggregate_branch_meshes: [Option<Handle<Mesh>>; 2],
    lod_cards: [Option<CachedTreeCardPresentation>; 4],
    bark_material: Handle<TacticalTreeBarkMaterial>,
    aggregate_bark_material: Handle<TacticalTreeAggregateBarkMaterial>,
    leaf_material: Handle<TacticalTreeLeafCardMaterial>,
    bud_material: Handle<StandardMaterial>,
    resident_assets: TreeAssetResidencyDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TreeAssetOrigin {
    Generated,
    #[cfg(all(feature = "foliage-review", not(target_family = "wasm")))]
    Prepared(super::prepared_cache::PreparedTreeIdentity),
}

#[derive(Clone)]
struct CachedTreeClusterPresentation {
    primary_group: u8,
    center: Vec3,
    radius: f32,
    detailed_branch_mesh: Option<Handle<Mesh>>,
    cambered_leaf_mesh: Option<Handle<Mesh>>,
    leaf_card_mesh: Option<Handle<Mesh>>,
    bud_mesh: Option<Handle<Mesh>>,
}

#[derive(Clone)]
struct CachedTreeCardPresentation {
    mesh: Handle<Mesh>,
    material: Handle<TacticalTreeImpostorMaterial>,
    provenance: TreeImpostorProvenance,
}

/// Root marker for a fully presented playable tree.
///
/// Capture benchmarks use this boundary to attribute tree cost without
/// depending on display names or reaching into the tree's child layout.
#[derive(Component)]
pub(crate) struct PresentedTree;

#[derive(Component, Clone)]
pub(in crate::presentation) struct StreamedTreePresentation {
    cache_key: Seed,
    resident_mask: u8,
    resident_leaf_mask: u8,
    active_mask: u8,
    active_leaf: Option<TreeLeafRepresentation>,
}

impl StreamedTreePresentation {
    pub(in crate::presentation) fn weather_occlusion_cache_key(&self) -> Seed {
        self.cache_key
    }
}

#[derive(Component)]
pub(in crate::presentation) struct StreamedTreeChild;

fn spawn_cached_tree(
    commands: &mut Commands,
    entity: Entity,
    cache_key: Seed,
    species_name: &'static str,
) {
    commands.entity(entity).insert((
        Name::new(format!("Presented mature {species_name}")),
        PresentedTree,
        Visibility::Inherited,
        StreamedTreePresentation {
            cache_key,
            resident_mask: 0,
            resident_leaf_mask: 0,
            active_mask: u8::MAX,
            active_leaf: None,
        },
    ));
}

fn spawn_streamed_tree_children(
    commands: &mut Commands,
    entity: Entity,
    cached: &CachedTreePresentation,
    new_mask: u8,
    new_leaf_mask: u8,
) {
    commands.entity(entity).with_children(|parent| {
        // Keep renderable trunk state below the obstacle root. Hiding the trunk
        // at whole-tree LOD must not hide the camera-facing billboard sibling
        // through inherited parent visibility.
        if new_mask & 1 != 0 {
            parent.spawn((
                Name::new(format!("{} detailed trunk and roots", cached.species_name)),
                StreamedTreeChild,
                TreeTrunkLod,
                PlayableTreeTrunk,
                PlayableTreeDetailedTrunk,
                Mesh3d(
                    cached
                        .trunk_mesh
                        .clone()
                        .expect("requested trunk mesh is resident"),
                ),
                MeshMaterial3d(cached.bark_material.clone()),
                tree_trunk_visibility(),
            ));
            parent.spawn((
                Name::new(format!("{} mid-distance trunk", cached.species_name)),
                StreamedTreeChild,
                TreeTrunkLod,
                PlayableTreeTrunk,
                PlayableTreeMidTrunk,
                Mesh3d(
                    cached
                        .mid_trunk_mesh
                        .clone()
                        .expect("requested mid trunk mesh is resident"),
                ),
                MeshMaterial3d(cached.bark_material.clone()),
                tree_mid_trunk_visibility(),
            ));
        }
        if new_mask & (1 << 1) != 0 || new_leaf_mask != 0 {
            for cluster in cached
                .clusters
                .as_ref()
                .expect("requested detailed crown assets are resident")
            {
                let cluster_marker = TreeLodCluster {
                    primary_group: cluster.primary_group,
                    center: cluster.center,
                    radius: cluster.radius,
                };
                let cluster_aabb = tree_cluster_aabb(cluster.center, cluster.radius);
                let cluster_leaf_count = cached
                    .leaves
                    .iter()
                    .filter(|leaf| leaf.primary_group == cluster.primary_group)
                    .count();
                if new_leaf_mask & 1 != 0 {
                    parent.spawn((
                        Name::new(format!(
                            "{} scaffold {} cambered PBR leaves",
                            cached.species_name, cluster.primary_group
                        )),
                        StreamedTreeChild,
                        TreeLod(0),
                        cluster_marker,
                        cluster_aabb,
                        TreeLeafRepresentation::TexturedMesh,
                        PlayableTreeDetailedLeaves,
                        TreeLeafTriangleCount(cluster_leaf_count * 8),
                        Mesh3d(
                            cluster
                                .cambered_leaf_mesh
                                .clone()
                                .expect("requested cambered leaf mesh is resident"),
                        ),
                        MeshMaterial3d(cached.leaf_material.clone()),
                        tree_leaf_visibility(
                            TreeLeafRepresentation::TexturedMesh,
                            1.0,
                            cluster.radius,
                        ),
                    ));
                }
                if new_leaf_mask & 2 != 0 {
                    parent.spawn((
                        Name::new(format!(
                            "{} scaffold {} PBR leaf cards",
                            cached.species_name, cluster.primary_group
                        )),
                        StreamedTreeChild,
                        TreeLod(0),
                        cluster_marker,
                        cluster_aabb,
                        TreeLeafRepresentation::AlphaCard,
                        PlayableTreeDetailedLeaves,
                        TreeLeafTriangleCount(cluster_leaf_count * 2),
                        Mesh3d(
                            cluster
                                .leaf_card_mesh
                                .clone()
                                .expect("requested leaf card mesh is resident"),
                        ),
                        MeshMaterial3d(cached.leaf_material.clone()),
                        tree_leaf_visibility(
                            TreeLeafRepresentation::AlphaCard,
                            1.0,
                            cluster.radius,
                        ),
                    ));
                }
                if new_mask & (1 << 1) != 0 {
                    parent.spawn((
                        Name::new(format!(
                            "{} scaffold {} terminal buds",
                            cached.species_name, cluster.primary_group
                        )),
                        StreamedTreeChild,
                        TreeLod(0),
                        PlayableTreeBuds,
                        cluster_marker,
                        cluster_aabb,
                        Mesh3d(
                            cluster
                                .bud_mesh
                                .clone()
                                .expect("requested bud mesh is resident"),
                        ),
                        MeshMaterial3d(cached.bud_material.clone()),
                        tree_lod_visibility(0),
                    ));
                    parent.spawn((
                        Name::new(format!(
                            "{} scaffold {} detailed wood",
                            cached.species_name, cluster.primary_group
                        )),
                        StreamedTreeChild,
                        TreeLod(0),
                        PlayableTreeDetailedWood,
                        cluster_marker,
                        cluster_aabb,
                        Mesh3d(
                            cluster
                                .detailed_branch_mesh
                                .clone()
                                .expect("requested detailed branch mesh is resident"),
                        ),
                        MeshMaterial3d(cached.bark_material.clone()),
                        tree_lod_visibility(0),
                    ));
                }
            }
        }
        for lod in 1..=3 {
            if new_mask & (1 << (lod + 1)) == 0 {
                continue;
            }
            let card = cached.lod_cards[lod as usize - 1]
                .as_ref()
                .expect("requested aggregate tree card is resident");
            parent.spawn((
                Name::new(tree_lod_name(lod, true)),
                StreamedTreeChild,
                TreeLod(lod),
                PlayableTreeCanopyCard,
                NotShadowCaster,
                Mesh3d(card.mesh.clone()),
                MeshMaterial3d(card.material.clone()),
                tree_lod_visibility(lod),
            ));
            if lod <= 2 {
                let mut aggregate_wood = parent.spawn((
                    Name::new(tree_lod_name(lod, false)),
                    StreamedTreeChild,
                    TreeLod(lod),
                    PlayableTreeAggregateWood,
                    Mesh3d(
                        cached.aggregate_branch_meshes[lod as usize - 1]
                            .clone()
                            .expect("requested aggregate branch mesh is resident"),
                    ),
                    aggregate_tree_wood_material(cached.aggregate_bark_material.clone()),
                    tree_lod_visibility(lod),
                ));
                if !aggregate_tree_wood_casts_shadows(lod) {
                    aggregate_wood.insert(NotShadowCaster);
                }
            }
        }
        if new_mask & (1 << 5) != 0 {
            let card = cached.lod_cards[3]
                .as_ref()
                .expect("requested whole-tree card is resident");
            parent.spawn((
                Name::new(tree_lod_name(4, true)),
                StreamedTreeChild,
                TreeLod(4),
                PlayableTreeCanopyCard,
                NoFrustumCulling,
                NotShadowCaster,
                Mesh3d(card.mesh.clone()),
                MeshMaterial3d(card.material.clone()),
                card.provenance.clone(),
                tree_lod_visibility(4),
            ));
        }
    });
}

fn aggregate_tree_wood_material(
    material: Handle<TacticalTreeAggregateBarkMaterial>,
) -> MeshMaterial3d<TacticalTreeAggregateBarkMaterial> {
    MeshMaterial3d(material)
}

fn aggregate_tree_wood_casts_shadows(lod: u8) -> bool {
    lod != 2
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects tree streaming state and each independently borrowed asset store as system parameters"
)]
pub(in crate::presentation) fn stream_tree_lod_children(
    mut commands: Commands,
    camera: Single<(&GlobalTransform, &Projection), With<TacticalGameplayCamera>>,
    lod_override: Res<TreeLodRenderOverride>,
    mut trees: Query<(Entity, &GlobalTransform, &mut StreamedTreePresentation)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tree_materials: ResMut<Assets<TacticalTreeImpostorMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut tree_cache: ResMut<TreePresentationCache>,
    mut residency: ResMut<TreeAssetResidencyDiagnostics>,
    prepared: PreparedTreeImpostors,
) -> bevy::prelude::Result {
    let focal_scale = match camera.1 {
        Projection::Perspective(projection) => {
            (80.0_f32.to_radians() * 0.5).tan() / (projection.fov * 0.5).tan()
        }
        _ => 1.0,
    } * lod_override.projected_scale.unwrap_or(1.0);
    for (entity, transform, mut presentation) in &mut trees {
        let distance = camera.0.translation().distance(transform.translation());
        let scaled = distance / focal_scale.max(0.25);
        let mask = if let Some(lod) = lod_override.lod {
            (1 << (lod + 1)) | u8::from(lod < 4)
        } else {
            tree_streaming_mask(scaled)
        };
        let selected_leaf = lod_override
            .leaf
            .or_else(|| tree_streamed_leaf_representation(scaled));
        if presentation.active_mask == mask && presentation.active_leaf == selected_leaf {
            continue;
        }
        debug!(
            distance,
            scaled_distance = scaled,
            requested_mask = mask,
            ?selected_leaf,
            "Changed streamed tactical tree residency"
        );
        let cached = tree_cache
            .variants
            .get_mut(&presentation.cache_key)
            .expect("streamed tree cache entry remains resident");
        let prepared = prepared.get();
        ensure_tree_assets_resident(
            cached,
            mask,
            selected_leaf,
            prepared,
            &mut meshes,
            &mut tree_materials,
            &mut images,
            &mut residency,
        )?;
        let desired_leaf_mask = if mask & (1 << 1) == 0 {
            0
        } else {
            match selected_leaf {
                Some(TreeLeafRepresentation::TexturedMesh) => 1,
                Some(TreeLeafRepresentation::AlphaCard) => 2,
                None => 3,
            }
        };
        let new_mask = mask & !presentation.resident_mask;
        let new_leaf_mask = desired_leaf_mask & !presentation.resident_leaf_mask;
        spawn_streamed_tree_children(&mut commands, entity, cached, new_mask, new_leaf_mask);
        presentation.resident_mask |= new_mask;
        presentation.resident_leaf_mask |= new_leaf_mask;
        presentation.active_mask = mask;
        presentation.active_leaf = selected_leaf;
    }
    Ok(())
}

fn tree_streamed_leaf_representation(scaled_distance: f32) -> Option<TreeLeafRepresentation> {
    // Keep residency decisions on the same projected-scale-aware base band as
    // tree_leaf_visibility. The cluster-specific visibility range may widen
    // this overlap, but it must never request a representation after its
    // corresponding visibility range has become active.
    if scaled_distance < TREE_LEAF_HANDOFF_START {
        Some(TreeLeafRepresentation::TexturedMesh)
    } else if scaled_distance > TREE_LEAF_HANDOFF_END {
        Some(TreeLeafRepresentation::AlphaCard)
    } else {
        None
    }
}

fn tree_streaming_mask(scaled_distance: f32) -> u8 {
    // Generate each incoming crown tier one full outgoing LOD band before it
    // becomes visible. Mesh/material insertion is asynchronous in the render
    // world: the old four-metre lead could leave only the already-resident
    // trunk on a first approach, while a second approach was correct because
    // the crown assets remained cached. This changes when eventual residency
    // is paid, not which tiers are drawn or retained.
    u8::from(scaled_distance < 75.0)
        | (u8::from(scaled_distance < 24.0) << 1)
        | (u8::from((3.0..44.0).contains(&scaled_distance)) << 2)
        | (u8::from((8.0..75.0).contains(&scaled_distance)) << 3)
        | (u8::from((18.0..100.0).contains(&scaled_distance)) << 4)
        | (u8::from((40.0..220.0).contains(&scaled_distance)) << 5)
}

fn tree_cluster_aabb(center: Vec3, radius: f32) -> Aabb {
    let extent = Vec3::splat(radius.max(0.01));
    Aabb::from_min_max(center - extent, center + extent)
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(in crate::presentation) fn present_pending_trees(
    mut commands: Commands,
    pending: Query<(Entity, &Transform), With<PendingTreePresentation>>,
    active: Res<ActiveTacticalScene>,
    environments: Query<&SceneEnvironment>,
    terrains: Query<&SceneTerrain>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bark_materials: ResMut<Assets<TacticalTreeBarkMaterial>>,
    mut aggregate_bark_materials: ResMut<Assets<TacticalTreeAggregateBarkMaterial>>,
    mut leaf_card_materials: ResMut<Assets<TacticalTreeLeafCardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut tree_cache: ResMut<TreePresentationCache>,
    mut retained: ResMut<RetainedTreePresentations>,
    mut residency: ResMut<TreeAssetResidencyDiagnostics>,
    procedural_assets: Res<ProceduralTextureAssets>,
) {
    let Some(environment) = active
        .entity
        .and_then(|entity| environments.get(entity).ok())
    else {
        return;
    };
    let Some(terrain) = active.entity.and_then(|entity| terrains.get(entity).ok()) else {
        return;
    };
    retained.activate(
        environment,
        terrain,
        &mut tree_cache,
        &mut residency,
        &mut images,
    );
    let terrain_heightmap = tree_cache
        .terrain_heightmap
        .clone()
        .expect("active terrain creates a tree contact heightmap");
    let competition = canopy_competition(environment.canopy_bps);
    for (entity, transform) in &pending {
        let started = web_time::Instant::now();
        info!("Generating playable tactical tree presentation");
        let species = tree_species_for_site(transform.translation, environment);
        let (variant_index, variant_seed) =
            super::specimen::oak_variant_for_site(transform.translation);
        let cache_key = playable_cache_key(species, variant_seed, environment);
        if !tree_cache.variants.contains_key(&cache_key)
            && let Some(mut cached) = retained.variant(cache_key)
        {
            // Plant geometry is site-local; bind only bark to destination terrain.
            cached.bark_material = tree_cache.bark_for_terrain(
                species,
                &procedural_assets,
                terrain_heightmap.clone(),
                terrain,
                &mut bark_materials,
            );
            residency.add_resident_assets(&cached.resident_assets);
            tree_cache.variants.insert(cache_key, cached);
        }
        if !tree_cache.variants.contains_key(&cache_key) {
            let (branches, leaves) = playable_tree_source(
                species,
                variant_seed,
                variant_index,
                competition,
                environment,
            );
            info!(
                elapsed_ms = started.elapsed().as_millis(),
                branches = branches.len(),
                leaves = leaves.len(),
                "Generated playable tactical tree source geometry"
            );
            let bark_material = tree_cache.bark_for_terrain(
                species,
                &procedural_assets,
                terrain_heightmap.clone(),
                terrain,
                &mut bark_materials,
            );
            let aggregate_bark_material = match species {
                TreePresentationSpecies::EnglishOak => tree_cache
                    .oak_aggregate_bark_material
                    .clone()
                    .unwrap_or_else(|| {
                        let material = aggregate_bark_materials.add(oak_aggregate_bark_material());
                        tree_cache.oak_aggregate_bark_material = Some(material.clone());
                        material
                    }),
                TreePresentationSpecies::CommonBeech => tree_cache
                    .beech_aggregate_bark_material
                    .clone()
                    .unwrap_or_else(|| {
                        let material =
                            aggregate_bark_materials.add(beech_aggregate_bark_material());
                        tree_cache.beech_aggregate_bark_material = Some(material.clone());
                        material
                    }),
            };
            let leaf_material = leaf_card_materials.add(match species {
                TreePresentationSpecies::EnglishOak => oak_leaf_material(&procedural_assets),
                TreePresentationSpecies::CommonBeech => beech_leaf_material(&procedural_assets),
            });
            let bud_material = materials.add(super::materials::tree_bud_material(species));
            let cached = CachedTreePresentation::from_generated(
                species,
                variant_seed,
                branches,
                leaves,
                assets::TreeMaterialBindings {
                    bark: bark_material,
                    aggregate_bark: aggregate_bark_material,
                    leaf: leaf_material,
                    bud: bud_material,
                },
            );
            residency.add_resident_assets(&cached.resident_assets);
            tree_cache.variants.insert(cache_key, cached);
        }
        spawn_cached_tree(&mut commands, entity, cache_key, species.name());
        info!(
            elapsed_ms = started.elapsed().as_millis(),
            "Generated playable tactical tree presentation"
        );
        commands.entity(entity).remove::<PendingTreePresentation>();
    }
    retained.update_diagnostics(&tree_cache, &mut residency);
}

#[cfg(test)]
pub(super) fn oak_gnarling_for_test_site(
    recipe: OakGnarlingParameters,
    environment: &SceneEnvironment,
    tree_seed: Seed,
) -> OakGnarlingParameters {
    oak_gnarling_for_site(recipe, environment, tree_seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_tactical_core::prelude::{Precipitation, WeatherSnapshot};

    fn environment(canopy_bps: u16, hilly_bps: u16, wetland_bps: u16) -> SceneEnvironment {
        SceneEnvironment {
            scene_digest: "oak-site-test".into(),
            generation_version: 7,
            latitude_microdegrees:
                adventuresim_world_schema::coordinates::LatitudeMicrodegrees::new(53_500_000)
                    .unwrap(),
            longitude_microdegrees:
                adventuresim_world_schema::coordinates::LongitudeMicrodegrees::new(10_000_000)
                    .unwrap(),
            absolute_minute: adventuresim_world_schema::calendar::StrategicMinute::new(340_440),
            lunar_phase_minute: adventuresim_world_schema::calendar::StrategicMinute::new(340_440),
            absolute_elevation_metres: adventuresim_world_schema::ElevationMeters::new(420)
                .unwrap(),
            weather: WeatherSnapshot {
                rules_version: adventuresim_tactical_core::prelude::WEATHER_RULES_VERSION,
                interval_start_minute: adventuresim_world_schema::calendar::StrategicMinute::new(
                    340_440,
                ),
                cell_latitude: 214,
                cell_longitude: 40,
                temperature_deci_c: 120,
                wind_speed_bps: 1_200,
                precipitation: Precipitation::Clear,
                intensity_bps: 0,
                ground_moisture_bps: 100,
                snow_cover_bps: 0,
                atmosphere: Default::default(),
            },
            canopy_bps,
            wetland_bps,
            cultivation_bps: 0,
            water_bps: 0,
            hilly_bps,
        }
    }

    #[test]
    fn stable_site_not_current_weather_drives_oak_growth_history() {
        let site = environment(2_000, 8_000, 0);
        let first = oak_gnarling_for_site(
            OAK_GNARLING_SHOWCASE[0],
            &site,
            fabelgeist_determinism::Seed::from_u64(42),
        );
        let mut storm = site.clone();
        storm.weather.wind_speed_bps = 10_000;
        storm.weather.intensity_bps = 10_000;
        assert_eq!(
            first,
            oak_gnarling_for_site(
                OAK_GNARLING_SHOWCASE[0],
                &storm,
                fabelgeist_determinism::Seed::from_u64(42)
            )
        );
        assert_eq!(oak_site_key(&site), oak_site_key(&storm));
    }

    #[test]
    fn streaming_keeps_adjacent_tree_lods_resident_through_every_handoff() {
        for (distance, required_bits) in [
            (7.0, (1 << 1) | (1 << 2)),
            (14.0, (1 << 2) | (1 << 3)),
            (28.0, (1 << 3) | (1 << 4)),
            (55.0, (1 << 4) | (1 << 5)),
        ] {
            assert_eq!(tree_streaming_mask(distance) & required_bits, required_bits);
        }
    }

    #[test]
    fn streaming_prefetches_each_incoming_crown_one_outgoing_band_early() {
        for (distance, incoming_lod_bit) in [
            (90.0, 1 << 4),
            (72.0, 1 << 3),
            (42.0, 1 << 2),
            (22.0, 1 << 1),
        ] {
            assert_ne!(tree_streaming_mask(distance) & incoming_lod_bit, 0);
        }
        assert_eq!(tree_streaming_mask(120.0), 1 << 5);
    }

    #[test]
    fn leaf_residency_only_overlaps_near_the_representation_handoff() {
        assert_eq!(
            tree_streamed_leaf_representation(1.4),
            Some(TreeLeafRepresentation::TexturedMesh)
        );
        assert_eq!(tree_streamed_leaf_representation(2.0), None);
        assert_eq!(
            tree_streamed_leaf_representation(2.6),
            Some(TreeLeafRepresentation::AlphaCard)
        );
    }

    #[test]
    fn aggregate_tree_children_use_the_cheap_bark_material_tier() {
        let mut world = World::new();
        let entity = world
            .spawn((
                PlayableTreeAggregateWood,
                aggregate_tree_wood_material(Handle::default()),
            ))
            .id();
        let entity = world.entity(entity);
        assert!(entity.contains::<MeshMaterial3d<TacticalTreeAggregateBarkMaterial>>());
        assert!(!entity.contains::<MeshMaterial3d<TacticalTreeBarkMaterial>>());
    }

    #[test]
    fn only_lod2_aggregate_wood_is_excluded_from_shadow_casting() {
        let mut world = World::new();
        for lod in [1, 2] {
            let mut entity = world.spawn((
                PlayableTreeAggregateWood,
                TreeLod(lod),
                aggregate_tree_wood_material(Handle::default()),
            ));
            if !aggregate_tree_wood_casts_shadows(lod) {
                entity.insert(NotShadowCaster);
            }
        }

        let mut aggregate_wood = world.query::<(&TreeLod, Has<NotShadowCaster>)>();
        let mut policies = aggregate_wood
            .iter(&world)
            .map(|(lod, not_shadow_caster)| (lod.0, not_shadow_caster))
            .collect::<Vec<_>>();
        policies.sort_unstable_by_key(|(lod, _)| *lod);

        assert_eq!(policies, [(1, false), (2, true)]);
        assert!(aggregate_tree_wood_casts_shadows(1));
        assert!(!aggregate_tree_wood_casts_shadows(2));
    }

    #[test]
    fn distant_request_does_not_generate_near_tree_geometry() {
        let variant_seed = fabelgeist_determinism::Seed::from_u64(42);
        let branches = procedural_tree_skeleton(variant_seed, 0.0);
        let leaves = procedural_oak_leaves(variant_seed, &branches, 0.0);
        let mut cached = CachedTreePresentation {
            origin: TreeAssetOrigin::Generated,
            variant_seed,
            species: TreePresentationSpecies::EnglishOak,
            species_name: TreePresentationSpecies::EnglishOak.name(),
            branches: branches.into(),
            leaves: leaves.into(),
            trunk_mesh: None,
            mid_trunk_mesh: None,
            cluster_layout: None,
            clusters: None,
            aggregate_branch_meshes: [None, None],
            lod_cards: [None, None, None, None],
            bark_material: Handle::default(),
            aggregate_bark_material: Handle::default(),
            leaf_material: Handle::default(),
            bud_material: Handle::default(),
            resident_assets: TreeAssetResidencyDiagnostics::default(),
        };
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<TacticalTreeImpostorMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut diagnostics = TreeAssetResidencyDiagnostics::default();
        ensure_tree_assets_resident(
            &mut cached,
            1 << 5,
            None,
            None,
            &mut meshes,
            &mut materials,
            &mut images,
            &mut diagnostics,
        )
        .expect("generated fixture assets are supported");
        assert!(cached.lod_cards[3].is_some());
        assert!(cached.lod_cards[..3].iter().all(Option::is_none));
        assert!(cached.trunk_mesh.is_none());
        assert!(cached.mid_trunk_mesh.is_none());
        assert!(cached.clusters.is_none());
        assert_eq!(diagnostics.detailed_trunk_vertices, 0);
        assert_eq!(diagnostics.mid_trunk_vertices, 0);
        assert_eq!(diagnostics.cambered_leaf_vertices, 0);
        assert_eq!(diagnostics.leaf_card_vertices, 0);
        assert!(diagnostics.impostor_vertices > 0);
        assert_eq!(
            diagnostics.impostor_texture_bytes_by_lod,
            [0, 0, 0, 2_359_296]
        );
        assert_eq!(diagnostics.impostor_texture_bytes, 2_359_296);
    }

    #[test]
    fn aggregate_residency_records_exact_lod_tier_geometry_budgets() {
        let variant_seed = fabelgeist_determinism::Seed::from_u64(42);
        let branches = procedural_tree_skeleton(variant_seed, 0.0);
        let leaves = procedural_oak_leaves(variant_seed, &branches, 0.0);
        let mut cached = CachedTreePresentation {
            origin: TreeAssetOrigin::Generated,
            variant_seed,
            species: TreePresentationSpecies::EnglishOak,
            species_name: TreePresentationSpecies::EnglishOak.name(),
            branches: branches.into(),
            leaves: leaves.into(),
            trunk_mesh: None,
            mid_trunk_mesh: None,
            cluster_layout: None,
            clusters: None,
            aggregate_branch_meshes: [None, None],
            lod_cards: [None, None, None, None],
            bark_material: Handle::default(),
            aggregate_bark_material: Handle::default(),
            leaf_material: Handle::default(),
            bud_material: Handle::default(),
            resident_assets: TreeAssetResidencyDiagnostics::default(),
        };
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<TacticalTreeImpostorMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut diagnostics = TreeAssetResidencyDiagnostics::default();

        ensure_tree_assets_resident(
            &mut cached,
            (1 << 2) | (1 << 3),
            None,
            None,
            &mut meshes,
            &mut materials,
            &mut images,
            &mut diagnostics,
        )
        .expect("generated fixture assets are supported");

        assert!(cached.aggregate_branch_meshes.iter().all(Option::is_some));
        assert_eq!(diagnostics.aggregate_branch_vertices_by_lod, [4_810, 462]);
        assert_eq!(diagnostics.aggregate_branch_triangles_by_lod, [7_099, 700]);
        assert_eq!(diagnostics.aggregate_branch_vertices, 5_272);
    }

    #[test]
    fn complete_live_oak_mesh_suite_remains_constructible() {
        let branches = procedural_tree_skeleton(fabelgeist_determinism::Seed::from_u64(42), 0.0);
        let leaves =
            procedural_oak_leaves(fabelgeist_determinism::Seed::from_u64(42), &branches, 0.0);
        let mut vertex_count = procedural_tree_branch_mesh(&branches, 0).count_vertices();
        vertex_count += procedural_woody_mid_trunk_mesh(&branches).count_vertices();
        for primary_group in 0..TREE_PRIMARY_GROUP_COUNT {
            vertex_count +=
                procedural_tree_branch_group_mesh(&branches, 3, primary_group).count_vertices();
            vertex_count +=
                procedural_oak_textured_leaf_group_mesh(&leaves, primary_group).count_vertices();
            vertex_count +=
                procedural_oak_leaf_card_group_mesh(&leaves, primary_group).count_vertices();
            vertex_count +=
                procedural_oak_bud_group_mesh(&branches, primary_group).count_vertices();
        }
        vertex_count += [
            (2, WoodyBranchMeshQuality::AggregateLod1),
            (1, WoodyBranchMeshQuality::AggregateLod2),
        ]
        .into_iter()
        .map(|(depth, quality)| {
            procedural_woody_crown_mesh(&branches, depth, quality).count_vertices()
        })
        .sum::<usize>();
        assert!(vertex_count > 0);
    }

    #[test]
    fn exposed_hilly_sites_share_wind_direction_and_gnarl_more_than_shelter() {
        let exposed = environment(1_000, 9_000, 0);
        let sheltered = environment(9_000, 0, 0);
        let exposed_a = oak_gnarling_for_site(
            OAK_GNARLING_SHOWCASE[0],
            &exposed,
            fabelgeist_determinism::Seed::from_u64(7),
        );
        let exposed_b = oak_gnarling_for_site(
            OAK_GNARLING_SHOWCASE[0],
            &exposed,
            fabelgeist_determinism::Seed::from_u64(91),
        );
        let sheltered = oak_gnarling_for_site(
            OAK_GNARLING_SHOWCASE[0],
            &sheltered,
            fabelgeist_determinism::Seed::from_u64(7),
        );
        assert_eq!(
            exposed_a.stress_azimuth_radians,
            exposed_b.stress_azimuth_radians
        );
        assert!(exposed_a.trunk_lean > sheltered.trunk_lean);
        assert!(exposed_a.scaffold_sweep > sheltered.scaffold_sweep);
        assert!(exposed_a.crown_asymmetry > sheltered.crown_asymmetry);
        assert!(exposed_a.root_spread > sheltered.root_spread);
        assert!(exposed_a.root_exposure > sheltered.root_exposure);
    }

    #[test]
    fn beech_selection_is_clustered_and_favors_mesic_closed_canopy() {
        let mut mesic = environment(9_000, 1_000, 0);
        mesic.weather.ground_moisture_bps = 7_000;
        let mut open = environment(1_000, 1_000, 0);
        open.weather.ground_moisture_bps = 800;
        let mesic_count = (-30..30)
            .flat_map(|x| (-30..30).map(move |z| Vec3::new(x as f32 * 30.0, 0.0, z as f32 * 30.0)))
            .filter(|position| {
                tree_species_for_site(*position, &mesic) == TreePresentationSpecies::CommonBeech
            })
            .count();
        let open_count = (-30..30)
            .flat_map(|x| (-30..30).map(move |z| Vec3::new(x as f32 * 30.0, 0.0, z as f32 * 30.0)))
            .filter(|position| {
                tree_species_for_site(*position, &open) == TreePresentationSpecies::CommonBeech
            })
            .count();
        assert!(mesic_count > open_count * 3);
        let origin = tree_species_for_site(Vec3::new(1.0, 0.0, 1.0), &mesic);
        assert_eq!(
            origin,
            tree_species_for_site(Vec3::new(29.0, 0.0, 29.0), &mesic)
        );
    }

    #[test]
    fn beech_whole_tree_billboard_uses_beech_geometry_and_palette() {
        let branches = procedural_woody_plant_skeleton(
            fabelgeist_determinism::Seed::from_u64(42),
            0.7,
            COMMON_BEECH_PARAMETERS,
        );
        let leaves = procedural_woody_plant_leaves(
            fabelgeist_determinism::Seed::from_u64(42),
            &branches,
            0.7,
            COMMON_BEECH_PARAMETERS,
        );
        let bake = bake_tree_lod_with_style(
            fabelgeist_determinism::Seed::from_u64(42),
            &branches,
            &leaves,
            4,
            TreePresentationSpecies::CommonBeech.bake_style(),
        );
        validate_tree_bake_provenance(&bake.provenance);
        assert!(
            bake.image
                .data
                .as_ref()
                .is_some_and(|pixels| pixels.iter().any(|channel| *channel > 0))
        );
        assert_eq!(
            bake.provenance.source_geometry_hash,
            super::super::impostor::tree_source_geometry_hash(&branches, &leaves)
                ^ u64::from(super::super::impostor::TREE_IMPOSTOR_BAKE_VERSION)
        );
    }

    #[test]
    fn forced_beech_vista_handoff_uses_beech_source_geometry_and_cache_identity() {
        let variant_seed = crate::presentation::obstacles::tree::specimen::oak_variant_seed(0);
        let competition = 0.5;
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<TacticalTreeImpostorMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut cache = VistaTreePresentationCache::default();
        let beech = ensure_vista_tree_variant(
            variant_seed,
            competition,
            TreePresentationSpecies::CommonBeech,
            &mut meshes,
            &mut materials,
            &mut images,
            &mut cache,
            None,
        );
        let oak = ensure_vista_tree_variant(
            variant_seed,
            competition,
            TreePresentationSpecies::EnglishOak,
            &mut meshes,
            &mut materials,
            &mut images,
            &mut cache,
            None,
        );
        let branches =
            procedural_woody_plant_skeleton(variant_seed, competition, COMMON_BEECH_PARAMETERS);
        let leaves = procedural_woody_plant_leaves(
            variant_seed,
            &branches,
            competition,
            COMMON_BEECH_PARAMETERS,
        );
        assert_eq!(
            beech.provenance.source_geometry_hash,
            super::super::impostor::tree_source_geometry_hash(&branches, &leaves)
                ^ u64::from(super::super::impostor::TREE_IMPOSTOR_BAKE_VERSION)
        );
        assert_ne!(
            beech.provenance.source_geometry_hash,
            oak.provenance.source_geometry_hash
        );
        assert_eq!(
            cache.variants.len(),
            2,
            "oak and beech vista atlases must not alias"
        );
    }
}

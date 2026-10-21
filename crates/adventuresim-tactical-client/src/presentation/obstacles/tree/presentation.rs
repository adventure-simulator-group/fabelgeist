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
use adventuresim_tactical_core::prelude::{SceneTerrain, SceneTreeContent};
use bevy::{
    camera::{primitives::Aabb, visibility::NoFrustumCulling},
    light::NotShadowCaster,
    prelude::*,
};
use fabelgeist_determinism::Seed;
use fabelgeist_determinism::StreamId;

use assets::ensure_tree_assets_resident;
use bevy::ecs::system::{SystemParam, SystemState};
use cache::{ActiveTreeCache, TreeCacheContents, TreeSceneBinding};
pub(in crate::presentation) use cache::{RecipeTreeCache, TreePresentationCache};
pub(crate) use content::TreeContentRequest;
pub(crate) use diagnostics::RecipeTreeAssetResidencyDiagnostics;
#[cfg(feature = "grown-trees")]
pub(in crate::presentation) use grown::install_grown_streaming;
#[cfg(feature = "grown-trees")]
pub(crate) use grown::{
    GrownCatalogueAssets, GrownPartBinding, GrownRootAttachment, GrownSceneAssets,
    GrownSceneRequest, GrownSource,
};
use recipe_materials::RecipeMaterials;
pub(in crate::presentation) use residency::RetainedTreePresentations;
use streaming::spawn_cached_tree;
pub(in crate::presentation) use streaming::stream_tree_lod_children;
pub(in crate::presentation) use vista::ensure_vista_tree_variant;

#[cfg(test)]
use super::{TREE_PRIMARY_GROUP_COUNT, source::oak_site_key};

mod assets;
mod cache;
pub(in crate::presentation) mod content;
mod diagnostics;
#[cfg(feature = "grown-trees")]
mod grown;
#[cfg(all(feature = "foliage-review", not(target_family = "wasm")))]
pub(super) mod prepared;
mod recipe_materials;
mod residency;
mod streaming;
mod vista;

#[derive(Component)]
pub(in crate::presentation) struct PendingTreePresentation;

/// Exact submitted triangle count for one streamed leaf-cluster entity.
/// Stored when the CPU mesh is built because production meshes intentionally
/// relinquish their main-world vertex data after render extraction.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct TreeLeafTriangleCount(pub(crate) usize);

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
    resident_assets: RecipeTreeAssetResidencyDiagnostics,
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

#[derive(Clone)]
pub(in crate::presentation) struct RecipeTreeStream {
    cache_key: Seed,
    resident_mask: u8,
    resident_leaf_mask: u8,
    active_mask: u8,
    active_leaf: Option<TreeLeafRepresentation>,
}

#[derive(Component, Clone)]
pub(in crate::presentation) enum StreamedTreePresentation {
    Recipe(RecipeTreeStream),
    #[cfg(feature = "grown-trees")]
    Grown(grown::GrownTreeStream),
}

impl StreamedTreePresentation {
    pub(super) fn recipe(&self) -> anyhow::Result<&RecipeTreeStream> {
        match self {
            Self::Recipe(recipe) => Ok(recipe),
            #[cfg(feature = "grown-trees")]
            Self::Grown(_) => anyhow::bail!("tree stream contains grown content"),
        }
    }

    pub(in crate::presentation) fn weather_occlusion_cache_key(&self) -> anyhow::Result<Seed> {
        Ok(self.recipe()?.cache_key)
    }
}

#[derive(Component)]
pub(in crate::presentation) struct StreamedTreeChild;

#[derive(SystemParam)]
pub(in crate::presentation) struct RecipePending<'w, 's> {
    commands: Commands<'w, 's>,
    pending: Query<'w, 's, (Entity, &'static Transform), With<PendingTreePresentation>>,
    environments: Query<'w, 's, &'static SceneEnvironment>,
    terrains: Query<'w, 's, &'static SceneTerrain>,
    source_materials: RecipeMaterials<'w>,
    images: ResMut<'w, Assets<Image>>,
    tree_cache: ResMut<'w, TreePresentationCache>,
    retained: ResMut<'w, RetainedTreePresentations>,
    residency: ResMut<'w, RecipeTreeAssetResidencyDiagnostics>,
}

impl RecipePending<'_, '_> {
    fn present(&mut self, scene: Entity) -> bevy::prelude::Result {
        let Self {
            commands,
            pending,
            environments,
            terrains,
            source_materials,
            images,
            tree_cache,
            retained,
            residency,
        } = self;
        let environment = environments.get(scene)?;
        let terrain = terrains.get(scene)?;
        retained.activate_recipe(scene, environment, terrain, tree_cache, residency, images)?;
        let recipe = tree_cache.recipe_mut()?;
        let terrain_heightmap = recipe.terrain_heightmap.clone();
        let competition = canopy_competition(environment.canopy_bps);
        for (entity, transform) in pending.iter() {
            let started = web_time::Instant::now();
            info!("Generating playable tactical tree presentation");
            let species = tree_species_for_site(transform.translation, environment);
            let (variant_index, variant_seed) =
                super::specimen::oak_variant_for_site(transform.translation);
            let cache_key = playable_cache_key(species, variant_seed, environment);
            if !recipe.variants.contains_key(&cache_key)
                && let Some(mut cached) = retained.recipe_variant(cache_key)
            {
                // Plant geometry is site-local; bind only bark to destination terrain.
                cached.bark_material = recipe.bark_for_terrain(
                    species,
                    &source_materials.textures,
                    terrain_heightmap.clone(),
                    terrain,
                    &mut source_materials.bark,
                );
                residency.add_resident_assets(&cached.resident_assets);
                recipe.variants.insert(cache_key, cached);
            }
            if !recipe.variants.contains_key(&cache_key) {
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
                let bindings = source_materials.bind(recipe, species, terrain);
                let cached = CachedTreePresentation::from_generated(
                    species,
                    variant_seed,
                    branches,
                    leaves,
                    bindings,
                );
                residency.add_resident_assets(&cached.resident_assets);
                recipe.variants.insert(cache_key, cached);
            }
            spawn_cached_tree(commands, entity, cache_key, species.name());
            info!(
                elapsed_ms = started.elapsed().as_millis(),
                "Generated playable tactical tree presentation"
            );
            commands.entity(entity).remove::<PendingTreePresentation>();
        }
        retained.update_diagnostics(tree_cache, residency);
        Ok(())
    }
}

pub(in crate::presentation) fn present_pending_trees(
    world: &mut World,
    recipe_state: &mut SystemState<RecipePending<'static, 'static>>,
) -> bevy::prelude::Result {
    let Some(scene) = world.resource::<ActiveTacticalScene>().entity else {
        return Ok(());
    };
    if world.get::<SceneEnvironment>(scene).is_none()
        || world.get::<SceneTerrain>(scene).is_none()
        || world.get::<SceneTreeContent>(scene).is_none()
    {
        return Ok(());
    }
    let Some(request) = world.get::<TreeContentRequest>(scene).cloned() else {
        return Ok(());
    };
    match request {
        TreeContentRequest::Recipe => {
            let result = recipe_state.get_mut(world)?.present(scene);
            recipe_state.apply(world);
            result
        }
        #[cfg(feature = "grown-trees")]
        TreeContentRequest::Grown(request) => {
            grown::present(world, scene, &request).map_err(Into::into)
        }
    }
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
            resident_assets: RecipeTreeAssetResidencyDiagnostics::default(),
        };
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<TacticalTreeImpostorMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut diagnostics = RecipeTreeAssetResidencyDiagnostics::default();
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
            resident_assets: RecipeTreeAssetResidencyDiagnostics::default(),
        };
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<TacticalTreeImpostorMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut diagnostics = RecipeTreeAssetResidencyDiagnostics::default();

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

//! Shared tactical scene and gameplay-camera presentation.
//!
//! Both the networked client and deterministic animation capture install this
//! plugin so screenshots cannot drift to a different camera, terrain mesh,
//! lighting, or post-processing setup.

#![expect(
    unused_imports,
    reason = "the gameplay client and capture binaries consume different parts of this shared presentation facade"
)]

mod atmosphere;
mod buildings;
mod clouds;
mod config;
mod doors;
mod environment;
pub(crate) mod ground_scatter;
mod materials;
mod obstacles;
mod procedural;
mod procedural_texture_setup;
mod sky;
mod terrain;
mod vista;
mod volumetric;
mod weather;
mod windows;

use adventuresim_procedural_textures::LeafTextureSet;
pub(crate) use adventuresim_procedural_textures::ProceduralTextureAssets;
#[cfg(test)]
use adventuresim_procedural_textures::generate_procedural_textures;
use atmosphere::*;
use buildings::*;
use clouds::*;
pub(crate) use doors::{DoorPresentationPlugin, GrabTargetOutline};
use environment::*;
use ground_scatter::*;
use obstacles::on_scene_obstacle_added;
use obstacles::rock::TacticalRockMaterial;
use obstacles::tree::*;
use procedural::*;
use procedural_texture_setup::setup_procedural_texture_assets;
use sky::*;
use terrain::*;
use vista::*;
use volumetric::*;
use weather::*;
use windows::WindowPresentationPlugin;

pub use config::{
    AntiAliasingConfig, PresentModeConfig, ShadowFiltering, SmaaQuality, TacticalGraphicsConfig,
    TonemappingConfig, WindowModeConfig,
};

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct TerrainTriangleCount(pub(crate) usize);

fn mesh_triangle_count(mesh: &Mesh) -> usize {
    mesh.indices()
        .map_or_else(|| mesh.count_vertices() / 3, |indices| indices.len() / 3)
}

// This facade is compiled independently by several binaries, so each binary
// uses only the subset of the stable presentation interface that it needs.
pub(crate) use buildings::{PresentedBuildingMesh, PresentedSign, TacticalBuildingMaterials};
pub(crate) use clouds::{
    TacticalCloudAnimationStatus, TacticalCloudBenchmarkIsolation, TacticalCloudCaptureOverride,
    TacticalCloudCaptureProfile, TacticalCloudLayer, TacticalCloudOffscreenCamera,
};
pub(crate) use environment::{
    TacticalCameraSetup, TacticalGameplayCamera, scene_ambient_light, scene_ibl_visibility_floor,
};
pub(crate) use ground_scatter::{
    GrassInteractor, GroundLitterCaptureAnchors, GroundLitterCapturePair, GroundLitterDiagnostics,
    GroundScatterLayer, LooseStonePebblePatch, UnderstoryReviewSpecimen,
    WoodyUnderstoryPresentationCache, spawn_understory_review_specimens,
};
pub(crate) use obstacles::oak_review_terminal_specimen;
pub(crate) use obstacles::rock::ProceduralRockVisual;
pub(crate) use obstacles::tree::TreeImpostorProvenance;
pub(crate) use obstacles::tree::{
    PlayableTreeAggregateWood, PlayableTreeBuds, PlayableTreeCanopyCard,
    PlayableTreeDetailedLeaves, PlayableTreeDetailedTrunk, PlayableTreeDetailedWood,
    PlayableTreeMidTrunk, PlayableTreeTrunk, PresentedTree, TacticalTreeAggregateBarkMaterial,
    TacticalTreeBarkMaterial, TacticalTreeBenchmarkIsolation, TacticalTreeLeafCardMaterial,
    TreeAssetResidencyDiagnostics, TreeLeafRepresentation, TreeLeafTriangleCount, TreeLod,
    TreeLodCluster, TreeLodRenderOverride, TreeTrunkLod, oak_aggregate_bark_material,
    oak_bark_material, oak_leaf_material,
};
pub(crate) use sky::AtmosphereIblAmbientHandoff;
pub(crate) use sky::{TacticalMoon, TacticalMoonlight, TacticalStars, TacticalSunlight};
pub(crate) use terrain::{
    DETAIL_PATCH_SPACING_METRES, TerrainDetailPatch, TerrainMaterialPresentation,
    terrain_heightmap_image,
};
pub(crate) use vista::{VistaTerrain, VistaTerrainMesh, VistaTreePresentation};
pub(crate) use weather::WeatherParticle;

use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::prelude::SceneVistaBundle;
#[cfg(test)]
use bevy::mesh::VertexAttributeValues;
use bevy::{
    asset::RenderAssetUsages,
    camera::{
        Exposure,
        visibility::{NoFrustumCulling, VisibilityRange},
    },
    core_pipeline::tonemapping::Tonemapping,
    image::ImageSampler,
    light::{
        Atmosphere, AtmosphereEnvironmentMapLight, DirectionalLightShadowMap, EnvironmentMapLight,
        NotShadowCaster, atmosphere::ScatteringMedium, light_consts::lux,
    },
    mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
    pbr::{AtmosphereSettings, ExtendedMaterial, MaterialExtension},
    post_process::bloom::Bloom,
    prelude::*,
    render::render_resource::{
        AsBindGroup, Extent3d, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        TextureDimension, TextureFormat, VertexFormat,
    },
    shader::ShaderRef,
};
pub(crate) use fabelgeist_determinism::splitmix64;
use web_time::Instant;

#[derive(Resource)]
pub(crate) struct ClientStartupTiming {
    started_at: Instant,
    terrain_preparation_reported: bool,
}

impl ClientStartupTiming {
    pub(crate) fn new(started_at: Instant) -> Self {
        Self {
            started_at,
            terrain_preparation_reported: false,
        }
    }

    pub(crate) fn mark(&self, phase: &str) {
        let elapsed_ms = self.started_at.elapsed().as_millis();
        info!(phase, elapsed_ms, "[startup] tactical client phase");
        #[cfg(not(target_family = "wasm"))]
        eprintln!("[startup] native client phase={phase:?} elapsed_ms={elapsed_ms}");
    }

    pub(crate) fn mark_terrain_prepared_once(&mut self) {
        if self.terrain_preparation_reported {
            return;
        }
        self.terrain_preparation_reported = true;
        self.mark("first tactical terrain prepared");
    }
}

#[derive(Debug, Clone)]
pub struct TacticalPresentationPlugin {
    pub config: TacticalGraphicsConfig,
}

impl Default for TacticalPresentationPlugin {
    fn default() -> Self {
        Self {
            config: TacticalGraphicsConfig::parse(include_str!(
                "../../../../assets/config/tactical-graphics.yaml"
            ))
            .expect("shipped tactical graphics configuration must be valid"),
        }
    }
}

fn tactical_global_ambient_light() -> GlobalAmbientLight {
    GlobalAmbientLight {
        color: Color::srgb(0.36, 0.48, 0.72),
        brightness: 0.6,
        ..default()
    }
}

impl Plugin for TacticalPresentationPlugin {
    fn build(&self, app: &mut App) {
        // GPU-instanced grass renders through bevy_eidolon on native and wasm
        // (the fork's WebGPU draw path substitutes draw_indexed_indirect for
        // multi-draw-indirect on the browser backend).
        app.add_plugins(ground_scatter::InstancedGrassPlugin);
        app.add_plugins(materials::TacticalMaterialsPlugin)
            // Tactical play uses one compact close-range cascade for whichever
            // celestial light is active. Keep the map allocation identical in the
            // game and all tactical review viewers.
            .insert_resource(DirectionalLightShadowMap {
                size: self.config.rendering.shadows.map_size,
            })
            .insert_resource(TacticalGraphicsSettings {
                config: self.config.clone(),
            })
            .init_resource::<TacticalCameraSetup>()
            // The sky observer preserves this low, cool floor at night and restores
            // physically scaled diffuse sky irradiance during daylight.
            .insert_resource(tactical_global_ambient_light())
            .add_systems(
                Startup,
                (
                    setup_procedural_texture_assets,
                    setup_tactical_building_materials,
                    setup_tactical_presentation,
                    setup_tactical_sky,
                    setup_tactical_clouds,
                )
                    .chain(),
            )
            .init_resource::<WoodyUnderstoryPresentationCache>()
            .init_resource::<GroundFoliagePresentationCache>()
            .init_resource::<TreePresentationCache>()
            .init_resource::<TreeAssetResidencyDiagnostics>()
            .init_resource::<VistaTreePresentationCache>()
            .init_resource::<ActiveVistaSurface>()
            .init_resource::<TreeLodRenderOverride>()
            .init_resource::<TacticalTreeBenchmarkIsolation>()
            .init_resource::<ActiveTacticalScene>()
            .init_resource::<PresentedCelestialLighting>()
            .init_resource::<FrozenAtmosphereStatus>()
            .init_resource::<AtmosphereIblAmbientHandoff>()
            .init_resource::<TacticalCloudCaptureOverride>()
            .init_resource::<TacticalCloudBenchmarkIsolation>()
            .init_resource::<WeatherOcclusionState>()
            .add_systems(
                Update,
                (
                    (
                        present_pending_terrain,
                        update_terrain_detail_patch,
                        present_ground_scatter,
                    )
                        .chain(),
                    (
                        refresh_active_tactical_scene,
                        update_presented_celestial_lighting,
                        apply_presented_celestial_lighting,
                    )
                        .chain(),
                    update_celestial_material_lighting
                        .after(update_presented_celestial_lighting)
                        .after(present_pending_trees),
                    (
                        present_pending_trees,
                        stream_tree_lod_children,
                        update_tree_projected_lod_ranges,
                    )
                        .chain(),
                    keep_celestial_visuals_centered.after(update_presented_celestial_lighting),
                    update_tactical_clouds.after(update_presented_celestial_lighting),
                    update_tactical_cloud_offscreen_target,
                    update_global_ambient_policy.after(apply_presented_celestial_lighting),
                    freeze_initialized_atmosphere
                        .after(update_global_ambient_policy)
                        .after(update_presented_celestial_lighting),
                    apply_active_environment_fog.after(refresh_active_tactical_scene),
                    apply_active_scene_weather
                        .after(refresh_active_tactical_scene)
                        .after(present_pending_trees),
                    update_weather_occlusion_map
                        .after(apply_active_scene_weather)
                        .after(present_pending_trees),
                ),
            )
            .add_observer(on_game_scene_added)
            .add_observer(activate_tactical_scene)
            .add_observer(terrain::on_environment_added)
            .add_observer(terrain::on_ground_added)
            .add_observer(on_scene_obstacle_added)
            .add_plugins(BuildingPresentationPlugin)
            .add_observer(on_scene_vista_bundle);
    }

    fn finish(&self, app: &mut App) {
        // Install this after every plugin has built so the RenderApp and
        // Bevy's atmosphere extractor exist regardless of plugin order.
        install_atmosphere_cleanup_backport(app);
    }
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct TacticalGraphicsSettings {
    pub(crate) config: TacticalGraphicsConfig,
}

impl Default for TacticalGraphicsSettings {
    fn default() -> Self {
        Self {
            config: TacticalPresentationPlugin::default().config,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    /// WGSL requires every directive to precede the first declaration, and an
    /// `#import` expands into declarations. A directive written below the
    /// imports fails composition for the whole file, which surfaces only as a
    /// log line while the material stops rendering on every backend.
    #[test]
    fn shader_directives_precede_declarations() {
        // Both shader roots the tactical renderer composes: the runtime asset
        // directory and the shaders embedded in the procedural materials crate.
        const SHADER_ROOTS: [&str; 2] = [
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/shaders"),
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../adventuresim-procedural-materials/src/shaders"
            ),
        ];
        let mut checked = 0;
        for root in SHADER_ROOTS {
            for entry in fs::read_dir(Path::new(root)).expect("tactical shader directory") {
                let path = entry.expect("shader directory entry").path();
                if path.extension().is_none_or(|extension| extension != "wgsl") {
                    continue;
                }
                let source = fs::read_to_string(&path).expect("readable tactical shader");
                assert_directives_precede_declarations(&path.display().to_string(), &source);
                checked += 1;
            }
        }
        assert!(checked > 0, "no tactical shaders were checked");
    }

    fn assert_directives_precede_declarations(label: &str, source: &str) {
        let mut declared = false;
        for (index, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            if line.starts_with("diagnostic(")
                || line.starts_with("enable ")
                || line.starts_with("requires ")
            {
                assert!(
                    !declared,
                    "{label}:{}: WGSL directive written after a declaration; \
                     move it above the imports or the shader will not compile",
                    index + 1
                );
            } else if !line.starts_with('#') || line.starts_with("#import") {
                declared = true;
            }
        }
    }
}

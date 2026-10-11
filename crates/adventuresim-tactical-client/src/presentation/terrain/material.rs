//! Physical terrain material construction shared by scene and map ground.
use super::*;

pub(in crate::presentation) fn terrain_material(
    terrain: &SceneTerrain,
    environment: &SceneEnvironment,
    ground: Option<&SceneGround>,
    procedural_assets: &ProceduralTextureAssets,
    images: &mut Assets<Image>,
    grass: &crate::presentation::config::GrassConfig,
) -> TacticalTerrainMaterial {
    TacticalTerrainMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.92,
            metallic: 0.0,
            ..default()
        },
        extension: TacticalTerrainExtension {
            source_from_world: Mat4::IDENTITY,
            base_color: color_vec4(scene_ground_color(environment)),
            // Match the rendered optical average of the sward rather than its
            // brighter pre-lighting blade pigment.
            grass_color: color_vec4(grass_terminal_pigment(environment)),
            cover: Vec4::new(
                bps(environment.canopy_bps),
                bps(environment.wetland_bps),
                bps(environment.cultivation_bps),
                bps(environment.water_bps),
            ),
            weather: Vec4::new(
                bps(environment.weather.ground_moisture_bps),
                bps(environment.weather.snow_cover_bps),
                bps(environment.hilly_bps),
                bps(environment.weather.wind_speed_bps),
            ),
            // The final Vista-to-terrain fade retains a band-limited sward
            // instead of paying for sub-pixel grass. x/y are its distance
            // interval; z is environment-dependent coverage and w is reserved.
            far_sward: Vec4::new(
                grass.lod.vista.fade_out_m[0],
                grass.lod.vista.fade_out_m[1],
                (1.0 - bps(environment.water_bps) * 0.9
                    - bps(environment.weather.snow_cover_bps) * 0.8)
                    .clamp(0.0, 1.0),
                0.0,
            ),
            // Replace Far's removed physical coverage during the Near-to-Far
            // crossfade. x/y are the shared blade-LOD interval; z is the
            // stable Far subset's missing projected coverage; w is reserved.
            lod_sward: Vec4::new(
                grass.lod.far.fade_in_m[0],
                grass.lod.far.fade_in_m[1],
                grass.transition.terrain_gap_fill_fraction,
                0.0,
            ),
            // x/y are the authoritative playable half extents. The detail
            // patch alone can extend beyond them; z controls its discrete
            // substrate-to-vista-sward handoff and w is reserved.
            playable_bounds: Vec4::new(terrain.width() * 0.5, terrain.depth() * 0.5, 4.0, 0.0),
            // x marks the coarse base material. Its shader removes only the
            // safely covered interior beneath the signed detail patch; the
            // outer overlap remains coplanar and morphs continuously.
            detail_patch: Vec4::new(1.0, DETAIL_PATCH_BASE_CUTOUT_RADIUS_METRES, 0.0, 0.0),
            // x is tile repetitions per metre, y is the decoded physical
            // height range, z scales the derivative normal, and w is the
            // distance where sub-centimetre detail is completely absent.
            soil_detail: Vec4::new(
                1.0 / FOREST_SOIL_TILE_METRES,
                FOREST_SOIL_HEIGHT_RANGE_METRES,
                1.0,
                24.0,
            ),
            // Dense leaf litter is an aggregate terrain material first. Its
            // packed height/AO/tone/coverage map supplies the continuous
            // forest-floor mass beneath sparse silhouette-breaking meshes.
            litter_detail: Vec4::new(
                1.0 / FOREST_LITTER_TILE_METRES,
                FOREST_LITTER_HEIGHT_RANGE_METRES,
                0.72,
                32.0,
            ),
            // Cliff presentation is disabled on both ordinary terrain draws.
            // The implicit patch clone enables it with its required recipe.
            cliff_palette_a: Vec4::ZERO,
            cliff_palette_b: Vec4::ZERO,
            cliff_surface: Vec4::new(1.0 / ROCK_TILE_METRES, ROCK_HEIGHT_RANGE_METRES, 0.8, 0.9),
            cliff_structure_a: Vec4::ZERO,
            cliff_structure_b: Vec4::ZERO,
            ground_map: images.add(ground_map_image(
                ground,
                stable_text_seed(&environment.scene_digest),
            )),
            soil_height_ao: procedural_assets.forest_soil.height_ao.clone(),
            litter_surface: procedural_assets.forest_soil.litter_surface.clone(),
            litter_normal: procedural_assets.forest_soil.litter_normal.clone(),
            blood_mask: procedural_assets.terrain_blood_mask.clone(),
            cliff_height: procedural_assets.rock.height.clone(),
            cliff_arm: procedural_assets.rock.arm.clone(),
        },
    }
}

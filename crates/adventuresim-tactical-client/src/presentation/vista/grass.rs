//! Shared pure vista tuft preparation and renderer installation.
use super::*;
use crate::presentation::ground_scatter::instanced_grass::{PreparedTufts, prepare_scene_tufts};
use crate::presentation::ground_scatter::{
    GrassWorld, MINIMUM_GRASS_SLOPE_NORMAL_Y, TierSpeciesBatches, TuftPigment, TuftPlacement,
    grass_scatter_density, scatter_cell_tufts, spawn_tuft_batches,
};
use fabelgeist_determinism::Seed;

#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::presentation) struct PreparedGrass {
    pub playable: PreparedTufts,
    pub vista: PreparedTufts,
}

impl PreparedGrass {
    pub(in crate::presentation) fn new(
        input: &TacticalSceneInput,
        terrain: &SceneTerrain,
        ground: &SceneGround,
        config: &crate::presentation::config::TacticalGraphicsConfig,
    ) -> Self {
        let environment = input.environment_snapshot(input.digest().expect("validated scene"));
        let playable = prepare_scene_tufts(
            terrain,
            ground,
            &environment,
            &config.grass,
            input.landform.as_ref(),
        );
        let lods = input
            .vista
            .lods
            .iter()
            .take(config.rendering.vista.maximum_lods)
            .collect::<Vec<_>>();
        let vista = lods
            .first()
            .map_or_else(TierSpeciesBatches::default, |lod| {
                scatter(
                    lod,
                    lods.get(1).copied(),
                    Vec2::new(terrain.width(), terrain.depth()) * 0.5,
                    terrain,
                    ground,
                    &environment,
                    &config.grass,
                    &UrbanGround::new(&input.streets, &input.yards),
                )
            });
        Self {
            playable: playable.into(),
            vista: vista.into(),
        }
    }
}
/// Instance placement for the vista rings: the same tuft lattice as the
/// playable sward. Owned scenes sample height from accepted physical support;
/// sampled scenes use the presented vista heightfield. Coverage comes from the
/// boundary-stitched ground cover. Sites inside the playable rectangle report
/// bare so the two swards tile without overlapping.
struct VistaTuftPlacement<'a> {
    lod: &'a VistaLod,
    coarser_lod: Option<&'a VistaLod>,
    playable_half_extent: Vec2,
    playable_terrain: &'a SceneTerrain,
    playable_ground: &'a SceneGround,
    urban_ground: &'a UrbanGround,
    profile: GrassCommunityProfile,
    communities: GrassCommunityField,
    /// How far past the playable rectangle this sward reaches, in metres.
    outer_collar: f32,
}

impl TuftPlacement for VistaTuftPlacement<'_> {
    fn lattice_bounds(&self, cell_spacing: f32) -> (IVec2, IVec2) {
        let outer = self.playable_half_extent + Vec2::splat(self.outer_collar);
        (
            (-outer / cell_spacing).floor().as_ivec2(),
            (outer / cell_spacing).ceil().as_ivec2(),
        )
    }

    /// `coverage` clears the playable interior per tuft, which is what keeps
    /// the boundary exact. Rejecting cells that sit wholly inside it as well is
    /// pure speed: those cells would query support height once per tuft only
    /// to discard every one.
    fn cell_allows(&self, _cell_hash: Seed, cell: IVec2, cell_spacing: f32, _jitter: f32) -> bool {
        let centre = cell.as_vec2() * cell_spacing;
        let interior = self.playable_half_extent - Vec2::splat(cell_spacing);
        centre.x.abs() > interior.x || centre.y.abs() > interior.y
    }

    fn coverage(&self, centre: Vec2) -> u8 {
        // The playable sward owns everything inside its rectangle. Gating per
        // tuft rather than per cell keeps the boundary exact, so neither
        // sward doubles up nor leaves a gap along it.
        if centre.x.abs() <= self.playable_half_extent.x
            && centre.y.abs() <= self.playable_half_extent.y
        {
            return 0;
        }
        let coverage = stitched_vista_topology_coverage(
            self.lod,
            self.playable_half_extent,
            self.playable_ground,
            centre,
            self.urban_ground,
        );
        (coverage.clamp(0.0, 1.0) * 255.0) as u8
    }

    fn height(&self, centre: Vec2) -> Option<f32> {
        if self.playable_terrain.property_surface().is_some() {
            let hit = self.playable_terrain.surface_below(
                adventuresim_tactical_core::city_layout::grounding::SupportQuery::unbounded(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                        bevy::math::Vec2::new(centre.x, centre.y),
                    )?,
                ),
            )?;
            return (hit.normal.y >= MINIMUM_GRASS_SLOPE_NORMAL_Y)
                .then_some(hit.elevation.metres());
        }
        let origin = Vec2::new(
            self.lod.origin_east_metres as f32,
            self.lod.origin_north_metres as f32,
        );
        let local = centre - origin;
        let at = |offset: Vec2| {
            presented_vista_vertex_height(
                self.lod,
                self.coarser_lod,
                Some(self.playable_terrain),
                local + offset,
                self.playable_half_extent,
            )
        };
        let height = at(Vec2::ZERO)?;
        // Central differences over the presented surface, matching the slope
        // gate the playable placement takes from `SceneTerrain::normal_at`.
        let delta = 2.0;
        let sample = |offset: Vec2| at(offset).unwrap_or(height);
        let tangent_x = Vec3::new(
            delta * 2.0,
            sample(Vec2::X * delta) - sample(-Vec2::X * delta),
            0.0,
        );
        let tangent_z = Vec3::new(
            0.0,
            sample(Vec2::Y * delta) - sample(-Vec2::Y * delta),
            delta * 2.0,
        );
        let normal = tangent_z.cross(tangent_x).normalize_or_zero();
        (normal.y >= MINIMUM_GRASS_SLOPE_NORMAL_Y).then_some(height)
    }

    fn community(&mut self, centre: Vec2) -> GrassCommunity {
        let profile = sample_vista_environment(self.lod, centre)
            .map_or(self.profile, |sample| self.profile.localized(sample));
        self.communities.at(centre, profile)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn spawn_near_vista_scatter(
    commands: &mut Commands,
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_half_extent: Vec2,
    playable_terrain: &SceneTerrain,
    playable_ground: &SceneGround,
    environment: &SceneEnvironment,
    meshes: &mut Assets<Mesh>,
    grass_materials: &mut Assets<TacticalGrassInstancedMaterial>,
    grass: &crate::presentation::config::GrassConfig,
    urban_ground: &UrbanGround,
) {
    if !grass.enabled {
        return;
    }
    let scene_seed = stable_text_seed(&environment.scene_digest);
    let (grass_color, grass_dryness) = grass_pigment(environment);
    // Blade density has to match the playable sward exactly: the two lattices
    // abut along the playable rectangle, so any difference reads as a seam.
    let pigment = TuftPigment {
        color: grass_color,
        density: grass_scatter_density(
            bps(environment.canopy_bps),
            bps(environment.water_bps),
            bps(environment.cultivation_bps),
            bps(environment.weather.snow_cover_bps),
        ) * grass.density_scale,
        dryness: grass_dryness,
        wind_scale: 0.16 + bps(environment.weather.wind_speed_bps) * 0.36,
    };
    let grass_seed = streams::GRASS.seed(scene_seed, &[]);
    #[cfg(target_family = "wasm")]
    let prepared = match crate::presentation::generation::landscape::grass(
        crate::presentation::generation::GenerationOwner::Scene,
        &environment.scene_digest,
    ) {
        Ok(prepared) => prepared,
        Err(error) => {
            warn!(%error, "Could not access prepared grass");
            return;
        }
    };
    #[cfg(not(target_family = "wasm"))]
    let prepared: Option<std::sync::Arc<PreparedGrass>> = None;
    let mut batches = prepared.map_or_else(
        || {
            scatter(
                lod,
                coarser_lod,
                playable_half_extent,
                playable_terrain,
                playable_ground,
                environment,
                grass,
                urban_ground,
            )
        },
        |p| p.vista.to_batches(),
    );

    spawn_tuft_batches(
        GrassWorld {
            commands,
            meshes,
            materials: grass_materials,
        },
        &mut batches,
        "Vista grass",
        // Shadow casting out here costs cascade budget for contact detail
        // nobody can resolve, so no vista ring tier ever casts.
        (
            VistaTerrain(lod.level),
            VistaGrassPresentation,
            NotShadowCaster,
        ),
        grass_seed,
        pigment,
        grass,
    );
}

/// How far past the playable rectangle a ring tier scatters: its terminal fade
/// distance, since the camera never leaves the rectangle.
fn tier_sward_collar_metres(
    lod: GrassMeshLod,
    grass: &crate::presentation::config::GrassConfig,
) -> f32 {
    let tier = match lod {
        GrassMeshLod::Near => &grass.lod.near,
        GrassMeshLod::NearEdge => &grass.lod.near_edge,
        GrassMeshLod::Far => &grass.lod.far,
        GrassMeshLod::Vista => &grass.lod.vista,
    };
    // The near tier hands off to near-edge, which reuses its placements, so the
    // near lattice has to reach as far as the wider of the two bands.
    tier.fade_out_m[1].max(grass.lod.near_edge.fade_out_m[1])
}

#[expect(
    clippy::too_many_arguments,
    reason = "Shared pure scatter inputs mirror the native rendering boundary"
)]
fn scatter(
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_half_extent: Vec2,
    playable_terrain: &SceneTerrain,
    playable_ground: &SceneGround,
    environment: &SceneEnvironment,
    grass: &crate::presentation::config::GrassConfig,
    urban_ground: &UrbanGround,
) -> TierSpeciesBatches {
    if !grass.enabled {
        return Default::default();
    }
    let grass_seed = streams::GRASS.seed(stable_text_seed(&environment.scene_digest), &[]);
    let profile = GrassCommunityProfile::from_environment(environment);
    let placement = |outer_collar| VistaTuftPlacement {
        lod,
        coarser_lod,
        playable_half_extent,
        playable_terrain,
        playable_ground,
        urban_ground,
        profile,
        communities: GrassCommunityField::new(grass_seed),
        outer_collar,
    };

    // The playable boundary selects the height/cover data source, never the
    // representation. The same globally aligned lattice and distance ranges
    // continue across it, so crossing the boundary cannot introduce an LOD
    // edge or replace blank space with a close representation.
    //
    // Each tier only reaches as far as its own fade-out: the camera stays
    // inside the playable rectangle, so a tuft farther out than the tier's
    // terminal fade distance can never be drawn, and placing one would only
    // cost instance memory and compute-cull work.
    let mut batches = TierSpeciesBatches::default();
    for grass_lod in [GrassMeshLod::Near, GrassMeshLod::Far] {
        scatter_cell_tufts(
            &mut batches[grass_lod.tier_index()],
            &mut placement(tier_sward_collar_metres(grass_lod, grass)),
            grass_seed,
            grass_lod,
            grass.placement.playable_patch_spacing_m,
            grass,
        );
    }
    // Reuse the near placements, exactly as the playable sward does, so the
    // near-edge crossfade morphs each tuft in place instead of moving it.
    for species in GrassSpecies::ALL {
        batches[GrassMeshLod::NearEdge.tier_index()][species.index()] =
            batches[GrassMeshLod::Near.tier_index()][species.index()].clone();
    }
    scatter_cell_tufts(
        &mut batches[GrassMeshLod::Vista.tier_index()],
        &mut placement(tier_sward_collar_metres(GrassMeshLod::Vista, grass)),
        streams::GRASS_LOD.seed(grass_seed, &[]),
        GrassMeshLod::Vista,
        grass.placement.vista_patch_spacing_m,
        grass,
    );

    batches
}

#[cfg(test)]
mod tests;

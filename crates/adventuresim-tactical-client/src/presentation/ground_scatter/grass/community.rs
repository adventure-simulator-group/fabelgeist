//! Spatially coherent grass communities and local habitat selection.
use super::*;
use fabelgeist_determinism::Seed;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::presentation) enum GrassCommunity {
    /// Fertile lowland hay meadow: tall false oat-grass with coarse cocksfoot clumps.
    MesicMeadow,
    /// Leaner or more exposed turf: fine red fescue with airy common bent.
    LeanSward,
    /// Damp meadow and wet woodland gap: tufted hair-grass with Yorkshire fog.
    WetTussock,
}

impl GrassCommunity {
    pub(in crate::presentation) const ALL: [Self; 3] =
        [Self::MesicMeadow, Self::LeanSward, Self::WetTussock];
    pub(in crate::presentation) const COUNT: usize = Self::ALL.len();

    pub(super) const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::presentation) struct GrassCommunityProfile {
    pub(super) weights: [f32; GrassCommunity::COUNT],
}

impl GrassCommunityProfile {
    pub(in crate::presentation) fn from_environment(environment: &SceneEnvironment) -> Self {
        let wet = (bps(environment.wetland_bps)
            + bps(environment.water_bps) * 0.55
            + bps(environment.weather.ground_moisture_bps) * 0.35)
            .clamp(0.0, 1.0);
        let exposed = (bps(environment.hilly_bps) * 0.72
            + (1.0 - bps(environment.cultivation_bps)) * 0.18)
            .clamp(0.0, 1.0);
        Self::from_site_drivers(wet, exposed)
    }

    pub(super) fn from_site_drivers(wet: f32, exposed: f32) -> Self {
        let wet = wet.clamp(0.0, 1.0);
        let exposed = exposed.clamp(0.0, 1.0);
        let mesic = (1.0 - wet) * (1.0 - exposed * 0.55);
        let lean = exposed * (1.0 - wet * 0.65);
        // A dry scene has no token wet-tussock cells. Deschampsia and
        // Yorkshire fog appear only once the available moisture signal is
        // material, while mesic and lean communities trade off continuously.
        let wet_tussock = (wet - 0.18).max(0.0) * 1.25;
        Self {
            weights: [mesic.max(0.001), lean, wet_tussock],
        }
    }

    pub(in crate::presentation) fn localized(self, sample: EnvironmentalSample) -> Self {
        let surface_wet = match sample.surface {
            TacticalSurface::Water | TacticalSurface::Wetland => 1.0,
            _ => 0.0,
        };
        let wet = (bps(sample.wetland_bps) + bps(sample.water_bps) * 0.7 + surface_wet * 0.7)
            .clamp(0.0, 1.0);
        let exposed = (bps(sample.hilly_bps) * 0.8 + (1.0 - bps(sample.cultivation_bps)) * 0.12)
            .clamp(0.0, 1.0);
        let local = Self::from_site_drivers(wet, exposed);
        Self {
            weights: core::array::from_fn(|index| {
                self.weights[index] * 0.35 + local.weights[index] * 0.65
            }),
        }
    }

    #[cfg(test)]
    fn select(self, site_hash: Seed) -> GrassCommunity {
        // Stable low-frequency pseudo-fields stand in for finer soil data we
        // do not yet have. They modulate, but never invent, a habitat that the
        // scene/local environmental sample assigned zero weight.
        let moisture_field =
            0.68 + streams::MOISTURE.rng(site_hash, &[]).inclusive_unit_f32() * 0.64;
        let exposure_field =
            0.68 + streams::EXPOSURE.rng(site_hash, &[]).inclusive_unit_f32() * 0.64;
        let fertility_field =
            0.76 + streams::FERTILITY.rng(site_hash, &[]).inclusive_unit_f32() * 0.48;
        let weights = [
            self.weights[0] * fertility_field,
            self.weights[1] * exposure_field,
            self.weights[2] * moisture_field,
        ];
        // Habitat interpolation is spatial math; quantize only the final
        // selection weights, preserving hard-zero exclusions.
        let weights = weights.map(|weight| {
            (weight * adventuresim_world_schema::BASIS_POINTS_PER_WHOLE as f32).round() as u64
        });
        let selected = streams::COMMUNITY_SPECIES
            .rng(site_hash, &[])
            .weighted_index(&weights)
            .expect("a grass habitat always has a positive mesic weight");
        [
            GrassCommunity::MesicMeadow,
            GrassCommunity::LeanSward,
            GrassCommunity::WetTussock,
        ][selected]
    }
}

#[cfg(test)]
pub(in crate::presentation) fn grass_community_at(
    point: Vec2,
    seed: Seed,
    profile: GrassCommunityProfile,
) -> GrassCommunity {
    // Jittered Voronoi cells create coherent 12-40 m sward communities. A
    // patch selects one community; species vary within that community rather
    // than becoming independent blade-by-blade confetti.
    const CELL_SIZE: f32 = 24.0;
    let cell = (point / CELL_SIZE).floor().as_ivec2();
    let mut nearest_distance = f32::INFINITY;
    let mut nearest_hash = Seed::from_u64(0);
    for offset_z in -1..=1 {
        for offset_x in -1..=1 {
            let candidate = cell + bevy::math::IVec2::new(offset_x, offset_z);
            let hash = streams::COMMUNITY.seed(
                seed,
                &[candidate.x as u32 as u64, candidate.y as u32 as u64],
            );
            let site = (candidate.as_vec2()
                + Vec2::new(
                    0.18 + streams::JITTER_X.rng(hash, &[]).inclusive_unit_f32() * 0.64,
                    0.18 + streams::JITTER_Z.rng(hash, &[]).inclusive_unit_f32() * 0.64,
                ))
                * CELL_SIZE;
            let distance = point.distance_squared(site);
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_hash = hash;
            }
        }
    }
    profile.select(nearest_hash)
}

/// Temporary habitat lattice shared by all tufts sampled during a scatter pass.
/// Site positions and random fields depend on the seed, never the local profile.
pub(in crate::presentation) struct GrassCommunityField {
    seed: Seed,
    sites: std::collections::BTreeMap<(i32, i32), CommunitySite>,
    neighbourhood: Option<(bevy::math::IVec2, [CommunitySite; 9])>,
}

#[derive(Clone, Copy)]
struct CommunitySite {
    position: Vec2,
    modulation: [f32; GrassCommunity::COUNT],
    selection: fabelgeist_determinism::Seed,
}

impl CommunitySite {
    fn new(cell: bevy::math::IVec2, seed: Seed) -> Self {
        let hash = streams::COMMUNITY.seed(seed, &[cell.x as u32 as u64, cell.y as u32 as u64]);
        Self {
            position: (cell.as_vec2()
                + Vec2::new(
                    0.18 + streams::JITTER_X.rng(hash, &[]).inclusive_unit_f32() * 0.64,
                    0.18 + streams::JITTER_Z.rng(hash, &[]).inclusive_unit_f32() * 0.64,
                ))
                * COMMUNITY_CELL_SIZE_METRES,
            modulation: [
                0.76 + streams::FERTILITY.rng(hash, &[]).inclusive_unit_f32() * 0.48,
                0.68 + streams::EXPOSURE.rng(hash, &[]).inclusive_unit_f32() * 0.64,
                0.68 + streams::MOISTURE.rng(hash, &[]).inclusive_unit_f32() * 0.64,
            ],
            selection: streams::COMMUNITY_SPECIES.seed(hash, &[]),
        }
    }
    fn select(self, profile: GrassCommunityProfile) -> GrassCommunity {
        let weights = core::array::from_fn::<_, { GrassCommunity::COUNT }, _>(|i| {
            (profile.weights[i]
                * self.modulation[i]
                * adventuresim_world_schema::BASIS_POINTS_PER_WHOLE as f32)
                .round() as u64
        });
        GrassCommunity::ALL[self
            .selection
            .rng()
            .weighted_index(&weights)
            .expect("a grass habitat always has a positive mesic weight")]
    }
}

const COMMUNITY_CELL_SIZE_METRES: f32 = 24.0;

impl GrassCommunityField {
    pub(in crate::presentation) fn new(seed: Seed) -> Self {
        Self {
            seed,
            sites: Default::default(),
            neighbourhood: None,
        }
    }

    pub(in crate::presentation) fn at(
        &mut self,
        point: Vec2,
        profile: GrassCommunityProfile,
    ) -> GrassCommunity {
        let cell = (point / COMMUNITY_CELL_SIZE_METRES).floor().as_ivec2();
        if self
            .neighbourhood
            .as_ref()
            .is_none_or(|(previous, _)| *previous != cell)
        {
            let sites = std::array::from_fn(|i| {
                let candidate = cell + bevy::math::IVec2::new(i as i32 % 3 - 1, i as i32 / 3 - 1);
                *self
                    .sites
                    .entry((candidate.x, candidate.y))
                    .or_insert_with(|| CommunitySite::new(candidate, self.seed))
            });
            self.neighbourhood = Some((cell, sites));
        }
        let sites = &self.neighbourhood.as_ref().unwrap().1;
        let mut nearest = &sites[0];
        let mut distance = f32::INFINITY;
        for site in sites {
            let candidate = point.distance_squared(site.position);
            if candidate < distance {
                distance = candidate;
                nearest = site;
            }
        }
        nearest.select(profile)
    }
}

#[test]
fn cached_habitat_matches_uncached_across_cells_seeds_and_profiles() {
    for seed in ([0, 42, u64::MAX])
        .into_iter()
        .map(fabelgeist_determinism::Seed::from_u64)
    {
        let mut field = GrassCommunityField::new(seed);
        for z in -16..=16 {
            for x in -16..=16 {
                let point = Vec2::new(x as f32 * 6.0, z as f32 * 6.0);
                for profile in [
                    GrassCommunityProfile::from_site_drivers(0.0, 0.0),
                    GrassCommunityProfile::from_site_drivers(1.0, 0.4),
                    GrassCommunityProfile::from_site_drivers(0.3, 0.9),
                ] {
                    assert_eq!(
                        field.at(point, profile),
                        grass_community_at(point, seed, profile)
                    );
                }
            }
        }
        assert!(field.sites.len() < 200);
    }
}

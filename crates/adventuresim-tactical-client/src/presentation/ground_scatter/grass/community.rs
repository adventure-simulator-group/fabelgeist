//! Spatially coherent grass communities and local habitat selection.
use super::*;

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

    fn select(self, site_hash: u64) -> GrassCommunity {
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

pub(in crate::presentation) fn grass_community_at(
    point: Vec2,
    seed: u64,
    profile: GrassCommunityProfile,
) -> GrassCommunity {
    // Jittered Voronoi cells create coherent 12-40 m sward communities. A
    // patch selects one community; species vary within that community rather
    // than becoming independent blade-by-blade confetti.
    const CELL_SIZE: f32 = 24.0;
    let cell = (point / CELL_SIZE).floor().as_ivec2();
    let mut nearest_distance = f32::INFINITY;
    let mut nearest_hash = 0;
    for offset_z in -1..=1 {
        for offset_x in -1..=1 {
            let candidate = cell + bevy::math::IVec2::new(offset_x, offset_z);
            let hash = streams::COMMUNITY
                .seed(
                    seed,
                    &[candidate.x as u32 as u64, candidate.y as u32 as u64],
                )
                .to_u64();
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

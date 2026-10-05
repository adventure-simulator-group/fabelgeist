//! Shade each immutable environmental sample once per ring construction.
//! Bilinear, LOD and playable-edge interpolation retain their original order.
//! The fields are local to one mesh preparation and its exact weather snapshot.
use super::*;

pub(super) struct VistaVertexColors<'a> {
    own: SampleColors<'a>,
    coarser: Option<SampleColors<'a>>,
    playable: Option<Vec4>,
}

impl<'a> VistaVertexColors<'a> {
    pub(super) fn new(
        lod: &'a VistaLod,
        coarser: Option<&'a VistaLod>,
        environment: Option<&SceneEnvironment>,
        weather: WeatherSnapshot,
    ) -> Self {
        Self {
            own: SampleColors::new(lod, weather),
            coarser: coarser.map(|lod| SampleColors::new(lod, weather)),
            playable: environment.map(|environment| {
                Vec4::from_array(scene_ground_color(environment).to_linear().to_f32_array())
            }),
        }
    }

    pub(super) fn at(&self, local: Vec2, playable_half_extent: Vec2) -> Option<[f32; 4]> {
        let lod = self.own.lod;
        let world = local
            + Vec2::new(
                lod.origin_east_metres as f32,
                lod.origin_north_metres as f32,
            );
        let own = self.own.at(world)?;
        let vista_color = self
            .coarser
            .as_ref()
            .map(|coarser| {
                let weight = lod_transition_weight(lod, coarser.lod, world);
                coarser
                    .at(world)
                    .map(|color| own.lerp(color, weight))
                    .unwrap_or(own)
            })
            .unwrap_or(own);
        Some(
            self.playable
                .map(|playable| {
                    // Cover proportions summarize wider ecological patches than height
                    // samples. Keep the existing four-cell pigment transition.
                    stitch_vista_color_to_playable_edge(
                        local,
                        playable_half_extent,
                        lod.spacing_metres * 4.0,
                        vista_color,
                        playable,
                    )
                })
                .unwrap_or(vista_color)
                .to_array(),
        )
    }
}

struct SampleColors<'a> {
    lod: &'a VistaLod,
    colors: Vec<Vec4>,
}

impl<'a> SampleColors<'a> {
    fn new(lod: &'a VistaLod, weather: WeatherSnapshot) -> Self {
        Self {
            lod,
            colors: lod
                .environment
                .iter()
                .map(|sample| vista_sample_color(*sample, weather))
                .collect(),
        }
    }

    fn at(&self, world: Vec2) -> Option<Vec4> {
        let lod = self.lod;
        let width = usize::from(lod.width);
        let depth = usize::from(lod.depth);
        let local = world
            - Vec2::new(
                lod.origin_east_metres as f32,
                lod.origin_north_metres as f32,
            );
        let coordinate = local / lod.spacing_metres
            + Vec2::new((width - 1) as f32 * 0.5, (depth - 1) as f32 * 0.5);
        if coordinate.x < 0.0
            || coordinate.y < 0.0
            || coordinate.x > (width - 1) as f32
            || coordinate.y > (depth - 1) as f32
        {
            return None;
        }
        let lower = coordinate.floor().as_uvec2();
        let upper = (lower + UVec2::ONE).min(UVec2::new(width as u32 - 1, depth as u32 - 1));
        let fraction = coordinate.fract();
        let at = |x: u32, z: u32| self.colors[z as usize * width + x as usize];
        let near = at(lower.x, lower.y).lerp(at(upper.x, lower.y), fraction.x);
        let far = at(lower.x, upper.y).lerp(at(upper.x, upper.y), fraction.x);
        Some(near.lerp(far, fraction.y))
    }
}

pub(super) fn stitch_vista_color_to_playable_edge(
    local: Vec2,
    playable_half_extent: Vec2,
    transition_width: f32,
    vista_color: Vec4,
    playable_color: Vec4,
) -> Vec4 {
    let outside_distance = (local.abs() - playable_half_extent)
        .max(Vec2::ZERO)
        .max_element();
    let vista_weight = (outside_distance / transition_width.max(f32::EPSILON)).clamp(0.0, 1.0);
    let mut stitched = playable_color.lerp(vista_color, vista_weight);
    // Alpha carries distant geometric-sward coverage, not material opacity.
    // Preserve it while blending only the molded substrate pigment.
    stitched.w = vista_color.w;
    stitched
}

pub(super) fn vista_sample_color(sample: EnvironmentalSample, weather: WeatherSnapshot) -> Vec4 {
    let environment = SceneEnvironment {
        scene_digest: String::new(),
        generation_version: TACTICAL_SCENE_GENERATION_VERSION,
        latitude_microdegrees: 53_500_000,
        longitude_microdegrees: 10_000_000,
        absolute_minute: adventuresim_world_schema::calendar::StrategicMinute::ZERO
            .saturating_add_minutes(12 * 60),
        lunar_phase_minute: adventuresim_world_schema::calendar::StrategicMinute::ZERO
            .saturating_add_minutes(12 * 60),
        absolute_elevation_metres: 20,
        weather,
        canopy_bps: sample.canopy_bps,
        wetland_bps: sample.wetland_bps,
        cultivation_bps: sample.cultivation_bps,
        water_bps: sample.water_bps,
        hilly_bps: sample.hilly_bps,
    };
    let mut color = Vec4::from_array(scene_ground_color(&environment).to_linear().to_f32_array());
    let hills = bps(sample.hilly_bps);
    let snow = bps(weather.snow_cover_bps);
    let exposed_rock = hills
        * (1.0 - bps(sample.water_bps))
        * (1.0 - bps(sample.wetland_bps) * 0.8)
        * (1.0 - bps(sample.canopy_bps) * 0.45)
        * (1.0 - snow);
    let rock = Color::srgb_u8(104, 101, 91).to_linear().to_f32_array();
    color = color.lerp(Vec4::from_array(rock), exposed_rock * 0.62);
    color.w = vista_sward_coverage(sample) * (1.0 - snow * 0.92);
    color
}

pub(super) fn vista_sward_coverage(sample: EnvironmentalSample) -> f32 {
    let surface = match sample.surface {
        TacticalSurface::Open | TacticalSurface::SparseWoods => 1.0,
        TacticalSurface::DeepWoods => 0.28,
        TacticalSurface::Wetland => 0.42,
        TacticalSurface::Road | TacticalSurface::Water => 0.0,
    };
    (surface
        * (1.0 - bps(sample.water_bps))
        * (1.0 - bps(sample.cultivation_bps) * 0.72)
        * (1.0 - bps(sample.hilly_bps) * 0.82))
        .clamp(0.0, 1.0)
}

#[cfg(test)]
pub(super) fn presented_color(
    lod: &VistaLod,
    x: usize,
    z: usize,
    world: Vec2,
    coarser_lod: Option<&VistaLod>,
    weather: WeatherSnapshot,
) -> [f32; 4] {
    let own = vista_sample_color(lod.environment[z * usize::from(lod.width) + x], weather);
    let Some(coarser) = coarser_lod else {
        return own.to_array();
    };
    let weight = lod_transition_weight(lod, coarser, world);
    SampleColors::new(coarser, weather)
        .at(world)
        .map(|color| own.lerp(color, weight))
        .unwrap_or(own)
        .to_array()
}

#[cfg(test)]
mod tests;

use super::*;

fn soil_random(
    params: &crate::TextureParameters,
    cell_x: i32,
    cell_y: i32,
    period: i32,
    salt: u64,
) -> f32 {
    let wrapped_x = cell_x.rem_euclid(period) as u64;
    let wrapped_y = cell_y.rem_euclid(period) as u64;
    let hash = crate::parameters::seeded_hash(
        params,
        wrapped_x | (wrapped_y << 16) | salt.rotate_left(33),
    );
    unit_hash(hash)
}

fn soil_value_noise(
    params: &crate::TextureParameters,
    point: Vec2,
    frequency: i32,
    salt: u64,
) -> f32 {
    let scaled = point * frequency as f32;
    let cell = scaled.floor().as_ivec2();
    let local = scaled - cell.as_vec2();
    let blend = local * local * (Vec2::splat(3.0) - local * 2.0);
    let sample = |x: i32, y: i32| soil_random(params, cell.x + x, cell.y + y, frequency, salt);
    let lower = sample(0, 0).lerp(sample(1, 0), blend.x);
    let upper = sample(0, 1).lerp(sample(1, 1), blend.x);
    lower.lerp(upper, blend.y) * 2.0 - 1.0
}

#[derive(Clone, Copy, Debug)]
struct SoilClusterRecipe {
    grid: i32,
    salt: u64,
    density: f32,
    base_radius: f32,
    radius_span: f32,
    spread: f32,
    sharpness: f32,
}

impl SoilClusterRecipe {
    fn sample(self, params: &crate::TextureParameters, point: Vec2) -> f32 {
        let scaled = point * self.grid as f32;
        let base_cell = scaled.floor().as_ivec2();
        let mut field = 0.0_f32;
        for offset_y in -1..=1 {
            for offset_x in -1..=1 {
                let cell = base_cell + IVec2::new(offset_x, offset_y);
                let enabled = soil_random(params, cell.x, cell.y, self.grid, self.salt ^ 0x5de3);
                if enabled > self.density + 0.14 {
                    continue;
                }
                let activation = smoothstep(
                    enabled - params.ground.sample_activation_1,
                    enabled + params.ground.sample_activation_2,
                    self.density,
                );
                let centre = cell.as_vec2()
                    + Vec2::new(
                        params.ground.sample_centre_1
                            + soil_random(params, cell.x, cell.y, self.grid, self.salt ^ 0x13a7)
                                * params.ground.sample_centre_2,
                        params.ground.sample_centre_3
                            + soil_random(params, cell.x, cell.y, self.grid, self.salt ^ 0x91cb)
                                * params.ground.sample_centre_4,
                    );
                let parent_angle =
                    soil_random(params, cell.x, cell.y, self.grid, self.salt ^ 0xc72d)
                        * core::f32::consts::TAU;
                let child_count = 2
                    + (soil_random(params, cell.x, cell.y, self.grid, self.salt ^ 0x314f)
                        * params.ground.sample_child_count)
                        .floor()
                        .min(3.0) as u32;
                let mut cluster = 0.0_f32;
                for child in 0..child_count {
                    let child_salt = self.salt ^ (u64::from(child) + 1).wrapping_mul(0x9e37);
                    let child_angle = parent_angle
                        + core::f32::consts::TAU
                            * soil_random(params, cell.x, cell.y, self.grid, child_salt ^ 0x7b21);
                    let radial = if child == 0 {
                        0.0
                    } else {
                        self.spread
                            * (params.ground.sample_radial_1
                                + soil_random(
                                    params,
                                    cell.x,
                                    cell.y,
                                    self.grid,
                                    child_salt ^ 0x2ad9,
                                ) * params.ground.sample_radial_2)
                    };
                    let child_centre =
                        centre + Vec2::new(child_angle.cos(), child_angle.sin()) * radial;
                    let axis_angle = parent_angle
                        + (soil_random(params, cell.x, cell.y, self.grid, child_salt ^ 0x8f61)
                            - 0.5)
                            * params.ground.sample_axis_angle;
                    let axis = Vec2::new(axis_angle.cos(), axis_angle.sin());
                    let delta = scaled - child_centre;
                    let local = Vec2::new(delta.dot(axis), delta.perp_dot(axis));
                    let radius = self.base_radius
                        + self.radius_span
                            * soil_random(params, cell.x, cell.y, self.grid, child_salt ^ 0x27f1);
                    let aspect = params.ground.sample_aspect_1
                        + soil_random(params, cell.x, cell.y, self.grid, child_salt ^ 0xe419)
                            * params.ground.sample_aspect_2;
                    let normalized = Vec2::new(local.x / radius, local.y / (radius * aspect));
                    let angle = normalized.y.atan2(normalized.x);
                    let first_lobes = 3.0
                        + (soil_random(params, cell.x, cell.y, self.grid, child_salt ^ 0x6bd3)
                            * 3.0)
                            .floor();
                    let second_lobes = first_lobes + 2.0;
                    let phase = soil_random(params, cell.x, cell.y, self.grid, child_salt ^ 0x41af)
                        * core::f32::consts::TAU;
                    let edge_warp = 1.0
                        + params.ground.sample_edge_warp_1 * (angle * first_lobes + phase).sin()
                        + params.ground.sample_edge_warp_2
                            * (angle * second_lobes - phase * params.ground.sample_edge_warp_3)
                                .sin();
                    let distance = normalized.length() * edge_warp;
                    let lump = (1.0 - smoothstep(params.ground.sample_lump, 1.0, distance))
                        .powf(self.sharpness)
                        .clamp(0.0, 1.0);
                    // Probabilistic union preserves rounded sub-lumps while
                    // smoothly fusing their contacts into one aggregate.
                    cluster = 1.0 - (1.0 - cluster) * (1.0 - lump);
                }
                field = 1.0 - (1.0 - field) * (1.0 - cluster * activation);
            }
        }
        field.clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug)]
struct ForestSoilConditions {
    compaction: f32,
    moisture: f32,
}

fn forest_soil_detached_relief(
    params: &crate::TextureParameters,
    sample: Vec2,
    conditions: ForestSoilConditions,
    aggregate_union: f32,
) -> (f32, f32) {
    let loose_dry = ((1.0 - conditions.compaction) * (1.0 - conditions.moisture))
        .powf(params.ground.forest_soil_detached_relief_loose_dry);
    let contact_band = smoothstep(
        params.ground.forest_soil_detached_relief_contact_band_1,
        params.ground.forest_soil_detached_relief_contact_band_2,
        aggregate_union,
    ) * (1.0
        - smoothstep(
            params.ground.forest_soil_detached_relief_contact_band_3,
            params.ground.forest_soil_detached_relief_contact_band_4,
            aggregate_union,
        ));
    let crumb_clusters = SoilClusterRecipe {
        grid: 68,
        salt: 0x6f2b,
        density: params.ground.forest_soil_detached_relief_crumb_clusters_1
            + loose_dry * params.ground.forest_soil_detached_relief_crumb_clusters_2,
        base_radius: params.ground.forest_soil_detached_relief_crumb_clusters_3,
        radius_span: params.ground.forest_soil_detached_relief_crumb_clusters_4,
        spread: params.ground.forest_soil_detached_relief_crumb_clusters_5,
        sharpness: params.ground.forest_soil_detached_relief_crumb_clusters_6,
    }
    .sample(params, sample);
    let crumbs = crumb_clusters * contact_band * loose_dry;
    let granular = soil_value_noise(params, sample, 97, 0xf28b)
        * params.ground.forest_soil_detached_relief_granular
        * loose_dry;
    (crumbs, granular)
}

fn forest_soil_context(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
) -> (Vec2, f32, ForestSoilConditions) {
    let point = Vec2::new(u, v);
    let warp = Vec2::new(
        soil_value_noise(params, point, 5, 0x8ae1),
        soil_value_noise(
            params,
            point
                + Vec2::new(
                    params.ground.forest_soil_context_warp_1,
                    params.ground.forest_soil_context_warp_2,
                ),
            5,
            0x42d7,
        ),
    ) * params.ground.forest_soil_context_warp_3;
    let sample = point + warp;
    let broad = soil_value_noise(params, sample, 3, 0x7c31)
        * params.ground.forest_soil_context_broad_1
        + soil_value_noise(params, sample, 7, 0xb527) * params.ground.forest_soil_context_broad_2;
    let compaction = smoothstep(
        -params.ground.forest_soil_context_compaction_1,
        params.ground.forest_soil_context_compaction_2,
        soil_value_noise(params, sample, 3, 0x1d93),
    );
    let moisture = smoothstep(
        -params.ground.forest_soil_context_moisture_1,
        params.ground.forest_soil_context_moisture_2,
        soil_value_noise(
            params,
            sample
                + Vec2::new(
                    params.ground.forest_soil_context_moisture_3,
                    params.ground.forest_soil_context_moisture_4,
                ),
            4,
            0x4bd1,
        ) - broad * params.ground.forest_soil_context_moisture_5,
    );
    (
        sample,
        broad,
        ForestSoilConditions {
            compaction,
            moisture,
        },
    )
}

/// Exactly periodic two-metre forest-floor relief. Broad compaction and
/// moisture patches govern the scale and sharpness of the aggregate instead
/// of merely tinting otherwise identical noise.
fn forest_soil_sample(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    let (sample, broad, conditions) = forest_soil_context(params, u, v);
    let loose_soil = 1.0 - conditions.compaction;

    // Parent clusters fuse two to five sub-lumps into irregular aggregates.
    // Moisture grows and softens the fused masses; compaction reduces their
    // count instead of leaving the same detached stamps at lower amplitude.
    let hollows = SoilClusterRecipe {
        grid: 10,
        salt: 0xd1a9,
        density: params.ground.forest_soil_sample_hollows_1
            + loose_soil * params.ground.forest_soil_sample_hollows_2
            + conditions.moisture * params.ground.forest_soil_sample_hollows_3,
        base_radius: params.ground.forest_soil_sample_hollows_4
            + conditions.moisture * params.ground.forest_soil_sample_hollows_5,
        radius_span: params.ground.forest_soil_sample_hollows_6,
        spread: params.ground.forest_soil_sample_hollows_7
            - conditions.moisture * params.ground.forest_soil_sample_hollows_8,
        sharpness: params.ground.forest_soil_sample_hollows_9
            - conditions.moisture * params.ground.forest_soil_sample_hollows_10,
    }
    .sample(params, sample);
    let cohesive_clods = SoilClusterRecipe {
        grid: 15,
        salt: 0x39e7,
        density: params.ground.forest_soil_sample_cohesive_clods_1
            + loose_soil * params.ground.forest_soil_sample_cohesive_clods_2
            + conditions.moisture * params.ground.forest_soil_sample_cohesive_clods_3,
        base_radius: params.ground.forest_soil_sample_cohesive_clods_4
            + conditions.moisture * params.ground.forest_soil_sample_cohesive_clods_5,
        radius_span: params.ground.forest_soil_sample_cohesive_clods_6,
        spread: params.ground.forest_soil_sample_cohesive_clods_7
            - conditions.moisture * params.ground.forest_soil_sample_cohesive_clods_8,
        sharpness: params.ground.forest_soil_sample_cohesive_clods_9
            - conditions.moisture * params.ground.forest_soil_sample_cohesive_clods_10,
    }
    .sample(params, sample);
    let aggregate = SoilClusterRecipe {
        grid: 34,
        salt: 0xa613,
        density: params.ground.forest_soil_sample_aggregate_1
            + loose_soil * params.ground.forest_soil_sample_aggregate_2
            + conditions.moisture * params.ground.forest_soil_sample_aggregate_3,
        base_radius: params.ground.forest_soil_sample_aggregate_4
            + conditions.moisture * params.ground.forest_soil_sample_aggregate_5,
        radius_span: params.ground.forest_soil_sample_aggregate_6,
        spread: params.ground.forest_soil_sample_aggregate_7
            - conditions.moisture * params.ground.forest_soil_sample_aggregate_8,
        sharpness: params.ground.forest_soil_sample_aggregate_9
            - conditions.moisture * params.ground.forest_soil_sample_aggregate_10,
    }
    .sample(params, sample);

    let aggregate_union = 1.0 - (1.0 - cohesive_clods) * (1.0 - aggregate);
    let (crumbs, granular) =
        forest_soil_detached_relief(params, sample, conditions, aggregate_union);

    // Pores belong to saddles between fused aggregates. They are not a second
    // independently stamped population that can float in smooth soil.
    let saddle = smoothstep(
        params.ground.forest_soil_sample_saddle_1,
        params.ground.forest_soil_sample_saddle_2,
        aggregate_union,
    ) * (1.0
        - smoothstep(
            params.ground.forest_soil_sample_saddle_3,
            params.ground.forest_soil_sample_saddle_4,
            aggregate_union,
        ));
    let pore_breakup = smoothstep(
        -params.ground.forest_soil_sample_pore_breakup_1,
        params.ground.forest_soil_sample_pore_breakup_2,
        -soil_value_noise(
            params,
            sample
                + Vec2::new(
                    params.ground.forest_soil_sample_pore_breakup_3,
                    params.ground.forest_soil_sample_pore_breakup_4,
                ),
            53,
            0x2cf5,
        ),
    );
    let pores = saddle
        * pore_breakup
        * (params.ground.forest_soil_sample_pores_1
            + loose_soil * params.ground.forest_soil_sample_pores_2);
    let clod_strength = (params.ground.forest_soil_sample_clod_strength_1
        + loose_soil * params.ground.forest_soil_sample_clod_strength_2)
        * (params.ground.forest_soil_sample_clod_strength_3
            + conditions.moisture * params.ground.forest_soil_sample_clod_strength_4);
    let height = broad
        - hollows
            * (params.ground.forest_soil_sample_height_1
                + conditions.moisture * params.ground.forest_soil_sample_height_2)
        + cohesive_clods * params.ground.forest_soil_sample_height_3 * clod_strength
        + aggregate
            * params.ground.forest_soil_sample_height_4
            * (params.ground.forest_soil_sample_height_5
                + loose_soil * params.ground.forest_soil_sample_height_6)
        + crumbs * params.ground.forest_soil_sample_height_7
        - pores * params.ground.forest_soil_sample_height_8
        + granular
        - conditions.moisture * params.ground.forest_soil_sample_height_9;

    height.clamp(-0.42, 0.46)
}

pub(super) fn forest_soil_height(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    forest_soil_sample(params, u, v)
}

pub(super) fn forest_soil_horizon_ao(
    params: &crate::TextureParameters,
    field: &[f32],
    x: i32,
    y: i32,
) -> f32 {
    let source_scale =
        (params.size(FOREST_SOIL_TEXTURE_SIZE) / params.size(FOREST_SOIL_AO_SIZE)) as i32;
    let source_x = x * source_scale + source_scale / 2;
    let source_y = y * source_scale + source_scale / 2;
    let centre = periodic_sample(
        field,
        params.size(FOREST_SOIL_TEXTURE_SIZE),
        source_x,
        source_y,
    ) * params.common.forest_soil_height_range_metres;
    let ao_texel_metres =
        params.common.forest_soil_tile_metres / params.size(FOREST_SOIL_AO_SIZE) as f32;
    let mut visibility = 0.0;
    for (direction_x, direction_y) in FOREST_SOIL_AO_DIRECTIONS {
        let mut maximum_slope = 0.0_f32;
        for ao_step in FOREST_SOIL_AO_STEPS {
            let source_step = ao_step * source_scale;
            let neighbor = periodic_sample(
                field,
                params.size(FOREST_SOIL_TEXTURE_SIZE),
                source_x + direction_x * source_step,
                source_y + direction_y * source_step,
            ) * params.common.forest_soil_height_range_metres;
            let run = ao_step as f32 * ao_texel_metres;
            maximum_slope = maximum_slope.max(((neighbor - centre) / run).max(0.0));
        }
        visibility += 1.0 / (1.0 + maximum_slope * maximum_slope).sqrt();
    }
    (visibility / FOREST_SOIL_AO_DIRECTIONS.len() as f32).clamp(0.55, 1.0)
}

pub(super) fn forest_soil_local_cavity(
    params: &crate::TextureParameters,
    field: &[f32],
    x: i32,
    y: i32,
) -> f32 {
    let centre = periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x, y);
    let immediate_neighbors =
        periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x - 1, y)
            + periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x + 1, y)
            + periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x, y - 1)
            + periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x, y + 1);
    let aggregate_neighbors =
        periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x - 4, y)
            + periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x + 4, y)
            + periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x, y - 4)
            + periodic_sample(field, params.size(FOREST_SOIL_TEXTURE_SIZE), x, y + 4);
    let immediate_cavity =
        (immediate_neighbors * params.ground.forest_soil_local_cavity_immediate_cavity - centre)
            .max(0.0);
    let aggregate_cavity =
        (aggregate_neighbors * params.ground.forest_soil_local_cavity_aggregate_cavity - centre)
            .max(0.0);
    let cavity = immediate_cavity * params.ground.forest_soil_local_cavity_cavity_1
        + aggregate_cavity * params.ground.forest_soil_local_cavity_cavity_2;
    (1.0 - cavity * 2.5).clamp(0.78, 1.0)
}

#[derive(Clone, Copy, Debug, Default)]
struct LitterLeafImprint {
    pub(super) coverage: f32,
    dome: f32,
    tone: f32,
    vein: f32,
    edge: f32,
    contact: f32,
    order: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LitterShapeClass {
    IntactOak,
    CurledOak,
    HalfLeaf,
    TornFragment,
    Skeleton,
    Humified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LitterStratum {
    Lower,
    Middle,
    Upper,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LitterDetail {
    Full,
    Mid,
    Far,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct LitterStratumRecipe {
    stratum: LitterStratum,
    grid: i32,
    salt: u64,
    density: f32,
    minimum_radius: f32,
    radius_span: f32,
    minimum_aspect: f32,
    aspect_span: f32,
    decomposition: f32,
}

const LITTER_STRATA: [LitterStratumRecipe; 3] = [
    LitterStratumRecipe {
        stratum: LitterStratum::Lower,
        grid: 40,
        salt: 0x1eaf_0001,
        density: 0.86,
        minimum_radius: 0.34,
        radius_span: 0.30,
        minimum_aspect: 0.55,
        aspect_span: 0.40,
        decomposition: 0.92,
    },
    LitterStratumRecipe {
        stratum: LitterStratum::Middle,
        grid: 28,
        salt: 0x1eaf_0002,
        density: 0.68,
        minimum_radius: 0.46,
        radius_span: 0.28,
        minimum_aspect: 0.72,
        aspect_span: 0.30,
        decomposition: 0.56,
    },
    LitterStratumRecipe {
        stratum: LitterStratum::Upper,
        grid: 21,
        salt: 0x1eaf_0003,
        density: 0.48,
        minimum_radius: 0.43,
        radius_span: 0.18,
        minimum_aspect: 0.82,
        aspect_span: 0.28,
        decomposition: 0.18,
    },
];

#[derive(Clone, Copy, Debug, Default)]
struct LitterShapeSample {
    coverage: f32,
    dome: f32,
    vein: f32,
    edge: f32,
    lift: f32,
}

fn litter_shape_class(
    params: &crate::TextureParameters,
    cell: IVec2,
    recipe: LitterStratumRecipe,
) -> LitterShapeClass {
    let selector = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x23e1);
    match recipe.stratum {
        LitterStratum::Lower if selector < 0.55 => LitterShapeClass::Humified,
        LitterStratum::Lower if selector < 0.85 => LitterShapeClass::TornFragment,
        LitterStratum::Lower => LitterShapeClass::Skeleton,
        LitterStratum::Middle if selector < 0.40 => LitterShapeClass::TornFragment,
        LitterStratum::Middle if selector < 0.58 => LitterShapeClass::HalfLeaf,
        LitterStratum::Middle if selector < 0.70 => LitterShapeClass::CurledOak,
        LitterStratum::Middle if selector < 0.88 => LitterShapeClass::Skeleton,
        LitterStratum::Middle => LitterShapeClass::IntactOak,
        LitterStratum::Upper if selector < 0.65 => LitterShapeClass::IntactOak,
        LitterStratum::Upper if selector < 0.80 => LitterShapeClass::CurledOak,
        LitterStratum::Upper if selector < 0.95 => LitterShapeClass::HalfLeaf,
        LitterStratum::Upper => LitterShapeClass::TornFragment,
    }
}

fn broad_oak_width(params: &crate::TextureParameters, t: f32, phase: f32, side: f32) -> f32 {
    let blade = (t * core::f32::consts::PI)
        .sin()
        .max(0.0)
        .powf(params.ground.broad_oak_width_blade);
    let basal_ears = smoothstep(0.0, params.ground.broad_oak_width_basal_ears_1, t)
        * (1.0
            - smoothstep(
                params.ground.broad_oak_width_basal_ears_2,
                params.ground.broad_oak_width_basal_ears_3,
                t,
            ));
    let side_phase = if side < 0.0 {
        phase * params.ground.broad_oak_width_side_phase_1
            + params.ground.broad_oak_width_side_phase_2
    } else {
        phase
    };
    let lobe_count = if side < 0.0 {
        params.ground.broad_oak_width_lobe_count_1
    } else {
        params.ground.broad_oak_width_lobe_count_2
    };
    let rounded_lobes = params.ground.broad_oak_width_rounded_lobes_1
        + params.ground.broad_oak_width_rounded_lobes_2
            * (0.5 + 0.5 * (t * lobe_count * core::f32::consts::TAU + side_phase).cos())
                .powf(params.ground.broad_oak_width_rounded_lobes_3);
    let side_bias = if side < 0.0 {
        params.ground.broad_oak_width_side_bias_1
    } else {
        params.ground.broad_oak_width_side_bias_2
    };
    blade * rounded_lobes * side_bias + basal_ears * if side < 0.0 { 0.11 } else { 0.20 }
}

fn oak_tissue_sample(
    params: &crate::TextureParameters,
    local: Vec2,
    radius: f32,
    aspect: f32,
    phase: f32,
) -> LitterShapeSample {
    let longitudinal = local.x / radius;
    if longitudinal.abs() >= 1.0 {
        return LitterShapeSample::default();
    }
    let t = (longitudinal + 1.0) * 0.5;
    let half_width = radius * aspect * broad_oak_width(params, t, phase, local.y.signum());
    let signed_lateral = local.y / half_width.max(params.ground.oak_tissue_sample_signed_lateral);
    let lateral = signed_lateral.abs();
    let coverage = 1.0
        - smoothstep(
            params.ground.oak_tissue_sample_coverage_1,
            params.ground.oak_tissue_sample_coverage_2,
            lateral,
        );
    let ordinary_edge = smoothstep(
        params.ground.oak_tissue_sample_ordinary_edge_1,
        params.ground.oak_tissue_sample_ordinary_edge_2,
        lateral,
    ) * (1.0
        - smoothstep(
            params.ground.oak_tissue_sample_ordinary_edge_3,
            params.ground.oak_tissue_sample_ordinary_edge_4,
            lateral,
        ));
    let midrib =
        (1.0 - smoothstep(
            params.ground.oak_tissue_sample_midrib_1,
            params.ground.oak_tissue_sample_midrib_2,
            local.y.abs() / radius.max(params.ground.oak_tissue_sample_midrib_3),
        )) * smoothstep(
            params.ground.oak_tissue_sample_midrib_4,
            params.ground.oak_tissue_sample_midrib_5,
            t,
        ) * (1.0
            - smoothstep(
                params.ground.oak_tissue_sample_midrib_6,
                params.ground.oak_tissue_sample_midrib_7,
                t,
            ));
    let vein_origin = (t * params.ground.oak_tissue_sample_vein_origin_1).floor()
        / params.ground.oak_tissue_sample_vein_origin_2
        + params.ground.oak_tissue_sample_vein_origin_3;
    let vein_run = (t - vein_origin).clamp(0.0, params.ground.oak_tissue_sample_vein_run);
    let side_vein =
        (1.0 - smoothstep(
            params.ground.oak_tissue_sample_side_vein_1,
            params.ground.oak_tissue_sample_side_vein_2,
            (signed_lateral.abs() - vein_run * params.ground.oak_tissue_sample_side_vein_3).abs(),
        )) * smoothstep(
            params.ground.oak_tissue_sample_side_vein_4,
            params.ground.oak_tissue_sample_side_vein_5,
            vein_run,
        ) * (1.0
            - smoothstep(
                params.ground.oak_tissue_sample_side_vein_6,
                params.ground.oak_tissue_sample_side_vein_7,
                vein_run,
            ));
    let lifted_sector = (0.5
        + 0.5 * (longitudinal * params.ground.oak_tissue_sample_lifted_sector + phase).sin())
    .powi(5);
    let contact_band = smoothstep(
        params.ground.oak_tissue_sample_contact_band_1,
        params.ground.oak_tissue_sample_contact_band_2,
        lateral,
    ) * (1.0
        - smoothstep(
            params.ground.oak_tissue_sample_contact_band_3,
            params.ground.oak_tissue_sample_contact_band_4,
            lateral,
        ));
    LitterShapeSample {
        coverage,
        dome: ((1.0 - lateral.powi(2)).max(0.0) * 0.58 + ordinary_edge * 0.18) * coverage,
        vein: midrib.max(side_vein) * coverage,
        edge: ordinary_edge * coverage,
        lift: contact_band * lifted_sector * 0.42 * coverage,
    }
}

fn half_leaf_sample(
    params: &crate::TextureParameters,
    local: Vec2,
    radius: f32,
    aspect: f32,
    phase: f32,
    side: f32,
) -> LitterShapeSample {
    let longitudinal = local.x / radius;
    if longitudinal.abs() >= 1.0 {
        return LitterShapeSample::default();
    }
    let t = (longitudinal + 1.0) * 0.5;
    let half_width = radius * aspect * broad_oak_width(params, t, phase, local.y.signum());
    let signed_lateral =
        local.y / half_width.max(params.ground.half_leaf_sample_signed_lateral) * side;
    let outer_distance =
        (local.y / half_width.max(params.ground.half_leaf_sample_outer_distance)).abs();
    let ragged_cut = -params.ground.half_leaf_sample_ragged_cut_1
        + params.ground.half_leaf_sample_ragged_cut_2
            * (t * params.ground.half_leaf_sample_ragged_cut_3 + phase).sin()
        + params.ground.half_leaf_sample_ragged_cut_4
            * (t * params.ground.half_leaf_sample_ragged_cut_5).sin();
    let retained_side = smoothstep(
        ragged_cut - params.ground.half_leaf_sample_retained_side_1,
        ragged_cut + params.ground.half_leaf_sample_retained_side_2,
        signed_lateral,
    );
    let coverage =
        (1.0 - smoothstep(
            params.ground.half_leaf_sample_coverage_1,
            params.ground.half_leaf_sample_coverage_2,
            outer_distance,
        )) * retained_side;
    let outer_edge = smoothstep(
        params.ground.half_leaf_sample_outer_edge_1,
        params.ground.half_leaf_sample_outer_edge_2,
        outer_distance,
    ) * (1.0
        - smoothstep(
            params.ground.half_leaf_sample_outer_edge_3,
            params.ground.half_leaf_sample_outer_edge_4,
            outer_distance,
        ));
    let torn_edge =
        (1.0 - smoothstep(
            params.ground.half_leaf_sample_torn_edge_1,
            params.ground.half_leaf_sample_torn_edge_2,
            (signed_lateral - ragged_cut).abs(),
        )) * retained_side;
    let midrib =
        (1.0 - smoothstep(
            params.ground.half_leaf_sample_midrib_1,
            params.ground.half_leaf_sample_midrib_2,
            (signed_lateral - ragged_cut).abs(),
        )) * smoothstep(
            params.ground.half_leaf_sample_midrib_3,
            params.ground.half_leaf_sample_midrib_4,
            t,
        ) * (1.0
            - smoothstep(
                params.ground.half_leaf_sample_midrib_5,
                params.ground.half_leaf_sample_midrib_6,
                t,
            ));
    let material = (1.0 - outer_distance.powi(2)).max(0.0);
    let lifted_sector =
        (0.5 + 0.5 * (t * params.ground.half_leaf_sample_lifted_sector - phase).sin()).powi(5);
    let contact_band = smoothstep(
        params.ground.half_leaf_sample_contact_band_1,
        params.ground.half_leaf_sample_contact_band_2,
        outer_distance,
    ) * (1.0
        - smoothstep(
            params.ground.half_leaf_sample_contact_band_3,
            params.ground.half_leaf_sample_contact_band_4,
            outer_distance,
        ));
    LitterShapeSample {
        coverage,
        dome: (material * 0.44 + torn_edge * 0.16) * coverage,
        vein: midrib * coverage,
        edge: outer_edge.max(torn_edge) * coverage,
        lift: contact_band * lifted_sector * 0.34 * coverage,
    }
}

fn curled_fold_sample(
    params: &crate::TextureParameters,
    local: Vec2,
    radius: f32,
    aspect: f32,
    phase: f32,
    side: f32,
) -> LitterShapeSample {
    let longitudinal = local.x / radius;
    if longitudinal.abs() >= 1.0 {
        return LitterShapeSample::default();
    }
    let taper = (1.0 - longitudinal.abs())
        .max(0.0)
        .powf(params.ground.curled_fold_sample_taper);
    let centreline = side
        * radius
        * aspect
        * (params.ground.curled_fold_sample_centreline_1 * (1.0 - longitudinal.powi(2))
            + params.ground.curled_fold_sample_centreline_2
                * (longitudinal * core::f32::consts::PI + phase).sin());
    let across =
        (local.y - centreline) / (radius * aspect).max(params.ground.curled_fold_sample_across);
    let ribbon_half_width = (params.ground.curled_fold_sample_ribbon_half_width_1
        + params.ground.curled_fold_sample_ribbon_half_width_2 * taper)
        * (params.ground.curled_fold_sample_ribbon_half_width_3
            + params.ground.curled_fold_sample_ribbon_half_width_4 * phase.sin().abs());
    let ribbon_distance =
        across.abs() / ribbon_half_width.max(params.ground.curled_fold_sample_ribbon_distance);
    let flap_centre = centreline
        - side
            * radius
            * aspect
            * (params.ground.curled_fold_sample_flap_centre_1
                + params.ground.curled_fold_sample_flap_centre_2 * longitudinal);
    let flap_across = (local.y - flap_centre)
        / (radius * aspect).max(params.ground.curled_fold_sample_flap_across);
    let flap =
        (1.0 - smoothstep(
            params.ground.curled_fold_sample_flap_1,
            params.ground.curled_fold_sample_flap_2,
            flap_across.abs(),
        )) * smoothstep(
            -params.ground.curled_fold_sample_flap_3,
            -params.ground.curled_fold_sample_flap_4,
            longitudinal,
        ) * (1.0
            - smoothstep(
                params.ground.curled_fold_sample_flap_5,
                params.ground.curled_fold_sample_flap_6,
                longitudinal,
            ));
    let ribbon = 1.0
        - smoothstep(
            params.ground.curled_fold_sample_ribbon_1,
            params.ground.curled_fold_sample_ribbon_2,
            ribbon_distance,
        );
    let coverage = ribbon.max(flap * params.ground.curled_fold_sample_coverage) * taper;
    let fold =
        (1.0 - smoothstep(
            params.ground.curled_fold_sample_fold_1,
            params.ground.curled_fold_sample_fold_2,
            ribbon_distance,
        )) * coverage;
    let edge = smoothstep(
        params.ground.curled_fold_sample_edge_1,
        params.ground.curled_fold_sample_edge_2,
        ribbon_distance,
    ) * (1.0
        - smoothstep(
            params.ground.curled_fold_sample_edge_3,
            params.ground.curled_fold_sample_edge_4,
            ribbon_distance,
        ))
        * coverage;
    let concave_contact = smoothstep(
        -params.ground.curled_fold_sample_concave_contact_1,
        -params.ground.curled_fold_sample_concave_contact_2,
        across * side,
    ) * (1.0
        - smoothstep(
            params.ground.curled_fold_sample_concave_contact_3,
            params.ground.curled_fold_sample_concave_contact_4,
            across * side,
        ))
        * taper;
    LitterShapeSample {
        coverage,
        dome: (fold * 0.52 + flap * 0.20) * coverage,
        vein: fold * 0.42,
        edge,
        lift: concave_contact * coverage * 0.58,
    }
}

fn torn_fragment_sample(
    params: &crate::TextureParameters,
    local: Vec2,
    radius: f32,
    aspect: f32,
    phase: f32,
) -> LitterShapeSample {
    let normalized = Vec2::new(local.x / radius, local.y / (radius * aspect));
    let angle = normalized.y.atan2(normalized.x);
    let ragged_radius = params.ground.torn_fragment_sample_ragged_radius_1
        + params.ground.torn_fragment_sample_ragged_radius_2 * (angle * 3.0 + phase).sin()
        + params.ground.torn_fragment_sample_ragged_radius_3
            * (angle * params.ground.torn_fragment_sample_ragged_radius_4
                - phase * params.ground.torn_fragment_sample_ragged_radius_5)
                .sin();
    let diagonal_tear = normalized.x * params.ground.torn_fragment_sample_diagonal_tear_1
        + normalized.y * params.ground.torn_fragment_sample_diagonal_tear_2;
    let torn_cut = smoothstep(
        params.ground.torn_fragment_sample_torn_cut_1,
        params.ground.torn_fragment_sample_torn_cut_2,
        diagonal_tear
            + params.ground.torn_fragment_sample_torn_cut_3
                * (normalized.y * params.ground.torn_fragment_sample_torn_cut_4).sin(),
    );
    let distance = normalized.length()
        / ragged_radius.max(params.ground.torn_fragment_sample_distance_1)
        + torn_cut * params.ground.torn_fragment_sample_distance_2;
    let coverage = 1.0
        - smoothstep(
            params.ground.torn_fragment_sample_coverage_1,
            params.ground.torn_fragment_sample_coverage_2,
            distance,
        );
    let edge = smoothstep(
        params.ground.torn_fragment_sample_edge_1,
        params.ground.torn_fragment_sample_edge_2,
        distance,
    ) * (1.0
        - smoothstep(
            params.ground.torn_fragment_sample_edge_3,
            params.ground.torn_fragment_sample_edge_4,
            distance,
        ));
    let ridge =
        (1.0 - smoothstep(
            params.ground.torn_fragment_sample_ridge_1,
            params.ground.torn_fragment_sample_ridge_2,
            diagonal_tear.abs(),
        )) * coverage;
    LitterShapeSample {
        coverage,
        dome: ((1.0 - distance).max(0.0) * 0.34 + ridge * 0.18) * coverage,
        vein: ridge * 0.36,
        edge: edge * coverage,
        lift: edge * (0.5 + 0.5 * (angle * 5.0 + phase).sin()).powi(6) * coverage,
    }
}

fn skeleton_sample(
    params: &crate::TextureParameters,
    local: Vec2,
    radius: f32,
    aspect: f32,
    phase: f32,
) -> LitterShapeSample {
    let longitudinal = local.x / radius;
    if longitudinal.abs() >= 0.96 {
        return LitterShapeSample::default();
    }
    let t = (longitudinal + 1.0) * 0.5;
    let signed_lateral =
        local.y / (radius * aspect).max(params.ground.skeleton_sample_signed_lateral);
    let midrib = 1.0
        - smoothstep(
            params.ground.skeleton_sample_midrib_1,
            params.ground.skeleton_sample_midrib_2,
            local.y.abs() / radius.max(params.ground.skeleton_sample_midrib_3),
        );
    let vein_origin = (t * params.ground.skeleton_sample_vein_origin_1).floor()
        / params.ground.skeleton_sample_vein_origin_2
        + params.ground.skeleton_sample_vein_origin_3;
    let vein_run = (t - vein_origin).clamp(0.0, params.ground.skeleton_sample_vein_run);
    let ribs =
        (1.0 - smoothstep(
            params.ground.skeleton_sample_ribs_1,
            params.ground.skeleton_sample_ribs_2,
            (signed_lateral.abs() - vein_run * params.ground.skeleton_sample_ribs_3).abs(),
        )) * smoothstep(
            params.ground.skeleton_sample_ribs_4,
            params.ground.skeleton_sample_ribs_5,
            vein_run,
        ) * (1.0
            - smoothstep(
                params.ground.skeleton_sample_ribs_6,
                params.ground.skeleton_sample_ribs_7,
                vein_run,
            ));
    let residual = smoothstep(
        params.ground.skeleton_sample_residual_1,
        params.ground.skeleton_sample_residual_2,
        (t * params.ground.skeleton_sample_residual_3
            + signed_lateral * params.ground.skeleton_sample_residual_4
            + phase)
            .sin(),
    ) * (1.0
        - smoothstep(
            params.ground.skeleton_sample_residual_5,
            params.ground.skeleton_sample_residual_6,
            signed_lateral.abs(),
        ));
    let coverage = midrib
        .max(ribs)
        .max(residual * params.ground.skeleton_sample_coverage);
    LitterShapeSample {
        coverage,
        dome: coverage * 0.22,
        vein: midrib.max(ribs),
        edge: residual * 0.20,
        lift: ribs * 0.12,
    }
}

fn humified_fragment_sample(
    params: &crate::TextureParameters,
    local: Vec2,
    radius: f32,
    aspect: f32,
    phase: f32,
) -> LitterShapeSample {
    let normalized = Vec2::new(local.x / radius, local.y / (radius * aspect));
    let angle = normalized.y.atan2(normalized.x);
    let edge_noise = params.ground.humified_fragment_sample_edge_noise_1
        + params.ground.humified_fragment_sample_edge_noise_2
            * (angle * params.ground.humified_fragment_sample_edge_noise_3 + phase).sin()
        + params.ground.humified_fragment_sample_edge_noise_4
            * (angle * params.ground.humified_fragment_sample_edge_noise_5 - phase).sin();
    let distance =
        normalized.length() / edge_noise.max(params.ground.humified_fragment_sample_distance);
    let coverage = 1.0
        - smoothstep(
            params.ground.humified_fragment_sample_coverage_1,
            params.ground.humified_fragment_sample_coverage_2,
            distance,
        );
    LitterShapeSample {
        coverage,
        dome: (1.0 - distance).max(0.0) * coverage * 0.12,
        edge: smoothstep(0.64, 0.88, distance)
            * (1.0 - smoothstep(0.94, 1.04, distance))
            * coverage
            * 0.20,
        ..LitterShapeSample::default()
    }
}

fn sample_litter_shape(
    params: &crate::TextureParameters,
    class: LitterShapeClass,
    local: Vec2,
    radius: f32,
    aspect: f32,
    phase: f32,
    side: f32,
) -> LitterShapeSample {
    match class {
        LitterShapeClass::IntactOak => oak_tissue_sample(params, local, radius, aspect, phase),
        LitterShapeClass::CurledOak => {
            curled_fold_sample(params, local, radius, aspect, phase, side)
        }
        LitterShapeClass::HalfLeaf => half_leaf_sample(params, local, radius, aspect, phase, side),
        LitterShapeClass::TornFragment => {
            torn_fragment_sample(params, local, radius, aspect, phase)
        }
        LitterShapeClass::Skeleton => skeleton_sample(params, local, radius, aspect, phase),
        LitterShapeClass::Humified => {
            humified_fragment_sample(params, local, radius, aspect, phase)
        }
    }
}

fn litter_shape_visible(
    class: LitterShapeClass,
    stratum: LitterStratum,
    detail: LitterDetail,
) -> bool {
    match detail {
        LitterDetail::Full => true,
        LitterDetail::Mid => {
            stratum != LitterStratum::Lower
                && matches!(
                    class,
                    LitterShapeClass::IntactOak
                        | LitterShapeClass::HalfLeaf
                        | LitterShapeClass::TornFragment
                )
        }
        LitterDetail::Far => false,
    }
}

fn litter_leaf_field_with_detail(
    params: &crate::TextureParameters,
    point: Vec2,
    recipe: LitterStratumRecipe,
    detail: LitterDetail,
) -> LitterLeafImprint {
    let scaled = point * recipe.grid as f32;
    let base_cell = scaled.floor().as_ivec2();
    let mut field = LitterLeafImprint {
        order: f32::NEG_INFINITY,
        ..LitterLeafImprint::default()
    };
    for offset_y in -1..=1 {
        for offset_x in -1..=1 {
            let cell = base_cell + IVec2::new(offset_x, offset_y);
            let occupancy = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x5de3);
            let decay_pocket = if recipe.stratum == LitterStratum::Lower {
                smoothstep(
                    -params.ground.litter_leaf_field_with_detail_decay_pocket_1,
                    params.ground.litter_leaf_field_with_detail_decay_pocket_2,
                    soil_value_noise(
                        params,
                        point
                            + Vec2::new(
                                params.ground.litter_leaf_field_with_detail_decay_pocket_3,
                                params.ground.litter_leaf_field_with_detail_decay_pocket_4,
                            ),
                        6,
                        recipe.salt ^ 0x77a1,
                    ),
                )
            } else {
                1.0
            };
            let effective_density = recipe.density
                * (params
                    .ground
                    .litter_leaf_field_with_detail_effective_density_1
                    + decay_pocket
                        * params
                            .ground
                            .litter_leaf_field_with_detail_effective_density_2);
            if occupancy > effective_density {
                continue;
            }
            let centre = cell.as_vec2()
                + Vec2::new(
                    params.ground.litter_leaf_field_with_detail_centre_1
                        + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x13a7)
                            * params.ground.litter_leaf_field_with_detail_centre_2,
                    params.ground.litter_leaf_field_with_detail_centre_3
                        + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x91cb)
                            * params.ground.litter_leaf_field_with_detail_centre_4,
                );
            let angle = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xc72d)
                * core::f32::consts::TAU;
            let long_axis = Vec2::new(angle.cos(), angle.sin());
            let delta = scaled - centre;
            let local = Vec2::new(delta.dot(long_axis), delta.perp_dot(long_axis));
            let radius = recipe.minimum_radius
                + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x27f1)
                    * recipe.radius_span;
            let aspect = recipe.minimum_aspect
                + soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xe419)
                    * recipe.aspect_span;
            let phase = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x41af)
                * core::f32::consts::TAU;
            let class = litter_shape_class(params, cell, recipe);
            if !litter_shape_visible(class, recipe.stratum, detail) {
                continue;
            }
            let side = (soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0x998b)
                - 0.5)
                .signum();
            let shape = sample_litter_shape(params, class, local, radius, aspect, phase, side);
            if shape.coverage <= 0.0 {
                continue;
            }
            let order = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xd34f)
                + shape.coverage * params.ground.litter_leaf_field_with_detail_order;
            if order <= field.order {
                continue;
            }
            let pigment = soil_random(params, cell.x, cell.y, recipe.grid, recipe.salt ^ 0xa531);
            let pigment_tone = (params.ground.litter_leaf_field_with_detail_pigment_tone_1
                - recipe.decomposition
                    * params.ground.litter_leaf_field_with_detail_pigment_tone_2)
                + (pigment - 0.5) * params.ground.litter_leaf_field_with_detail_pigment_tone_3;
            field = LitterLeafImprint {
                coverage: shape.coverage,
                dome: shape.dome,
                tone: (pigment_tone + shape.vein * 0.12
                    - shape.edge * 0.06
                    - if class == LitterShapeClass::Humified {
                        0.10
                    } else {
                        0.0
                    })
                .clamp(0.0, 1.0),
                vein: shape.vein,
                edge: shape.edge,
                contact: shape.lift,
                order,
            };
        }
    }
    field
}

#[cfg(test)]
fn litter_leaf_field(point: Vec2, recipe: LitterStratumRecipe) -> LitterLeafImprint {
    let params = &crate::TextureParameters::default();

    litter_leaf_field_with_detail(params, point, recipe, LitterDetail::Full)
}

fn humified_debris_patch(params: &crate::TextureParameters, point: Vec2) -> LitterLeafImprint {
    let pocket = smoothstep(
        -params.ground.humified_debris_patch_pocket_1,
        params.ground.humified_debris_patch_pocket_2,
        soil_value_noise(params, point, 5, 0x8c31),
    );
    let breakup = smoothstep(
        -params.ground.humified_debris_patch_breakup_1,
        params.ground.humified_debris_patch_breakup_2,
        soil_value_noise(params, point, 47, 0xe729),
    );
    let coverage = pocket
        * (params.ground.humified_debris_patch_coverage_1
            + breakup * params.ground.humified_debris_patch_coverage_2);
    LitterLeafImprint {
        coverage,
        dome: coverage * (0.025 + breakup * 0.040),
        tone: (0.17 + soil_value_noise(params, point, 19, 0x4a61) * 0.055).clamp(0.08, 0.25),
        contact: pocket * (0.38 + (1.0 - breakup) * 0.36),
        ..LitterLeafImprint::default()
    }
}

fn forest_litter_far_sample(params: &crate::TextureParameters, point: Vec2) -> ForestLitterSample {
    let broad_humus = smoothstep(
        -params.ground.forest_litter_far_sample_broad_humus_1,
        params.ground.forest_litter_far_sample_broad_humus_2,
        soil_value_noise(params, point, 4, 0x8c31),
    );
    let merged_litter = smoothstep(
        -params.ground.forest_litter_far_sample_merged_litter_1,
        params.ground.forest_litter_far_sample_merged_litter_2,
        soil_value_noise(
            params,
            point
                + Vec2::new(
                    params.ground.forest_litter_far_sample_merged_litter_3,
                    params.ground.forest_litter_far_sample_merged_litter_4,
                ),
            7,
            0x3d91,
        ) + soil_value_noise(params, point, 13, 0xa741)
            * params.ground.forest_litter_far_sample_merged_litter_5,
    );
    let coverage = (broad_humus * params.ground.forest_litter_far_sample_coverage_1
        + merged_litter * params.ground.forest_litter_far_sample_coverage_2)
        .clamp(0.0, 1.0);
    let contact = smoothstep(
        params.ground.forest_litter_far_sample_contact_1,
        params.ground.forest_litter_far_sample_contact_2,
        merged_litter,
    ) * (1.0
        - smoothstep(
            params.ground.forest_litter_far_sample_contact_3,
            params.ground.forest_litter_far_sample_contact_4,
            merged_litter,
        ));
    let broad_relief = soil_value_noise(
        params,
        point
            + Vec2::new(
                params.ground.forest_litter_far_sample_broad_relief_1,
                params.ground.forest_litter_far_sample_broad_relief_2,
            ),
        5,
        0x91b3,
    ) * params.ground.forest_litter_far_sample_broad_relief_3;
    ForestLitterSample {
        height: (0.49 + broad_humus * 0.045 + merged_litter * 0.13 + broad_relief)
            .clamp(0.48, 0.72),
        ao: (0.96 - broad_humus * 0.07 - merged_litter * 0.06 - contact * 0.08).clamp(0.74, 0.98),
        tone: (0.12 + broad_humus * 0.10 + merged_litter * 0.27).clamp(0.10, 0.49),
        coverage,
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ForestLitterSample {
    pub(super) height: f32,
    pub(super) ao: f32,
    pub(super) tone: f32,
    pub(super) coverage: f32,
}

fn forest_litter_sample_with_detail(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
    detail: LitterDetail,
) -> ForestLitterSample {
    let point = Vec2::new(u, v);
    let warp = Vec2::new(
        soil_value_noise(params, point, 7, 0x51a7),
        soil_value_noise(
            params,
            point
                + Vec2::new(
                    params.ground.forest_litter_sample_with_detail_warp_1,
                    params.ground.forest_litter_sample_with_detail_warp_2,
                ),
            7,
            0x8d31,
        ),
    ) * params.ground.forest_litter_sample_with_detail_warp_3;
    let sample = point + warp;
    if detail == LitterDetail::Far {
        return forest_litter_far_sample(params, sample);
    }
    let humus = humified_debris_patch(params, sample);
    let [lower, middle, upper] = params
        .ground
        .litter_strata
        .map(|recipe| litter_leaf_field_with_detail(params, sample, recipe, detail));
    let coverage = 1.0
        - (1.0 - humus.coverage)
            * (1.0 - lower.coverage)
            * (1.0 - middle.coverage)
            * (1.0 - upper.coverage);
    let humus_height = params
        .ground
        .forest_litter_sample_with_detail_humus_height_1
        + humus.dome
            * params
                .ground
                .forest_litter_sample_with_detail_humus_height_2;
    let lower_height = params
        .ground
        .forest_litter_sample_with_detail_lower_height_1
        + lower.dome
            * params
                .ground
                .forest_litter_sample_with_detail_lower_height_2
        + lower.edge
            * params
                .ground
                .forest_litter_sample_with_detail_lower_height_3;
    let middle_height = params
        .ground
        .forest_litter_sample_with_detail_middle_height_1
        + middle.dome
            * params
                .ground
                .forest_litter_sample_with_detail_middle_height_2
        + middle.edge
            * params
                .ground
                .forest_litter_sample_with_detail_middle_height_3;
    let upper_height = params
        .ground
        .forest_litter_sample_with_detail_upper_height_1
        + upper.dome
            * params
                .ground
                .forest_litter_sample_with_detail_upper_height_2
        + upper.edge
            * params
                .ground
                .forest_litter_sample_with_detail_upper_height_3;
    let height = humus_height
        .lerp(lower_height, lower.coverage)
        .lerp(middle_height, middle.coverage)
        .lerp(upper_height, upper.coverage)
        .clamp(
            params.ground.forest_litter_sample_with_detail_height_1,
            params.ground.forest_litter_sample_with_detail_height_2,
        );
    let vein = lower.vein * (1.0 - middle.coverage) * (1.0 - upper.coverage)
        + middle.vein * (1.0 - upper.coverage)
        + upper.vein;
    let contact = lower.contact * (1.0 - middle.coverage) * (1.0 - upper.coverage)
        + middle.contact * (1.0 - upper.coverage)
        + upper.contact;
    let upper_support = middle
        .coverage
        .max(lower.coverage)
        .max(humus.coverage * params.ground.forest_litter_sample_with_detail_upper_support);
    let middle_support = lower.coverage.max(
        humus.coverage
            * params
                .ground
                .forest_litter_sample_with_detail_middle_support,
    );
    let layer_contact = upper.coverage
        * upper_support
        * (params
            .ground
            .forest_litter_sample_with_detail_layer_contact_1
            + upper.dome
                * params
                    .ground
                    .forest_litter_sample_with_detail_layer_contact_2)
        + middle.coverage
            * middle_support
            * (params
                .ground
                .forest_litter_sample_with_detail_layer_contact_3
                + middle.dome
                    * params
                        .ground
                        .forest_litter_sample_with_detail_layer_contact_4)
        + lower.coverage
            * humus.coverage
            * (params
                .ground
                .forest_litter_sample_with_detail_layer_contact_5
                + lower.dome
                    * params
                        .ground
                        .forest_litter_sample_with_detail_layer_contact_6);
    let ao = (1.0
        - humus.contact * params.ground.forest_litter_sample_with_detail_ao_1
        - contact * params.ground.forest_litter_sample_with_detail_ao_2
        - layer_contact * params.ground.forest_litter_sample_with_detail_ao_3
        - vein * params.ground.forest_litter_sample_with_detail_ao_4)
        .clamp(params.ground.forest_litter_sample_with_detail_ao_5, 1.0);
    let visible_tone = humus.tone
        * humus.coverage
        * (1.0 - lower.coverage)
        * (1.0 - middle.coverage)
        * (1.0 - upper.coverage)
        + lower.tone * lower.coverage * (1.0 - middle.coverage) * (1.0 - upper.coverage)
        + middle.tone * middle.coverage * (1.0 - upper.coverage)
        + upper.tone * upper.coverage;
    let exposed_soil_tone = (soil_value_noise(params, sample, 11, 0xb875)
        * params
            .ground
            .forest_litter_sample_with_detail_exposed_soil_tone_1
        + params
            .ground
            .forest_litter_sample_with_detail_exposed_soil_tone_2)
        .clamp(0.0, 1.0);
    let tone = (visible_tone + exposed_soil_tone * (1.0 - coverage)).clamp(0.0, 1.0);
    ForestLitterSample {
        height,
        ao,
        tone,
        coverage,
    }
}

pub(super) fn forest_litter_sample(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
) -> ForestLitterSample {
    forest_litter_sample_with_detail(params, u, v, LitterDetail::Full)
}

#[cfg(all(test, not(target_family = "wasm")))]
mod litter_tests {
    use std::{fs, path::Path};

    use ::image::{ColorType, ImageFormat, save_buffer_with_format};

    use super::*;

    fn base_mip(image: &Image, bytes_per_pixel: usize, level: u32) -> &[u8] {
        let size = FOREST_SOIL_TEXTURE_SIZE >> level;
        let offset = (0..level)
            .map(|prior| (FOREST_SOIL_TEXTURE_SIZE >> prior).pow(2) as usize * bytes_per_pixel)
            .sum::<usize>();
        &image.data.as_deref().unwrap()[offset..offset + size.pow(2) as usize * bytes_per_pixel]
    }

    fn save_png(path: &Path, pixels: &[u8], size: u32, color: ColorType) {
        save_buffer_with_format(path, pixels, size, size, color, ImageFormat::Png).unwrap();
    }

    fn channel(surface: &[u8], channel: usize) -> Vec<u8> {
        surface
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[channel])
            .collect()
    }

    fn mean_adjacent_delta(mip: &[u8], size: usize, channel: usize) -> f32 {
        let mut total = 0_u64;
        let mut pairs = 0_u64;
        for y in 0..size {
            for x in 0..size {
                let current = mip[(y * size + x) * 4 + channel];
                let right = mip[(y * size + (x + 1) % size) * 4 + channel];
                let down = mip[(((y + 1) % size) * size + x) * 4 + channel];
                total += u64::from(current.abs_diff(right)) + u64::from(current.abs_diff(down));
                pairs += 2;
            }
        }
        total as f32 / pairs as f32
    }

    fn appearance(surface: &[u8]) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(surface.len() / 4 * 3);
        for pixel in surface.as_chunks::<4>().0 {
            let tone = pixel[2] as f32 / 255.0;
            let coverage = pixel[3] as f32 / 255.0;
            let tone_mix = smoothstep(0.18, 0.72, tone);
            let leaf = [
                72.0_f32.lerp(154.0, tone_mix),
                48.0_f32.lerp(111.0, tone_mix),
                30.0_f32.lerp(54.0, tone_mix),
            ];
            let soil = [55.0, 43.0, 32.0];
            let visibility = pixel[1] as f32 / 255.0;
            for color in 0..3 {
                let value = soil[color] * (1.0 - coverage) + leaf[color] * coverage;
                rgb.push((value * visibility).round().clamp(0.0, 255.0) as u8);
            }
        }
        rgb
    }

    fn normal_rgb(normal: &[u8]) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(normal.len() / 2 * 3);
        for pixel in normal.as_chunks::<2>().0 {
            let x = pixel[0] as f32 / 127.5 - 1.0;
            let z = pixel[1] as f32 / 127.5 - 1.0;
            let y = (1.0 - x * x - z * z).max(0.0).sqrt();
            rgb.extend_from_slice(&[pixel[0], ((y * 0.5 + 0.5) * 255.0).round() as u8, pixel[1]]);
        }
        rgb
    }

    #[test]
    fn strata_encode_plausible_scale_decomposition_and_structural_detail() {
        let physical_lengths = LITTER_STRATA.map(|recipe| {
            (
                2.0 * recipe.minimum_radius * FOREST_LITTER_TILE_METRES / recipe.grid as f32,
                2.0 * (recipe.minimum_radius + recipe.radius_span) * FOREST_LITTER_TILE_METRES
                    / recipe.grid as f32,
            )
        });
        assert!((0.05..=0.10).contains(&physical_lengths[0].0));
        assert!((0.10..=0.22).contains(&physical_lengths[1].1));
        assert!((0.15..=0.23).contains(&physical_lengths[2].0));
        assert!(LITTER_STRATA[2].density < LITTER_STRATA[1].density);
        assert!(LITTER_STRATA[0].decomposition > LITTER_STRATA[1].decomposition);
        assert!(LITTER_STRATA[1].decomposition > LITTER_STRATA[2].decomposition);

        let mut covered = [0_usize; 3];
        let mut tone = [0.0_f32; 3];
        let mut edges = [0_usize; 3];
        let mut veins = [0_usize; 3];
        for y in 0..192 {
            for x in 0..192 {
                let point = Vec2::new((x as f32 + 0.5) / 192.0, (y as f32 + 0.5) / 192.0);
                for (index, recipe) in LITTER_STRATA.into_iter().enumerate() {
                    let leaf = litter_leaf_field(point, recipe);
                    if leaf.coverage > 0.5 {
                        covered[index] += 1;
                        tone[index] += leaf.tone;
                        edges[index] += usize::from(leaf.edge > 0.20);
                        veins[index] += usize::from(leaf.vein > 0.24);
                    }
                }
            }
        }
        let mean_tone =
            core::array::from_fn::<_, 3, _>(|index| tone[index] / covered[index] as f32);
        assert!(mean_tone[0] + 0.12 < mean_tone[2]);
        assert!(covered.into_iter().all(|count| count > 2_000));
        assert!(edges.into_iter().all(|count| count > 180));
        assert!(veins.into_iter().all(|count| count > 80));
    }

    #[test]
    fn litter_shape_classes_have_distinct_deterministic_masks() {
        let params = &crate::TextureParameters::default();
        let classes = [
            LitterShapeClass::IntactOak,
            LitterShapeClass::CurledOak,
            LitterShapeClass::HalfLeaf,
            LitterShapeClass::TornFragment,
            LitterShapeClass::Skeleton,
            LitterShapeClass::Humified,
        ];
        let areas = classes.map(|class| {
            let mut area = 0_usize;
            for y in 0..96 {
                for x in 0..96 {
                    let local = Vec2::new(
                        (x as f32 + 0.5) / 96.0 * 2.0 - 1.0,
                        (y as f32 + 0.5) / 96.0 * 1.4 - 0.7,
                    );
                    area += usize::from(
                        sample_litter_shape(params, class, local, 0.92, 0.62, 0.73, 1.0).coverage
                            > 0.5,
                    );
                }
            }
            area
        });
        assert!(areas[0] > areas[1]);
        assert!(areas[0] > areas[2] * 3 / 2);
        assert!(areas[3] < areas[0] * 3 / 4);
        assert!(areas[4] < areas[0] / 3);
        assert!(areas[5] < areas[3]);
        for first in 0..areas.len() {
            for second in first + 1..areas.len() {
                assert_ne!(areas[first], areas[second]);
            }
        }
    }

    #[test]
    fn shape_selection_is_deterministic_and_stratum_specific() {
        let params = &crate::TextureParameters::default();
        let mut counts = [[0_usize; 6]; 3];
        for (stratum, recipe) in LITTER_STRATA.into_iter().enumerate() {
            for y in 0..64 {
                for x in 0..64 {
                    let cell = IVec2::new(x, y);
                    let class = litter_shape_class(params, cell, recipe);
                    assert_eq!(class, litter_shape_class(params, cell, recipe));
                    let index = match class {
                        LitterShapeClass::IntactOak => 0,
                        LitterShapeClass::CurledOak => 1,
                        LitterShapeClass::HalfLeaf => 2,
                        LitterShapeClass::TornFragment => 3,
                        LitterShapeClass::Skeleton => 4,
                        LitterShapeClass::Humified => 5,
                    };
                    counts[stratum][index] += 1;
                }
            }
        }
        assert_eq!(counts[0][0] + counts[0][1], 0);
        assert!(counts[0][3] > counts[0][2]);
        assert!(counts[0][5] > 1_500);
        assert!(counts[1][0..5].iter().all(|count| *count > 300));
        assert_eq!(counts[1][5], 0);
        assert!(counts[2][0] > counts[2][1]);
        assert_eq!(counts[2][4] + counts[2][5], 0);
    }

    #[test]
    fn litter_mips_preserve_bounded_channel_contrast_at_gameplay_distance() {
        let params = &crate::TextureParameters::default();
        let (surface, _) = generate_forest_litter_textures(params);
        let mut ao = base_mip(&surface, 4, 0)
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[1])
            .collect::<Vec<_>>();
        ao.sort_unstable();
        assert!(ao[ao.len() / 2] <= 246, "median AO: {}", ao[ao.len() / 2]);
        assert!(ao[ao.len() / 20] <= 224, "AO p5: {}", ao[ao.len() / 20]);
        for level in [3, 4] {
            let mip = base_mip(&surface, 4, level);
            for channel_index in 0..4 {
                let values = mip
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|pixel| pixel[channel_index])
                    .collect::<Vec<_>>();
                let minimum = *values.iter().min().unwrap();
                let maximum = *values.iter().max().unwrap();
                assert!(
                    maximum - minimum >= 8,
                    "level {level} channel {channel_index}"
                );
            }
        }
        let middle_distance = base_mip(&surface, 4, 3);
        let distant = base_mip(&surface, 4, 4);
        assert!(
            mean_adjacent_delta(distant, 64, 2)
                < mean_adjacent_delta(middle_distance, 128, 2) * 0.72
        );
        assert!(
            mean_adjacent_delta(distant, 64, 3)
                < mean_adjacent_delta(middle_distance, 128, 3) * 0.72
        );
        let mut patch_tones = Vec::new();
        let mut patch_coverages = Vec::new();
        for patch_y in 0..8 {
            for patch_x in 0..8 {
                let mut tone = 0_u32;
                let mut coverage = 0_u32;
                for y in patch_y * 8..patch_y * 8 + 8 {
                    for x in patch_x * 8..patch_x * 8 + 8 {
                        let pixel = (y * 64 + x) * 4;
                        tone += u32::from(distant[pixel + 2]);
                        coverage += u32::from(distant[pixel + 3]);
                    }
                }
                patch_tones.push(tone / 64);
                patch_coverages.push(coverage / 64);
            }
        }
        assert!(patch_tones.iter().max().unwrap() - patch_tones.iter().min().unwrap() >= 10);
        assert!(patch_coverages.iter().max().unwrap() - patch_coverages.iter().min().unwrap() >= 8);
    }

    #[test]
    #[ignore = "writes deterministic visual-review evidence under target"]
    fn export_forest_litter_visual_review() {
        let params = &crate::TextureParameters::default();
        let output_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        let output = output_root.join("procedural-texture-reviews/forest-litter/candidate-3");
        fs::create_dir_all(&output).unwrap();
        let (surface, normal) = generate_forest_litter_textures(params);
        let surface_base = base_mip(&surface, 4, 0);
        let normal_base = base_mip(&normal, 2, 0);
        for (name, index) in [("height", 0), ("ao", 1), ("tone", 2), ("coverage", 3)] {
            save_png(
                &output.join(format!("forest-litter-{name}.png")),
                &channel(surface_base, index),
                FOREST_SOIL_TEXTURE_SIZE,
                ColorType::L8,
            );
        }
        save_png(
            &output.join("forest-litter-normal-rgb.png"),
            &normal_rgb(normal_base),
            FOREST_SOIL_TEXTURE_SIZE,
            ColorType::Rgb8,
        );
        let interpreted = appearance(surface_base);
        save_png(
            &output.join("forest-litter-interpreted.png"),
            &interpreted,
            FOREST_SOIL_TEXTURE_SIZE,
            ColorType::Rgb8,
        );
        let mut tiled = vec![0_u8; interpreted.len() * 4];
        let row_bytes = FOREST_SOIL_TEXTURE_SIZE as usize * 3;
        for y in 0..FOREST_SOIL_TEXTURE_SIZE as usize * 2 {
            let source = (y % FOREST_SOIL_TEXTURE_SIZE as usize) * row_bytes;
            let target = y * row_bytes * 2;
            tiled[target..target + row_bytes]
                .copy_from_slice(&interpreted[source..source + row_bytes]);
            tiled[target + row_bytes..target + row_bytes * 2]
                .copy_from_slice(&interpreted[source..source + row_bytes]);
        }
        save_buffer_with_format(
            output.join("forest-litter-interpreted-2x2.png"),
            &tiled,
            FOREST_SOIL_TEXTURE_SIZE * 2,
            FOREST_SOIL_TEXTURE_SIZE * 2,
            ColorType::Rgb8,
            ImageFormat::Png,
        )
        .unwrap();
        for (level, size) in [(3, 128), (4, 64)] {
            save_png(
                &output.join(format!("forest-litter-interpreted-mip-{size}.png")),
                &appearance(base_mip(&surface, 4, level)),
                size,
                ColorType::Rgb8,
            );
        }
    }
}

fn litter_mip_samples(
    params: &crate::TextureParameters,
    size: u32,
    detail: LitterDetail,
) -> Vec<ForestLitterSample> {
    let taps = if size >= 128 { 2 } else { 4 };
    (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                let mut height = 0.0;
                let mut ao = 0.0;
                let mut tone = 0.0;
                let mut coverage = 0.0;
                for tap_y in 0..taps {
                    for tap_x in 0..taps {
                        let sample = forest_litter_sample_with_detail(
                            params,
                            (x as f32 + (tap_x as f32 + 0.5) / taps as f32) / size as f32,
                            (y as f32 + (tap_y as f32 + 0.5) / taps as f32) / size as f32,
                            detail,
                        );
                        height += sample.height;
                        ao += sample.ao;
                        tone += sample.tone;
                        coverage += sample.coverage;
                    }
                }
                let weight = 1.0 / (taps * taps) as f32;
                ForestLitterSample {
                    height: height * weight,
                    ao: ao * weight,
                    tone: tone * weight,
                    coverage: coverage * weight,
                }
            })
        })
        .collect()
}

fn encode_litter_surface(samples: &[ForestLitterSample]) -> Vec<u8> {
    let mut data = Vec::with_capacity(samples.len() * 4);
    for sample in samples {
        data.extend_from_slice(&[
            (sample.height * 255.0).round().clamp(0.0, 255.0) as u8,
            (sample.ao * 255.0).round().clamp(0.0, 255.0) as u8,
            (sample.tone * 255.0).round().clamp(0.0, 255.0) as u8,
            (sample.coverage * 255.0).round().clamp(0.0, 255.0) as u8,
        ]);
    }
    data
}

fn encode_litter_normals(
    params: &crate::TextureParameters,
    samples: &[ForestLitterSample],
    size: u32,
) -> Vec<u8> {
    let sample_at = |x: i32, y: i32| {
        samples[y.rem_euclid(size as i32) as usize * size as usize
            + x.rem_euclid(size as i32) as usize]
    };
    let texel_metres = params.common.forest_litter_tile_metres / size as f32;
    let mut data = Vec::with_capacity((size * size * 2) as usize);
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            let height_x = (sample_at(x + 1, y).height - sample_at(x - 1, y).height)
                * params.common.forest_litter_height_range_metres
                / (2.0 * texel_metres);
            let height_z = (sample_at(x, y + 1).height - sample_at(x, y - 1).height)
                * params.common.forest_litter_height_range_metres
                / (2.0 * texel_metres);
            let normal = Vec3::new(-height_x, 1.0, -height_z).normalize();
            data.extend_from_slice(&[
                ((normal.x * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8,
                ((normal.z * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8,
            ]);
        }
    }
    data
}

fn litter_mip_offset(size: u32, bytes_per_pixel: usize, level: u32) -> usize {
    (0..level)
        .map(|prior| (size >> prior).pow(2) as usize * bytes_per_pixel)
        .sum()
}

fn replace_litter_semantic_mips(
    params: &crate::TextureParameters,
    surface: &mut Image,
    normal: &mut Image,
    size: u32,
) {
    let surface_data = surface.data.as_mut().unwrap();
    let normal_data = normal.data.as_mut().unwrap();
    for level in 3..=size.ilog2() {
        let mip_size = size >> level;
        let detail = if level == 3 {
            LitterDetail::Mid
        } else {
            LitterDetail::Far
        };
        let samples = litter_mip_samples(params, mip_size, detail);
        let surface_mip = encode_litter_surface(&samples);
        let normal_mip = encode_litter_normals(params, &samples, mip_size);
        let surface_offset = litter_mip_offset(size, 4, level);
        surface_data[surface_offset..surface_offset + surface_mip.len()]
            .copy_from_slice(&surface_mip);
        let normal_offset = litter_mip_offset(size, 2, level);
        normal_data[normal_offset..normal_offset + normal_mip.len()].copy_from_slice(&normal_mip);
    }
}

pub(crate) fn generate_forest_litter_textures(params: &crate::TextureParameters) -> (Image, Image) {
    let size = params.size(FOREST_SOIL_TEXTURE_SIZE);
    let samples = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                forest_litter_sample(
                    params,
                    (x as f32 + 0.5) / size as f32,
                    (y as f32 + 0.5) / size as f32,
                )
            })
        })
        .collect::<Vec<_>>();
    let data = encode_litter_surface(&samples);
    let normal_data = encode_litter_normals(params, &samples, size);
    let mut surface = image_rgba_mipped(data, size, true);
    let mut normal = image_rg_mipped(normal_data, size, true);
    replace_litter_semantic_mips(params, &mut surface, &mut normal, size);
    (surface, normal)
}

pub(crate) fn generate_soil_height_ao(params: &crate::TextureParameters) -> Image {
    let size = params.size(FOREST_SOIL_TEXTURE_SIZE);
    let heights = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                forest_soil_height(
                    params,
                    (x as f32 + 0.5) / size as f32,
                    (y as f32 + 0.5) / size as f32,
                )
            })
        })
        .collect::<Vec<_>>();
    let horizon_ao = (0..params.size(FOREST_SOIL_AO_SIZE))
        .flat_map(|y| {
            let heights = &heights;
            (0..params.size(FOREST_SOIL_AO_SIZE))
                .map(move |x| forest_soil_horizon_ao(params, heights, x as i32, y as i32))
        })
        .collect::<Vec<_>>();
    let mut height_ao = Vec::with_capacity((size * size * 2) as usize);
    for y in 0..size {
        for x in 0..size {
            let height = periodic_sample(&heights, size, x as i32, y as i32);
            let encoded_height = ((height + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8;
            let u = (x as f32 + 0.5) / size as f32;
            let v = (y as f32 + 0.5) / size as f32;
            let broad_visibility =
                periodic_bilinear_sample(&horizon_ao, params.size(FOREST_SOIL_AO_SIZE), u, v);
            let local_visibility = forest_soil_local_cavity(params, &heights, x as i32, y as i32);
            let ao = (broad_visibility * local_visibility * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            height_ao.extend_from_slice(&[encoded_height, ao]);
        }
    }

    image_rg_mipped(height_ao, size, true)
}

pub(super) fn generate_forest_soil_texture(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> GroundTextureSet {
    let height_ao = generate_soil_height_ao(params);
    let (litter_surface, litter_normal) = generate_forest_litter_textures(params);
    GroundTextureSet {
        height_ao: images.add(height_ao),
        litter_surface: images.add(litter_surface),
        litter_normal: images.add(litter_normal),
    }
}

#[cfg(test)]
mod forest_soil_tests {
    use super::*;

    #[test]
    fn morphology_is_periodic_deterministic_and_patch_bounded() {
        let params = &crate::TextureParameters::default();

        for (u, v) in [(0.03, 0.17), (0.31, 0.79), (0.72, 0.44), (0.96, 0.08)] {
            let height = forest_soil_sample(params, u, v);
            let repeated = forest_soil_sample(params, u, v);
            let (_, _, conditions) = forest_soil_context(params, u, v);
            assert_eq!(height.to_bits(), repeated.to_bits());
            assert!((height - forest_soil_sample(params, u + 1.0, v)).abs() < 1.0e-5);
            assert!((height - forest_soil_sample(params, u, v + 1.0)).abs() < 1.0e-5);
            assert!((0.0..=1.0).contains(&conditions.compaction));
            assert!((0.0..=1.0).contains(&conditions.moisture));
        }
    }

    #[test]
    fn aggregate_retains_fine_mid_and_broad_scale_relief() {
        let params = &crate::TextureParameters::default();

        let mean_delta = |offset: f32| {
            let mut sum = 0.0;
            let mut count = 0;
            for y in 0..96 {
                for x in 0..96 {
                    let u = (x as f32 + 0.5) / 96.0;
                    let v = (y as f32 + 0.5) / 96.0;
                    sum += (forest_soil_height(params, u, v)
                        - forest_soil_height(params, u + offset, v))
                    .abs();
                    count += 1;
                }
            }
            sum / count as f32
        };
        let fine = mean_delta(1.0 / FOREST_SOIL_TEXTURE_SIZE as f32);
        let mid = mean_delta(8.0 / FOREST_SOIL_TEXTURE_SIZE as f32);
        let broad = mean_delta(32.0 / FOREST_SOIL_TEXTURE_SIZE as f32);
        assert!(fine > 0.001, "fine aggregate delta: {fine}");
        assert!(mid > fine * 1.8, "fine {fine}, mid {mid}");
        assert!(broad > mid * 1.35, "mid {mid}, broad {broad}");
    }

    #[test]
    fn wet_compaction_suppresses_detached_crumbs_and_grain() {
        let params = &crate::TextureParameters::default();

        let dry_loose = ForestSoilConditions {
            compaction: 0.0,
            moisture: 0.0,
        };
        let damp_compact = ForestSoilConditions {
            compaction: 0.75,
            moisture: 0.75,
        };
        let wet_compact = ForestSoilConditions {
            compaction: 1.0,
            moisture: 1.0,
        };
        let mut dry_energy = 0.0;
        let mut damp_energy = 0.0;
        for index in 0..256 {
            let point = Vec2::new(
                (index as f32 + 0.5) / 256.0,
                ((index * 73 % 256) as f32 + 0.5) / 256.0,
            );
            let aggregate_union = 0.18 + (index % 7) as f32 * 0.07;
            let dry = forest_soil_detached_relief(params, point, dry_loose, aggregate_union);
            let damp = forest_soil_detached_relief(params, point, damp_compact, aggregate_union);
            let wet = forest_soil_detached_relief(params, point, wet_compact, aggregate_union);
            dry_energy += dry.0 + dry.1.abs();
            damp_energy += damp.0 + damp.1.abs();
            assert_eq!(wet, (0.0, 0.0));
        }
        assert!(
            dry_energy > 0.10,
            "dry detached-relief energy: {dry_energy}"
        );
        assert!(
            dry_energy > damp_energy * 8.0,
            "dry energy {dry_energy}, damp compact energy {damp_energy}"
        );
    }

    #[test]
    fn packed_height_ao_mips_preserve_midscale_signal_without_aliasing() {
        let params = &crate::TextureParameters::default();

        let mut images = Assets::<Image>::default();
        let textures = generate_forest_soil_texture(params, &mut images);
        let image = images.get(&textures.height_ao).unwrap();
        let data = image.data.as_ref().unwrap();
        let mut offset = 0_usize;
        let mut previous_range = u8::MAX;
        for level in 0..=6 {
            let side = (FOREST_SOIL_TEXTURE_SIZE >> level) as usize;
            let level_bytes = side * side * 2;
            let height_values = data[offset..offset + level_bytes]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pixel| pixel[0]);
            let minimum = height_values.clone().min().unwrap();
            let maximum = height_values.max().unwrap();
            let range = maximum - minimum;
            assert!(
                range <= previous_range,
                "mip {level} range grew: {range} > {previous_range}"
            );
            if level <= 4 {
                assert!(range >= 18, "mip {level} lost aggregate signal: {range}");
            }
            for ao in data[offset + 1..offset + level_bytes].iter().step_by(2) {
                assert!(*ao >= 140, "mip {level} AO underflow: {ao}");
            }
            previous_range = range;
            offset += level_bytes;
        }
    }
}

mod controls;
pub use controls::Parameters;

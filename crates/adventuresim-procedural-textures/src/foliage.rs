use super::*;

const WHITE_OAK_EDGE_SAMPLES: u32 = 4;

#[derive(Clone, Copy)]
enum WhiteOakSide {
    Left,
    Right,
}

impl WhiteOakSide {
    const fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct WhiteOakLobe {
    center: f32,
    proximal_radius: f32,
    distal_radius: f32,
    reach: f32,
    vein_origin: f32,
    vein_rise: f32,
    vein_reach: f32,
}

const WHITE_OAK_LEFT_LOBES: [WhiteOakLobe; 5] = [
    WhiteOakLobe {
        center: 0.075,
        proximal_radius: 0.058,
        distal_radius: 0.074,
        reach: 0.046,
        vein_origin: 0.030,
        vein_rise: 0.052,
        vein_reach: 0.060,
    },
    WhiteOakLobe {
        center: 0.235,
        proximal_radius: 0.098,
        distal_radius: 0.118,
        reach: 0.076,
        vein_origin: 0.139,
        vein_rise: 0.103,
        vein_reach: 0.137,
    },
    WhiteOakLobe {
        center: 0.415,
        proximal_radius: 0.110,
        distal_radius: 0.128,
        reach: 0.112,
        vein_origin: 0.305,
        vein_rise: 0.119,
        vein_reach: 0.178,
    },
    WhiteOakLobe {
        center: 0.615,
        proximal_radius: 0.106,
        distal_radius: 0.116,
        reach: 0.094,
        vein_origin: 0.512,
        vein_rise: 0.111,
        vein_reach: 0.161,
    },
    WhiteOakLobe {
        center: 0.790,
        proximal_radius: 0.090,
        distal_radius: 0.084,
        reach: 0.055,
        vein_origin: 0.704,
        vein_rise: 0.091,
        vein_reach: 0.118,
    },
];

const WHITE_OAK_RIGHT_LOBES: [WhiteOakLobe; 5] = [
    WhiteOakLobe {
        center: 0.094,
        proximal_radius: 0.066,
        distal_radius: 0.054,
        reach: 0.038,
        vein_origin: 0.050,
        vein_rise: 0.050,
        vein_reach: 0.054,
    },
    WhiteOakLobe {
        center: 0.292,
        proximal_radius: 0.112,
        distal_radius: 0.096,
        reach: 0.088,
        vein_origin: 0.178,
        vein_rise: 0.119,
        vein_reach: 0.151,
    },
    WhiteOakLobe {
        center: 0.482,
        proximal_radius: 0.118,
        distal_radius: 0.108,
        reach: 0.100,
        vein_origin: 0.367,
        vein_rise: 0.121,
        vein_reach: 0.169,
    },
    WhiteOakLobe {
        center: 0.675,
        proximal_radius: 0.106,
        distal_radius: 0.094,
        reach: 0.078,
        vein_origin: 0.575,
        vein_rise: 0.105,
        vein_reach: 0.143,
    },
    WhiteOakLobe {
        center: 0.835,
        proximal_radius: 0.078,
        distal_radius: 0.068,
        reach: 0.025,
        vein_origin: 0.758,
        vein_rise: 0.080,
        vein_reach: 0.088,
    },
];

#[derive(Clone, Copy)]
struct WhiteOakSample {
    inside: bool,
    vein: bool,
    petiole: bool,
    height: f32,
    tissue_mottle: f32,
}

#[derive(Clone, Copy)]
pub(super) enum LeafMipSemantic {
    Coverage,
    ColorCoverage,
    Normal,
    Scalar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WhiteOakState {
    Living,
    Dry,
}

fn white_oak_lobes(params: &crate::TextureParameters, side: WhiteOakSide) -> &[WhiteOakLobe; 5] {
    match side {
        WhiteOakSide::Left => &params.foliage.white_oak_left_lobes,
        WhiteOakSide::Right => &params.foliage.white_oak_right_lobes,
    }
}

fn white_oak_side_width(params: &crate::TextureParameters, t: f32, side: WhiteOakSide) -> f32 {
    if !(0.0..=1.0).contains(&t) {
        return 0.0;
    }

    // A continuous lamina keeps the sinuses connected well away from the
    // midrib. Q. robur's first independently authored contribution on each
    // side forms the auriculate blade base around its very short petiole.
    let longitudinal_envelope = (t * core::f32::consts::PI)
        .sin()
        .max(0.0)
        .powf(params.foliage.white_oak_side_width_longitudinal_envelope);
    let middle_breadth = params.foliage.white_oak_side_width_middle_breadth_1
        + params.foliage.white_oak_side_width_middle_breadth_2
            * (t * core::f32::consts::PI).sin().max(0.0);
    let mut width = middle_breadth * longitudinal_envelope;
    for lobe in white_oak_lobes(params, side) {
        let radius = if t < lobe.center {
            lobe.proximal_radius
        } else {
            lobe.distal_radius
        } * params.foliage.white_oak_side_width_radius;
        let longitudinal = (t - lobe.center) / radius;
        if longitudinal.abs() < 1.0 {
            let profile = (1.0 - longitudinal * longitudinal)
                .powf(params.foliage.white_oak_side_width_profile);
            width += lobe.reach * profile;
        }
    }
    let (terminal_center, proximal_radius, distal_radius, terminal_reach) = match side {
        WhiteOakSide::Left => (0.905, 0.115, 0.095, 0.082),
        WhiteOakSide::Right => (0.892, 0.128, 0.108, 0.090),
    };
    let terminal_radius = if t < terminal_center {
        proximal_radius
    } else {
        distal_radius
    };
    let terminal = (t - terminal_center) / terminal_radius;
    if terminal.abs() < 1.0 {
        width += terminal_reach * (1.0 - terminal * terminal).powf(0.72);
    }
    width
}

#[cfg(test)]
fn white_oak_half_width(t: f32) -> f32 {
    let params = &crate::TextureParameters::default();

    white_oak_side_width(params, t, WhiteOakSide::Left).max(white_oak_side_width(
        params,
        t,
        WhiteOakSide::Right,
    ))
}

fn white_oak_tissue_mottle(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    let broad = (u * params.foliage.white_oak_tissue_mottle_broad_1
        + v * params.foliage.white_oak_tissue_mottle_broad_2
        + params.foliage.white_oak_tissue_mottle_broad_3)
        .sin();
    let cross = (u * params.foliage.white_oak_tissue_mottle_cross_1
        - v * params.foliage.white_oak_tissue_mottle_cross_2
        + params.foliage.white_oak_tissue_mottle_cross_3)
        .cos();
    let fine = (u * params.foliage.white_oak_tissue_mottle_fine_1
        + v * params.foliage.white_oak_tissue_mottle_fine_2
        + params.foliage.white_oak_tissue_mottle_fine_3)
        .sin();
    (broad * 0.52 + cross * 0.33 + fine * 0.15).clamp(-1.0, 1.0)
}

fn white_oak_vein_run(lobe: WhiteOakLobe) -> f32 {
    lobe.vein_rise + (lobe.proximal_radius + lobe.distal_radius) * 0.09
}

fn white_oak_vein_x(
    params: &crate::TextureParameters,
    side: WhiteOakSide,
    lobe: WhiteOakLobe,
    progress: f32,
) -> f32 {
    let eased = progress.powf(params.foliage.white_oak_vein_x_eased);
    let inward_curve =
        (progress * core::f32::consts::PI).sin() * params.foliage.white_oak_vein_x_inward_curve;
    side.sign() * (lobe.vein_reach * eased - inward_curve)
}

fn white_oak_mottled_color(base: [u8; 3], mottle: f32) -> [u8; 3] {
    // The field above remains continuous; eight-bit storage quantizes this
    // deliberately tiny pigment variation to adjacent solid-color regions.
    // That keeps the molded-material palette while avoiding discrete blobs.
    let darkening = ((1.0 - mottle) * 0.5).round() as u8;
    [
        base[0].saturating_sub(darkening),
        base[1].saturating_sub(darkening),
        base[2].saturating_sub(darkening),
    ]
}

fn white_oak_sample(params: &crate::TextureParameters, u: f32, v: f32) -> WhiteOakSample {
    let blade_y = 1.0 - v;
    let t = (blade_y - params.foliage.white_oak_sample_t_1) / params.foliage.white_oak_sample_t_2;
    let axis = params.foliage.white_oak_sample_axis_1
        * (t - params.foliage.white_oak_sample_axis_2).powi(2)
        - params.foliage.white_oak_sample_axis_3;
    let x = (u - 0.5) - axis;
    let petiole =
        (0.026..0.084).contains(&blade_y) && x.abs() < params.foliage.white_oak_sample_petiole;
    let side = if x < 0.0 {
        WhiteOakSide::Left
    } else {
        WhiteOakSide::Right
    };
    let side_width = white_oak_side_width(params, t, side);
    let inside_blade = (0.0..=1.0).contains(&t) && x.abs() <= side_width;
    if !inside_blade && !petiole {
        return WhiteOakSample {
            inside: false,
            vein: false,
            petiole: false,
            height: 0.0,
            tissue_mottle: 0.0,
        };
    }

    let midrib_width = params.foliage.white_oak_sample_midrib_width_1
        + params.foliage.white_oak_sample_midrib_width_2 * (1.0 - t.clamp(0.0, 1.0));
    let mut vein_distance = x.abs();
    let mut vein = x.abs() <= midrib_width;
    let mut corrugation = 0.0_f32;
    for (index, lobe) in white_oak_lobes(params, side).iter().enumerate() {
        let vein_run = white_oak_vein_run(*lobe);
        let progress = ((t - lobe.vein_origin) / vein_run).clamp(0.0, 1.0);
        if progress > 0.0 && progress < 1.0 {
            let target_x = white_oak_vein_x(params, side, *lobe, progress);
            let distance = (x - target_x).abs();
            vein_distance = vein_distance.min(distance);
            vein |= distance < 0.0034;
            let alternating_fold = if index % 2 == 0 { 1.0 } else { -1.0 };
            corrugation += alternating_fold
                * (1.0 - distance / 0.045).clamp(0.0, 1.0)
                * (progress * core::f32::consts::PI).sin()
                * 0.018;

            // A short fork supplies tertiary structure without turning the
            // albedo into a dense line drawing.
            let fork_progress = ((progress - params.foliage.white_oak_sample_fork_progress_1)
                / params.foliage.white_oak_sample_fork_progress_2)
                .clamp(0.0, 1.0);
            if fork_progress > 0.0 && fork_progress < 1.0 {
                let fork_x =
                    target_x - side.sign() * params.foliage.white_oak_sample_fork_x * fork_progress;
                let fork_distance = (x - fork_x).abs();
                vein_distance = vein_distance.min(fork_distance);
                vein |= fork_distance < 0.0018;
            }
        }
    }

    let transverse =
        (x / side_width.max(params.foliage.white_oak_sample_transverse)).clamp(-1.0, 1.0);
    let blade_dome = (1.0 - transverse * transverse)
        .powf(params.foliage.white_oak_sample_blade_dome_1)
        * params.foliage.white_oak_sample_blade_dome_2;
    let longitudinal_dome = (t.clamp(0.0, 1.0) * core::f32::consts::PI).sin().sqrt();
    let vein_ridge = (1.0 - vein_distance / params.foliage.white_oak_sample_vein_ridge_1)
        .clamp(0.0, 1.0)
        .powi(2)
        * params.foliage.white_oak_sample_vein_ridge_2;
    let tissue_mottle = white_oak_tissue_mottle(params, u, v);
    let height = if petiole && !inside_blade {
        params.foliage.white_oak_sample_height_1
    } else {
        (blade_dome * longitudinal_dome
            + corrugation
            + vein_ridge
            + tissue_mottle * params.foliage.white_oak_sample_height_2)
            .clamp(
                params.foliage.white_oak_sample_height_3,
                params.foliage.white_oak_sample_height_4,
            )
    };
    WhiteOakSample {
        inside: true,
        vein: vein || petiole,
        petiole,
        height,
        tissue_mottle,
    }
}

fn white_oak_state_sample(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
    state: WhiteOakState,
) -> WhiteOakSample {
    let mut sample = white_oak_sample(params, u, v);
    if state == WhiteOakState::Dry && sample.inside {
        sample.height = crate::dry_white_oak_leaf::relief(params, sample.height, u, v);
    }
    sample
}

fn white_oak_coverage(params: &crate::TextureParameters, u: f32, v: f32, texel: f32) -> bool {
    let mut covered = 0;
    for sample_y in 0..WHITE_OAK_EDGE_SAMPLES {
        for sample_x in 0..WHITE_OAK_EDGE_SAMPLES {
            let offset_x = (sample_x as f32 + 0.5) / WHITE_OAK_EDGE_SAMPLES as f32 - 0.5;
            let offset_y = (sample_y as f32 + 0.5) / WHITE_OAK_EDGE_SAMPLES as f32 - 0.5;
            covered += u32::from(
                white_oak_sample(params, u + offset_x * texel, v + offset_y * texel).inside,
            );
        }
    }
    covered * 2 >= WHITE_OAK_EDGE_SAMPLES.pow(2)
}

pub(super) fn leaf_mipped_image(
    params: &crate::TextureParameters,
    base_level: Vec<u8>,
    srgb: bool,
    semantic: LeafMipSemantic,
) -> Image {
    let mut mip_data = base_level.clone();
    let mut previous = base_level.clone();
    let mut previous_size = params.size(TEXTURE_SIZE);
    while previous_size > 1 {
        let next_size = previous_size / 2;
        let mut next = Vec::with_capacity((next_size * next_size * 4) as usize);
        for y in 0..next_size {
            for x in 0..next_size {
                let source_pixels = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(offset_x, offset_y)| {
                    let source_x = x * 2 + offset_x;
                    let source_y = y * 2 + offset_y;
                    let index = ((source_y * previous_size + source_x) * 4) as usize;
                    [
                        previous[index],
                        previous[index + 1],
                        previous[index + 2],
                        previous[index + 3],
                    ]
                });
                let pixel = match semantic {
                    LeafMipSemantic::Coverage => {
                        let value = source_pixels.iter().map(|pixel| pixel[0]).max().unwrap();
                        [value; 4]
                    }
                    LeafMipSemantic::ColorCoverage => {
                        let covered = source_pixels
                            .iter()
                            .copied()
                            .filter(|pixel| pixel[3] > 0)
                            .collect::<Vec<_>>();
                        if covered.is_empty() {
                            [0; 4]
                        } else {
                            let mean = covered.iter().fold(Vec3::ZERO, |sum, pixel| {
                                sum + Vec3::new(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32)
                            }) / covered.len() as f32;
                            covered
                                .into_iter()
                                .min_by_key(|pixel| {
                                    let color = Vec3::new(
                                        pixel[0] as f32,
                                        pixel[1] as f32,
                                        pixel[2] as f32,
                                    );
                                    ((color - mean).length_squared() * 16.0) as u32
                                })
                                .unwrap()
                        }
                    }
                    LeafMipSemantic::Normal => {
                        let summed = source_pixels.iter().fold(Vec3::ZERO, |sum, pixel| {
                            sum + Vec3::new(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32)
                                / 127.5
                                - Vec3::ONE
                        });
                        let normal = summed.normalize_or(Vec3::Z);
                        let encoded =
                            ((normal + Vec3::ONE) * 127.5).clamp(Vec3::ZERO, Vec3::splat(255.0));
                        [encoded.x as u8, encoded.y as u8, encoded.z as u8, 255]
                    }
                    LeafMipSemantic::Scalar => {
                        let mut pixel = [0; 4];
                        for channel in 0..4 {
                            pixel[channel] = ((source_pixels
                                .iter()
                                .map(|source| source[channel] as u32)
                                .sum::<u32>()
                                + 2)
                                / 4) as u8;
                        }
                        pixel
                    }
                };
                next.extend_from_slice(&pixel);
            }
        }
        mip_data.extend_from_slice(&next);
        previous = next;
        previous_size = next_size;
    }

    let mut image = Image::new(
        Extent3d {
            width: params.size(TEXTURE_SIZE),
            height: params.size(TEXTURE_SIZE),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        base_level,
        if srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(mip_data);
    image.texture_descriptor.mip_level_count = params.size(TEXTURE_SIZE).ilog2() + 1;
    image.sampler = ImageSampler::linear();
    image
}

fn generate_white_oak_leaf_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
    recipe: LeafRecipe,
    state: WhiteOakState,
) -> LeafTextureSet {
    let pixel_count = (params.size(TEXTURE_SIZE) * params.size(TEXTURE_SIZE)) as usize;
    let mut opacity = Vec::with_capacity(pixel_count * 4);
    let mut front = Vec::with_capacity(pixel_count * 4);
    let mut back = Vec::with_capacity(pixel_count * 4);
    let mut normal_front = Vec::with_capacity(pixel_count * 4);
    let mut normal_back = Vec::with_capacity(pixel_count * 4);
    let mut height_map = Vec::with_capacity(pixel_count * 4);
    let mut arm = Vec::with_capacity(pixel_count * 4);
    let texel = 1.0 / params.size(TEXTURE_SIZE) as f32;
    for y in 0..params.size(TEXTURE_SIZE) {
        for x in 0..params.size(TEXTURE_SIZE) {
            let u = (x as f32 + 0.5) * texel;
            let v = (y as f32 + 0.5) * texel;
            let sample = white_oak_state_sample(params, u, v, state);
            let inside = white_oak_coverage(params, u, v, texel);
            let alpha = if inside { 255 } else { 0 };
            opacity.extend_from_slice(&[alpha; 4]);
            let (front_color, back_color) = match state {
                WhiteOakState::Living => (
                    white_oak_mottled_color(recipe.blade, sample.tissue_mottle),
                    white_oak_mottled_color(recipe.back_blade, sample.tissue_mottle * 0.72),
                ),
                WhiteOakState::Dry => crate::dry_white_oak_leaf::albedo(
                    recipe.blade,
                    recipe.vein,
                    recipe.back_blade,
                    sample.vein,
                    sample.tissue_mottle,
                ),
            };
            if inside {
                front.extend_from_slice(&[front_color[0], front_color[1], front_color[2], alpha]);
                back.extend_from_slice(&[back_color[0], back_color[1], back_color[2], alpha]);
            } else {
                front.extend_from_slice(&[0; 4]);
                back.extend_from_slice(&[0; 4]);
            }
            let hx = white_oak_state_sample(params, (u + texel).min(1.0), v, state).height
                - white_oak_state_sample(params, (u - texel).max(0.0), v, state).height;
            let hy = white_oak_state_sample(params, u, (v + texel).min(1.0), state).height
                - white_oak_state_sample(params, u, (v - texel).max(0.0), state).height;
            let front_surface_normal = Vec3::new(-hx * 7.0, hy * 7.0, 1.0).normalize();
            let encoded =
                ((front_surface_normal + Vec3::ONE) * 127.5).clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal_front.extend_from_slice(&[
                encoded.x as u8,
                encoded.y as u8,
                encoded.z as u8,
                255,
            ]);
            let back_surface_normal = Vec3::new(-hx * 7.7, hy * 7.7, 1.0).normalize();
            let encoded_back =
                ((back_surface_normal + Vec3::ONE) * 127.5).clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal_back.extend_from_slice(&[
                encoded_back.x as u8,
                (255.0 - encoded_back.y) as u8,
                encoded_back.z as u8,
                255,
            ]);
            let height = if inside { sample.height } else { 0.0 };
            let ao = if !inside {
                255
            } else if sample.petiole {
                235
            } else if sample.vein {
                (params.foliage.generate_white_oak_leaf_textures_ao_1
                    + height * params.foliage.generate_white_oak_leaf_textures_ao_2)
                    .min(params.foliage.generate_white_oak_leaf_textures_ao_3) as u8
            } else {
                (params.foliage.generate_white_oak_leaf_textures_ao_4
                    + height * params.foliage.generate_white_oak_leaf_textures_ao_5
                    + sample.tissue_mottle * params.foliage.generate_white_oak_leaf_textures_ao_6)
                    .clamp(
                        params.foliage.generate_white_oak_leaf_textures_ao_7,
                        params.foliage.generate_white_oak_leaf_textures_ao_8,
                    ) as u8
            };
            let encoded_height = (height * 255.0).clamp(0.0, 255.0) as u8;
            height_map.extend_from_slice(&[encoded_height, encoded_height, encoded_height, 255]);
            arm.extend_from_slice(&[ao, recipe.roughness, 0, 255]);
        }
    }

    LeafTextureSet {
        opacity: images.add(leaf_mipped_image(
            params,
            opacity,
            false,
            LeafMipSemantic::Coverage,
        )),
        front_albedo: images.add(leaf_mipped_image(
            params,
            front,
            true,
            LeafMipSemantic::ColorCoverage,
        )),
        back_albedo: images.add(leaf_mipped_image(
            params,
            back,
            true,
            LeafMipSemantic::ColorCoverage,
        )),
        front_normal: images.add(leaf_mipped_image(
            params,
            normal_front,
            false,
            LeafMipSemantic::Normal,
        )),
        back_normal: images.add(leaf_mipped_image(
            params,
            normal_back,
            false,
            LeafMipSemantic::Normal,
        )),
        height: images.add(leaf_mipped_image(
            params,
            height_map,
            false,
            LeafMipSemantic::Scalar,
        )),
        arm: images.add(leaf_mipped_image(
            params,
            arm,
            false,
            LeafMipSemantic::Scalar,
        )),
    }
}

pub(super) fn generate_leaf_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
    species: crate::LeafSpecies,
) -> LeafTextureSet {
    use crate::LeafSpecies::*;
    let recipe = match species {
        WhiteOak => params.leaf_colors.white_oak,
        DryWhiteOak => params.leaf_colors.dry_white_oak,
        Hazel => params.leaf_colors.hazel,
        Blackthorn => params.leaf_colors.blackthorn,
        Hawthorn => params.leaf_colors.hawthorn,
        Beech => params.leaf_colors.beech,
    };
    match species {
        WhiteOak => generate_white_oak_leaf_textures(params, images, recipe, WhiteOakState::Living),
        DryWhiteOak => generate_white_oak_leaf_textures(params, images, recipe, WhiteOakState::Dry),
        Hazel => crate::hazel_leaf::generate(params, images, recipe),
        Blackthorn => crate::blackthorn_leaf::generate(params, images, recipe),
        Hawthorn => crate::hawthorn_leaf::generate(params, images, recipe),
        Beech => crate::beech_leaf::generate(params, images, recipe),
    }
}

#[cfg(test)]
mod white_oak_tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn living_white_oak_is_auriculate_and_has_independent_lobes_and_veins() {
        let params = &crate::TextureParameters::default();
        for side in [WhiteOakSide::Left, WhiteOakSide::Right] {
            let lobes = white_oak_lobes(params, side);
            let sinus_ratios = lobes
                .windows(2)
                .map(|pair| {
                    let midpoint = (pair[0].center + pair[1].center) * 0.5;
                    let sinus = white_oak_side_width(params, midpoint, side);
                    let neighboring_lobes = white_oak_side_width(params, pair[0].center, side)
                        .min(white_oak_side_width(params, pair[1].center, side));
                    sinus / neighboring_lobes
                })
                .collect::<Vec<_>>();
            let pronounced_sinuses = sinus_ratios.iter().filter(|ratio| **ratio < 0.94).count();
            assert!(
                pronounced_sinuses >= 1,
                "pronounced sinuses: {pronounced_sinuses}; ratios: {sinus_ratios:?}"
            );
            let shallowest = sinus_ratios
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            let deepest = sinus_ratios.iter().copied().fold(f32::INFINITY, f32::min);
            assert!(
                shallowest - deepest > 0.035,
                "sinus ratios: {sinus_ratios:?}"
            );
            for sample in 40..=340 {
                let t = sample as f32 / 400.0;
                assert!(white_oak_side_width(params, t, side) > 0.035);
            }

            for lobe in lobes {
                let progress = 0.78_f32;
                let vein_run = white_oak_vein_run(*lobe);
                let t = lobe.vein_origin + vein_run * progress;
                let axis = 0.026 * (t - 0.42).powi(2) - 0.004;
                let x = white_oak_vein_x(params, side, *lobe, progress);
                let u = 0.5 + axis + x;
                let v = 1.0 - (0.075 + t * 0.85);
                assert!(white_oak_sample(params, u, v).vein, "vein reaches its lobe");
            }
        }

        assert_ne!(
            WHITE_OAK_LEFT_LOBES[2].center,
            WHITE_OAK_RIGHT_LOBES[2].center
        );
        assert_ne!(
            WHITE_OAK_LEFT_LOBES[3].reach,
            WHITE_OAK_RIGHT_LOBES[3].reach
        );
        assert!(white_oak_half_width(0.075) > white_oak_half_width(0.015) * 1.8);
        assert!(white_oak_half_width(0.90) > white_oak_half_width(1.0));
        assert!(
            white_oak_sample(params, 0.5, 0.95).inside,
            "short petiole is present"
        );
    }

    #[test]
    fn living_white_oak_channels_are_detailed_palette_bounded_and_mip_complete() {
        let params = &crate::TextureParameters::default();
        let mut images = Assets::<Image>::default();
        let textures = generate_leaf_textures(params, &mut images, crate::LeafSpecies::WhiteOak);
        for handle in [
            &textures.opacity,
            &textures.front_albedo,
            &textures.back_albedo,
            &textures.front_normal,
            &textures.back_normal,
            &textures.height,
            &textures.arm,
        ] {
            let image = images.get(handle).unwrap();
            assert_eq!(image.texture_descriptor.mip_level_count, 9);
            let texels = (0..9)
                .map(|level| (TEXTURE_SIZE >> level).pow(2))
                .sum::<u32>();
            assert_eq!(image.data.as_ref().unwrap().len(), (texels * 4) as usize);
        }

        let base_bytes = (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize;
        let opacity = &images
            .get(&textures.opacity)
            .unwrap()
            .data
            .as_ref()
            .unwrap()[..base_bytes];
        assert!(
            opacity
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[0] == 0 || pixel[0] == 255)
        );
        let final_opacity = images
            .get(&textures.opacity)
            .unwrap()
            .data
            .as_ref()
            .unwrap()
            .last();
        assert_eq!(final_opacity, Some(&255), "coverage survives distant mips");

        let albedo = &images
            .get(&textures.front_albedo)
            .unwrap()
            .data
            .as_ref()
            .unwrap()[..base_bytes];
        let colors = albedo
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] > 0)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<BTreeSet<_>>();
        assert_eq!(
            colors.len(),
            2,
            "continuous mottle quantizes to adjacent colors"
        );

        let back_albedo = &images
            .get(&textures.back_albedo)
            .unwrap()
            .data
            .as_ref()
            .unwrap()[..base_bytes];
        let (front_sum, back_sum, covered) = albedo
            .as_chunks::<4>()
            .0
            .iter()
            .zip(back_albedo.as_chunks::<4>().0)
            .filter(|(front, _)| front[3] > 0)
            .fold(
                (0_u64, 0_u64, 0_u64),
                |(front_sum, back_sum, count), (front, back)| {
                    (
                        front_sum + u64::from(front[0]) + u64::from(front[1]) + u64::from(front[2]),
                        back_sum + u64::from(back[0]) + u64::from(back[1]) + u64::from(back[2]),
                        count + 1,
                    )
                },
            );
        assert!(back_sum / covered > front_sum / covered + 25);

        let height = &images.get(&textures.height).unwrap().data.as_ref().unwrap()[..base_bytes];
        let height_values = height
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[0])
            .collect::<BTreeSet<_>>();
        assert!(
            height_values.len() > 48,
            "height levels: {}",
            height_values.len()
        );
        let normal = &images
            .get(&textures.front_normal)
            .unwrap()
            .data
            .as_ref()
            .unwrap()[..base_bytes];
        let lateral_normal_values = normal
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| (pixel[0], pixel[1]))
            .collect::<BTreeSet<_>>();
        assert!(
            lateral_normal_values.len() > 200,
            "lateral normal responses: {}",
            lateral_normal_values.len()
        );

        let arm = &images.get(&textures.arm).unwrap().data.as_ref().unwrap()[..base_bytes];
        let ao_values = arm
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[0])
            .collect::<BTreeSet<_>>();
        assert!(ao_values.len() > 16, "ARM AO regions: {}", ao_values.len());
        assert_eq!(
            arm.as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| pixel[1])
                .collect::<BTreeSet<_>>()
                .len(),
            1,
            "shared ARM retains one bounded roughness baseline"
        );
    }

    #[test]
    fn living_white_oak_generation_is_repeatable() {
        let params = &crate::TextureParameters::default();
        let mut first_images = Assets::<Image>::default();
        let first = generate_leaf_textures(params, &mut first_images, crate::LeafSpecies::WhiteOak);
        let mut second_images = Assets::<Image>::default();
        let second =
            generate_leaf_textures(params, &mut second_images, crate::LeafSpecies::WhiteOak);
        for (first_handle, second_handle) in [
            (&first.opacity, &second.opacity),
            (&first.front_albedo, &second.front_albedo),
            (&first.back_albedo, &second.back_albedo),
            (&first.front_normal, &second.front_normal),
            (&first.back_normal, &second.back_normal),
            (&first.height, &second.height),
            (&first.arm, &second.arm),
        ] {
            assert_eq!(
                first_images.get(first_handle).unwrap().data,
                second_images.get(second_handle).unwrap().data
            );
        }
    }

    #[test]
    fn dry_white_oak_preserves_species_silhouette_with_distinct_material_response() {
        let params = &crate::TextureParameters::default();
        let mut images = Assets::<Image>::default();
        let living = generate_leaf_textures(params, &mut images, crate::LeafSpecies::WhiteOak);
        let dry = generate_leaf_textures(params, &mut images, crate::LeafSpecies::DryWhiteOak);
        let base_bytes = (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize;

        assert_eq!(
            &images.get(&living.opacity).unwrap().data.as_ref().unwrap()[..base_bytes],
            &images.get(&dry.opacity).unwrap().data.as_ref().unwrap()[..base_bytes],
            "drying must not replace the species-defining outline"
        );
        assert_ne!(
            &images
                .get(&living.front_albedo)
                .unwrap()
                .data
                .as_ref()
                .unwrap()[..base_bytes],
            &images
                .get(&dry.front_albedo)
                .unwrap()
                .data
                .as_ref()
                .unwrap()[..base_bytes]
        );
        assert_ne!(
            &images.get(&living.height).unwrap().data.as_ref().unwrap()[..base_bytes],
            &images.get(&dry.height).unwrap().data.as_ref().unwrap()[..base_bytes]
        );

        for handle in [
            &dry.opacity,
            &dry.front_albedo,
            &dry.back_albedo,
            &dry.front_normal,
            &dry.back_normal,
            &dry.height,
            &dry.arm,
        ] {
            let image = images.get(handle).unwrap();
            assert_eq!(image.texture_descriptor.mip_level_count, 9);
            let texels = (0..9)
                .map(|level| (TEXTURE_SIZE >> level).pow(2))
                .sum::<u32>();
            assert_eq!(image.data.as_ref().unwrap().len(), (texels * 4) as usize);
        }

        let dry_albedo = &images
            .get(&dry.front_albedo)
            .unwrap()
            .data
            .as_ref()
            .unwrap()[..base_bytes];
        let colors = dry_albedo
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] > 0)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<BTreeSet<_>>();
        assert!(
            colors.len() == 2,
            "dry molded palette regions: {}",
            colors.len()
        );
    }

    #[test]
    fn dry_white_oak_generation_is_repeatable() {
        let params = &crate::TextureParameters::default();
        let mut first_images = Assets::<Image>::default();
        let first =
            generate_leaf_textures(params, &mut first_images, crate::LeafSpecies::DryWhiteOak);
        let mut second_images = Assets::<Image>::default();
        let second =
            generate_leaf_textures(params, &mut second_images, crate::LeafSpecies::DryWhiteOak);
        for (first_handle, second_handle) in [
            (&first.opacity, &second.opacity),
            (&first.front_albedo, &second.front_albedo),
            (&first.back_albedo, &second.back_albedo),
            (&first.front_normal, &second.front_normal),
            (&first.back_normal, &second.back_normal),
            (&first.height, &second.height),
            (&first.arm, &second.arm),
        ] {
            assert_eq!(
                first_images.get(first_handle).unwrap().data,
                second_images.get(second_handle).unwrap().data
            );
        }
    }
}

mod controls;
pub use controls::Parameters;

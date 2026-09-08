use bevy::{asset::Assets, image::Image, math::Vec3};

use crate::{LeafRecipe, LeafTextureSet, TEXTURE_SIZE};

const EDGE_SAMPLES: u32 = 4;

#[derive(Clone, Copy, Debug)]
pub(super) enum HazelSide {
    Left,
    Right,
}

impl HazelSide {
    const fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct HazelVein {
    origin: f32,
    margin: f32,
    reach: f32,
}

const LEFT_VEINS: [HazelVein; 8] = [
    HazelVein {
        origin: 0.018,
        margin: 0.135,
        reach: 0.93,
    },
    HazelVein {
        origin: 0.105,
        margin: 0.240,
        reach: 0.94,
    },
    HazelVein {
        origin: 0.210,
        margin: 0.348,
        reach: 0.95,
    },
    HazelVein {
        origin: 0.320,
        margin: 0.458,
        reach: 0.95,
    },
    HazelVein {
        origin: 0.430,
        margin: 0.566,
        reach: 0.94,
    },
    HazelVein {
        origin: 0.540,
        margin: 0.664,
        reach: 0.93,
    },
    HazelVein {
        origin: 0.645,
        margin: 0.752,
        reach: 0.91,
    },
    HazelVein {
        origin: 0.735,
        margin: 0.835,
        reach: 0.88,
    },
];

const RIGHT_VEINS: [HazelVein; 8] = [
    HazelVein {
        origin: 0.026,
        margin: 0.148,
        reach: 0.92,
    },
    HazelVein {
        origin: 0.118,
        margin: 0.257,
        reach: 0.95,
    },
    HazelVein {
        origin: 0.224,
        margin: 0.366,
        reach: 0.94,
    },
    HazelVein {
        origin: 0.338,
        margin: 0.473,
        reach: 0.95,
    },
    HazelVein {
        origin: 0.446,
        margin: 0.580,
        reach: 0.93,
    },
    HazelVein {
        origin: 0.558,
        margin: 0.680,
        reach: 0.94,
    },
    HazelVein {
        origin: 0.658,
        margin: 0.768,
        reach: 0.90,
    },
    HazelVein {
        origin: 0.748,
        margin: 0.846,
        reach: 0.87,
    },
];

#[derive(Clone, Copy)]
struct HazelSample {
    inside: bool,
    vein: bool,
    petiole: bool,
    height: f32,
}

fn veins(params: &crate::TextureParameters, side: HazelSide) -> &[HazelVein; 8] {
    match side {
        HazelSide::Left => &params.hazel_leaf.left_veins,
        HazelSide::Right => &params.hazel_leaf.right_veins,
    }
}

fn axis(t: f32) -> f32 {
    0.012 * (t - 0.42).powi(2) - 0.004
}

fn tooth_pulse(t: f32, center: f32, radius: f32) -> f32 {
    (1.0 - (t - center).abs() / radius).clamp(0.0, 1.0)
}

pub(super) fn side_width(params: &crate::TextureParameters, t: f32, side: HazelSide) -> f32 {
    if !(0.0..=1.0).contains(&t) {
        return 0.0;
    }

    // Common hazel remains broad through most of the lamina, then contracts
    // abruptly into its acuminate tip. Independent side scales keep the blade
    // organic without changing its characteristic near-orbicular proportion.
    let envelope = if t < params.hazel_leaf.side_width_envelope_1 {
        params.hazel_leaf.side_width_envelope_2
            + params.hazel_leaf.side_width_envelope_3
                * (t / params.hazel_leaf.side_width_envelope_4 * core::f32::consts::PI)
                    .sin()
                    .max(0.0)
                    .powf(params.hazel_leaf.side_width_envelope_5)
    } else {
        params.hazel_leaf.side_width_envelope_6
            * ((1.0 - t) / params.hazel_leaf.side_width_envelope_7)
                .clamp(0.0, 1.0)
                .powf(params.hazel_leaf.side_width_envelope_8)
    };
    let side_scale = match side {
        HazelSide::Left => params.hazel_leaf.side_width_side_scale_1,
        HazelSide::Right => params.hazel_leaf.side_width_side_scale_2,
    };
    let mut width = envelope * side_scale;

    // The larger tooth at each secondary-vein terminus plus two smaller teeth
    // on its shoulders produces a coarse, irregular double-serrate margin.
    for (index, vein) in veins(params, side).iter().enumerate() {
        let asymmetry = if (index + usize::from(matches!(side, HazelSide::Right))) % 3 == 0 {
            params.hazel_leaf.side_width_asymmetry_1
        } else {
            params.hazel_leaf.side_width_asymmetry_2
        };
        width += tooth_pulse(t, vein.margin, 0.022) * 0.020 * asymmetry;
        width += tooth_pulse(t, vein.margin - 0.031, 0.010) * 0.009;
        width += tooth_pulse(t, vein.margin + 0.030, 0.010) * 0.008;
    }
    width
}

fn cordate_base_minimum_t(x: f32) -> f32 {
    // The blade descends on either side of the petiole but is not filled across
    // the centre, producing the narrow heart-shaped basal notch.
    0.058 * (-(x.abs() / 0.060).powi(2)).exp()
}

fn vein_target_x(
    params: &crate::TextureParameters,
    side: HazelSide,
    vein: HazelVein,
    progress: f32,
) -> f32 {
    let terminal_width = side_width(params, vein.margin, side) * vein.reach;
    side.sign() * terminal_width * progress.powf(0.82)
}

fn tissue_relief(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    let broad = (u * params.hazel_leaf.tissue_relief_broad_1
        + v * params.hazel_leaf.tissue_relief_broad_2
        + params.hazel_leaf.tissue_relief_broad_3)
        .sin();
    let cross = (u * params.hazel_leaf.tissue_relief_cross_1
        - v * params.hazel_leaf.tissue_relief_cross_2
        + params.hazel_leaf.tissue_relief_cross_3)
        .cos();
    (broad * 0.65 + cross * 0.35) * 0.004
}

fn sample(params: &crate::TextureParameters, u: f32, v: f32) -> HazelSample {
    let longitudinal = 1.0 - v;
    let t = (longitudinal - params.hazel_leaf.sample_t_1) / params.hazel_leaf.sample_t_2;
    let x = (u - 0.5) - axis(t);
    let petiole =
        (0.018..0.105).contains(&longitudinal) && x.abs() < params.hazel_leaf.sample_petiole;
    if !(0.0..=1.0).contains(&t) {
        return HazelSample {
            inside: petiole,
            vein: petiole,
            petiole,
            height: if petiole { 0.10 } else { 0.0 },
        };
    }

    let side = if x < 0.0 {
        HazelSide::Left
    } else {
        HazelSide::Right
    };
    let width = side_width(params, t, side);
    let inside_blade = x.abs() <= width && t >= cordate_base_minimum_t(x);
    if !inside_blade && !petiole {
        return HazelSample {
            inside: false,
            vein: false,
            petiole: false,
            height: 0.0,
        };
    }

    let midrib_width =
        params.hazel_leaf.sample_midrib_width_1 - t * params.hazel_leaf.sample_midrib_width_2;
    let mut vein_distance = x.abs();
    let mut vein = x.abs() <= midrib_width;
    let mut corrugation = 0.0;
    for (index, secondary) in veins(params, side).iter().enumerate() {
        let progress =
            ((t - secondary.origin) / (secondary.margin - secondary.origin)).clamp(0.0, 1.0);
        if progress > 0.0 && progress < 1.0 {
            let target_x = vein_target_x(params, side, *secondary, progress);
            let distance = (x - target_x).abs();
            vein_distance = vein_distance.min(distance);
            vein |= distance < 0.0040;
            let fold = if index % 2 == 0 { 1.0 } else { -1.0 };
            corrugation += fold
                * (1.0 - distance / 0.055).clamp(0.0, 1.0)
                * (progress * core::f32::consts::PI).sin()
                * 0.014;
        }
    }

    let transverse = (x / width.max(params.hazel_leaf.sample_transverse)).clamp(-1.0, 1.0);
    let blade_dome = (1.0 - transverse * transverse).powf(params.hazel_leaf.sample_blade_dome_1)
        * params.hazel_leaf.sample_blade_dome_2;
    let longitudinal_dome = (t * core::f32::consts::PI).sin().max(0.0).sqrt();
    let vein_ridge = (1.0 - vein_distance / params.hazel_leaf.sample_vein_ridge_1)
        .clamp(0.0, 1.0)
        .powi(2)
        * params.hazel_leaf.sample_vein_ridge_2;
    let height = if petiole && !inside_blade {
        params.hazel_leaf.sample_height_1
    } else {
        (blade_dome * longitudinal_dome + vein_ridge + corrugation + tissue_relief(params, u, v))
            .clamp(
                params.hazel_leaf.sample_height_2,
                params.hazel_leaf.sample_height_3,
            )
    };
    HazelSample {
        inside: true,
        vein: vein || petiole,
        petiole,
        height,
    }
}

fn coverage(params: &crate::TextureParameters, u: f32, v: f32, texel: f32) -> bool {
    let mut covered = 0;
    for sample_y in 0..EDGE_SAMPLES {
        for sample_x in 0..EDGE_SAMPLES {
            let offset_x = (sample_x as f32 + 0.5) / EDGE_SAMPLES as f32 - 0.5;
            let offset_y = (sample_y as f32 + 0.5) / EDGE_SAMPLES as f32 - 0.5;
            covered += u32::from(sample(params, u + offset_x * texel, v + offset_y * texel).inside);
        }
    }
    covered * 2 >= EDGE_SAMPLES.pow(2)
}

pub(super) fn generate(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
    recipe: LeafRecipe,
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
            let leaf = sample(params, u, v);
            let inside = coverage(params, u, v, texel);
            let alpha = if inside { 255 } else { 0 };
            opacity.extend_from_slice(&[alpha; 4]);
            let front_color = if leaf.vein { recipe.vein } else { recipe.blade };
            let back_color = if leaf.vein {
                recipe.vein
            } else {
                recipe.back_blade
            };
            if inside {
                front.extend_from_slice(&[front_color[0], front_color[1], front_color[2], alpha]);
                back.extend_from_slice(&[back_color[0], back_color[1], back_color[2], alpha]);
            } else {
                front.extend_from_slice(&[0; 4]);
                back.extend_from_slice(&[0; 4]);
            }

            let hx = sample(params, (u + texel).min(1.0), v).height
                - sample(params, (u - texel).max(0.0), v).height;
            let hy = sample(params, u, (v + texel).min(1.0)).height
                - sample(params, u, (v - texel).max(0.0)).height;
            let front_surface_normal = Vec3::new(-hx * 7.0, hy * 7.0, 1.0).normalize();
            let front_encoded =
                ((front_surface_normal + Vec3::ONE) * 127.5).clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal_front.extend_from_slice(&[
                front_encoded.x as u8,
                front_encoded.y as u8,
                front_encoded.z as u8,
                255,
            ]);
            let back_surface_normal = Vec3::new(-hx * 7.8, hy * 7.8, 1.0).normalize();
            let back_encoded =
                ((back_surface_normal + Vec3::ONE) * 127.5).clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal_back.extend_from_slice(&[
                back_encoded.x as u8,
                (255.0 - back_encoded.y) as u8,
                back_encoded.z as u8,
                255,
            ]);

            let height = if inside { leaf.height } else { 0.0 };
            let ao = if !inside {
                255
            } else if leaf.petiole {
                235
            } else if leaf.vein {
                (params.hazel_leaf.generate_ao_1 + height * params.hazel_leaf.generate_ao_2)
                    .min(params.hazel_leaf.generate_ao_3) as u8
            } else {
                (params.hazel_leaf.generate_ao_4 + height * params.hazel_leaf.generate_ao_5).clamp(
                    params.hazel_leaf.generate_ao_6,
                    params.hazel_leaf.generate_ao_7,
                ) as u8
            };
            let encoded_height = (height * 255.0).clamp(0.0, 255.0) as u8;
            height_map.extend_from_slice(&[encoded_height, encoded_height, encoded_height, 255]);
            arm.extend_from_slice(&[ao, recipe.roughness, 0, 255]);
        }
    }

    crate::leaf_pixels::LeafPixels {
        opacity,
        front,
        back,
        normal_front,
        normal_back,
        height_map,
        arm,
    }
    .upload(params, images)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn silhouette_is_broad_cordate_and_abruptly_acuminate() {
        let params = &crate::TextureParameters::default();
        let middle =
            side_width(params, 0.44, HazelSide::Left) + side_width(params, 0.44, HazelSide::Right);
        let length = 1.0;
        assert!(
            (0.78..=1.02).contains(&(middle / length)),
            "width ratio: {middle}"
        );
        assert!(
            side_width(params, 0.72, HazelSide::Left)
                > side_width(params, 0.88, HazelSide::Left) * 2.5
        );
        assert!(!sample(params, 0.5, 1.0 - (0.09 + 0.02 * 0.82)).inside);
        assert!(sample(params, 0.43, 1.0 - (0.09 + 0.02 * 0.82)).inside);
        assert!(sample(params, 0.5, 0.95).petiole);
    }

    #[test]
    fn eight_asymmetric_secondary_veins_reach_their_margin_teeth() {
        let params = &crate::TextureParameters::default();
        for side in [HazelSide::Left, HazelSide::Right] {
            for secondary in veins(params, side) {
                let progress = 0.92;
                let t = secondary.origin + (secondary.margin - secondary.origin) * progress;
                let x = vein_target_x(params, side, *secondary, progress);
                let u = 0.5 + axis(t) + x;
                let v = 1.0 - (0.09 + t * 0.82);
                assert!(
                    sample(params, u, v).vein,
                    "secondary at t={t} reaches its tooth"
                );
            }
        }
        assert_ne!(LEFT_VEINS[2].margin, RIGHT_VEINS[2].margin);
    }

    #[test]
    fn generated_channels_are_detailed_palette_bounded_and_mip_complete() {
        let params = &crate::TextureParameters::default();
        let mut images = Assets::<Image>::default();
        let textures = generate(params, &mut images, LeafRecipe::HAZEL);
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
        assert_eq!(
            images
                .get(&textures.opacity)
                .unwrap()
                .data
                .as_ref()
                .unwrap()
                .last(),
            Some(&255),
            "coverage survives distant mips"
        );

        let front = &images
            .get(&textures.front_albedo)
            .unwrap()
            .data
            .as_ref()
            .unwrap()[..base_bytes];
        let front_colors = front
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] > 0)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<BTreeSet<_>>();
        assert_eq!(front_colors.len(), 2);

        let height = &images.get(&textures.height).unwrap().data.as_ref().unwrap()[..base_bytes];
        assert!(
            height
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| pixel[0])
                .collect::<BTreeSet<_>>()
                .len()
                > 48
        );
        let normal = &images
            .get(&textures.front_normal)
            .unwrap()
            .data
            .as_ref()
            .unwrap()[..base_bytes];
        assert!(
            normal
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| (pixel[0], pixel[1]))
                .collect::<BTreeSet<_>>()
                .len()
                > 200
        );
    }

    #[test]
    fn generation_is_repeatable() {
        let params = &crate::TextureParameters::default();
        let mut first_images = Assets::<Image>::default();
        let first = generate(params, &mut first_images, LeafRecipe::HAZEL);
        let mut second_images = Assets::<Image>::default();
        let second = generate(params, &mut second_images, LeafRecipe::HAZEL);
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

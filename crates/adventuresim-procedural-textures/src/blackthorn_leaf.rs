use bevy::{asset::Assets, image::Image, math::Vec3};

use crate::{LeafRecipe, LeafTextureSet, TEXTURE_SIZE};

const EDGE_SAMPLES: u32 = 4;
const TOOTH_COUNT: usize = 13;

#[derive(Clone, Copy, Debug)]
enum BlackthornSide {
    Left,
    Right,
}

impl BlackthornSide {
    const fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BlackthornVein {
    origin: f32,
    margin: f32,
    reach: f32,
}

const LEFT_VEINS: [BlackthornVein; 6] = [
    BlackthornVein {
        origin: 0.095,
        margin: 0.230,
        reach: 0.88,
    },
    BlackthornVein {
        origin: 0.220,
        margin: 0.365,
        reach: 0.91,
    },
    BlackthornVein {
        origin: 0.355,
        margin: 0.505,
        reach: 0.92,
    },
    BlackthornVein {
        origin: 0.495,
        margin: 0.640,
        reach: 0.91,
    },
    BlackthornVein {
        origin: 0.635,
        margin: 0.765,
        reach: 0.88,
    },
    BlackthornVein {
        origin: 0.755,
        margin: 0.865,
        reach: 0.82,
    },
];

const RIGHT_VEINS: [BlackthornVein; 6] = [
    BlackthornVein {
        origin: 0.105,
        margin: 0.245,
        reach: 0.87,
    },
    BlackthornVein {
        origin: 0.235,
        margin: 0.380,
        reach: 0.90,
    },
    BlackthornVein {
        origin: 0.370,
        margin: 0.520,
        reach: 0.91,
    },
    BlackthornVein {
        origin: 0.510,
        margin: 0.655,
        reach: 0.90,
    },
    BlackthornVein {
        origin: 0.650,
        margin: 0.778,
        reach: 0.87,
    },
    BlackthornVein {
        origin: 0.770,
        margin: 0.875,
        reach: 0.80,
    },
];

#[derive(Clone, Copy)]
struct BlackthornSample {
    inside: bool,
    vein: bool,
    petiole: bool,
    height: f32,
    back_height: f32,
}

fn veins(params: &crate::TextureParameters, side: BlackthornSide) -> &[BlackthornVein; 6] {
    match side {
        BlackthornSide::Left => &params.blackthorn_leaf.left_veins,
        BlackthornSide::Right => &params.blackthorn_leaf.right_veins,
    }
}

fn axis(t: f32) -> f32 {
    0.008 * (t - 0.38).powi(2) - 0.002
}

fn smooth_pulse(t: f32, center: f32, radius: f32) -> f32 {
    let distance = ((t - center) / radius).abs();
    if distance >= 1.0 {
        0.0
    } else {
        let rounded = 1.0 - distance * distance;
        rounded * rounded
    }
}

fn tooth_center(params: &crate::TextureParameters, index: usize, side: BlackthornSide) -> f32 {
    let offset = match side {
        BlackthornSide::Left => -params.blackthorn_leaf.tooth_center_offset_1,
        BlackthornSide::Right => params.blackthorn_leaf.tooth_center_offset_2,
    };
    0.115 + (index as f32 + 0.5) / params.blackthorn_leaf.tooth_count as f32 * 0.785 + offset
}

fn tooth_extension(params: &crate::TextureParameters, t: f32, side: BlackthornSide) -> f32 {
    (0..params.blackthorn_leaf.tooth_count)
        .map(|index| {
            let pattern = index + usize::from(matches!(side, BlackthornSide::Right));
            let amplitude = match pattern % 4 {
                0 => params.blackthorn_leaf.tooth_extension_amplitude_1,
                1 => params.blackthorn_leaf.tooth_extension_amplitude_2,
                2 => params.blackthorn_leaf.tooth_extension_amplitude_3,
                _ => params.blackthorn_leaf.tooth_extension_amplitude_4,
            };
            smooth_pulse(t, tooth_center(params, index, side), 0.025) * amplitude
        })
        .sum()
}

fn side_width(params: &crate::TextureParameters, t: f32, side: BlackthornSide) -> f32 {
    if !(0.0..=1.0).contains(&t) {
        return 0.0;
    }
    // A rounded basal contribution blends into a broad elliptic/obovate
    // lamina. The sine envelope keeps the distal half full before contracting
    // into a short tip instead of producing the generic diamond silhouette.
    let basal = params.blackthorn_leaf.side_width_basal_1
        * (1.0 - t).powf(params.blackthorn_leaf.side_width_basal_2);
    let broad_lamina = params.blackthorn_leaf.side_width_broad_lamina_1
        * (t * core::f32::consts::PI)
            .sin()
            .max(0.0)
            .powf(params.blackthorn_leaf.side_width_broad_lamina_2)
        * (params.blackthorn_leaf.side_width_broad_lamina_3
            + params.blackthorn_leaf.side_width_broad_lamina_4 * t);
    let envelope = basal + broad_lamina;
    let side_scale = match side {
        BlackthornSide::Left => params.blackthorn_leaf.side_width_side_scale_1,
        BlackthornSide::Right => params.blackthorn_leaf.side_width_side_scale_2,
    };
    envelope * side_scale + tooth_extension(params, t, side)
}

fn rounded_base_minimum_t(x: f32) -> f32 {
    let normalized = (x.abs() / 0.12).clamp(0.0, 1.0);
    0.045 * normalized * normalized
}

fn vein_target_x(
    params: &crate::TextureParameters,
    side: BlackthornSide,
    vein: BlackthornVein,
    progress: f32,
) -> f32 {
    side.sign() * side_width(params, vein.margin, side) * vein.reach * progress.powf(0.86)
}

fn tissue_relief(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    let broad = (u * params.blackthorn_leaf.tissue_relief_broad_1
        + v * params.blackthorn_leaf.tissue_relief_broad_2
        + params.blackthorn_leaf.tissue_relief_broad_3)
        .sin();
    let cross = (u * params.blackthorn_leaf.tissue_relief_cross_1
        - v * params.blackthorn_leaf.tissue_relief_cross_2
        + params.blackthorn_leaf.tissue_relief_cross_3)
        .cos();
    (broad * 0.6 + cross * 0.4) * 0.003
}

fn sample(params: &crate::TextureParameters, u: f32, v: f32) -> BlackthornSample {
    let longitudinal = 1.0 - v;
    let t = (longitudinal - params.blackthorn_leaf.sample_t_1) / params.blackthorn_leaf.sample_t_2;
    let x = (u - 0.5) - axis(t);
    let petiole =
        (0.022..0.105).contains(&longitudinal) && x.abs() < params.blackthorn_leaf.sample_petiole;
    if !(0.0..=1.0).contains(&t) {
        let height = if petiole {
            params.blackthorn_leaf.sample_height
        } else {
            0.0
        };
        return BlackthornSample {
            inside: petiole,
            vein: petiole,
            petiole,
            height,
            back_height: height,
        };
    }

    let side = if x < 0.0 {
        BlackthornSide::Left
    } else {
        BlackthornSide::Right
    };
    let width = side_width(params, t, side);
    let inside_blade = x.abs() <= width && t >= rounded_base_minimum_t(x);
    if !inside_blade && !petiole {
        return BlackthornSample {
            inside: false,
            vein: false,
            petiole: false,
            height: 0.0,
            back_height: 0.0,
        };
    }

    let midrib_width = params.blackthorn_leaf.sample_midrib_width_1
        - t * params.blackthorn_leaf.sample_midrib_width_2;
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
            vein |= distance < 0.0033;
            let fold = if index % 2 == 0 { 1.0 } else { -1.0 };
            corrugation += fold
                * (1.0 - distance / 0.052).clamp(0.0, 1.0)
                * (progress * core::f32::consts::PI).sin()
                * 0.009;
        }
    }

    let transverse = (x / width.max(params.blackthorn_leaf.sample_transverse)).clamp(-1.0, 1.0);
    let blade_dome = (1.0 - transverse * transverse)
        .powf(params.blackthorn_leaf.sample_blade_dome_1)
        * params.blackthorn_leaf.sample_blade_dome_2;
    let longitudinal_dome = (t * core::f32::consts::PI).sin().max(0.0).sqrt();
    let vein_ridge = (1.0 - vein_distance / params.blackthorn_leaf.sample_vein_ridge_1)
        .clamp(0.0, 1.0)
        .powi(2)
        * params.blackthorn_leaf.sample_vein_ridge_2;
    let height = if petiole && !inside_blade {
        params.blackthorn_leaf.sample_height_1
    } else {
        (blade_dome * longitudinal_dome + vein_ridge + corrugation + tissue_relief(params, u, v))
            .clamp(
                params.blackthorn_leaf.sample_height_2,
                params.blackthorn_leaf.sample_height_3,
            )
    };
    // The lower surface is commonly hairy along the veins. Keep albedo clean
    // and express that bounded underside difference through relief instead.
    let underside_vein_relief = (1.0
        - vein_distance / params.blackthorn_leaf.sample_underside_vein_relief_1)
        .clamp(0.0, 1.0)
        .powi(2)
        * params.blackthorn_leaf.sample_underside_vein_relief_2;
    BlackthornSample {
        inside: true,
        vein: vein || petiole,
        petiole,
        height,
        back_height: height + underside_vein_relief,
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

fn encoded_normal(
    height_left: f32,
    height_right: f32,
    height_down: f32,
    height_up: f32,
) -> [u8; 3] {
    let hx = height_right - height_left;
    let hy = height_up - height_down;
    let normal = Vec3::new(-hx * 7.2, hy * 7.2, 1.0).normalize();
    let encoded = ((normal + Vec3::ONE) * 127.5).clamp(Vec3::ZERO, Vec3::splat(255.0));
    [encoded.x as u8, encoded.y as u8, encoded.z as u8]
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

            let left = sample(params, (u - texel).max(0.0), v);
            let right = sample(params, (u + texel).min(1.0), v);
            let down = sample(params, u, (v - texel).max(0.0));
            let up = sample(params, u, (v + texel).min(1.0));
            let front_encoded = encoded_normal(left.height, right.height, down.height, up.height);
            normal_front.extend_from_slice(&[
                front_encoded[0],
                front_encoded[1],
                front_encoded[2],
                255,
            ]);
            let back_encoded = encoded_normal(
                left.back_height,
                right.back_height,
                down.back_height,
                up.back_height,
            );
            normal_back.extend_from_slice(&[
                back_encoded[0],
                255 - back_encoded[1],
                back_encoded[2],
                255,
            ]);

            let height = if inside { leaf.height } else { 0.0 };
            let ao = if !inside {
                255
            } else if leaf.petiole {
                234
            } else if leaf.vein {
                (params.blackthorn_leaf.generate_ao_1
                    + height * params.blackthorn_leaf.generate_ao_2)
                    .min(params.blackthorn_leaf.generate_ao_3) as u8
            } else {
                (params.blackthorn_leaf.generate_ao_4
                    + height * params.blackthorn_leaf.generate_ao_5)
                    .clamp(
                        params.blackthorn_leaf.generate_ao_6,
                        params.blackthorn_leaf.generate_ao_7,
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

    fn base_bytes(image: &Image) -> &[u8] {
        &image.data.as_ref().unwrap()[..(TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize]
    }

    #[test]
    fn silhouette_is_compact_rounded_at_base_and_short_tipped() {
        let params = &crate::TextureParameters::default();
        let middle = side_width(params, 0.46, BlackthornSide::Left)
            + side_width(params, 0.46, BlackthornSide::Right);
        assert!(
            (0.76..=0.84).contains(&middle),
            "plate width ratio: {middle}"
        );
        assert!(
            side_width(params, 0.90, BlackthornSide::Left)
                > side_width(params, 0.98, BlackthornSide::Left) * 2.5
        );
        let basal_v = 1.0 - (0.09 + 0.018 * 0.82);
        assert!(
            sample(params, 0.5 + axis(0.018), basal_v).inside,
            "base has no cordate notch"
        );
        assert!(
            !sample(params, 0.62 + axis(0.018), basal_v).inside,
            "base edge rounds upward"
        );
        assert!(sample(params, 0.5, 0.95).petiole);
    }

    #[test]
    fn margin_teeth_are_shallow_blunt_and_asymmetric() {
        let params = &crate::TextureParameters::default();
        for side in [BlackthornSide::Left, BlackthornSide::Right] {
            for index in 0..TOOTH_COUNT {
                let center = tooth_center(params, index, side);
                let peak = tooth_extension(params, center, side);
                assert!(
                    (0.006..=0.011).contains(&peak),
                    "tooth {index} extension: {peak}"
                );
                assert!(
                    peak > tooth_extension(params, center - 0.027, side)
                        && peak > tooth_extension(params, center + 0.027, side),
                    "tooth {index} is a rounded marginal projection"
                );
            }
        }
        assert_ne!(
            tooth_center(params, 4, BlackthornSide::Left),
            tooth_center(params, 4, BlackthornSide::Right)
        );
    }

    #[test]
    fn six_asymmetric_secondary_veins_approach_the_margin() {
        let params = &crate::TextureParameters::default();
        for side in [BlackthornSide::Left, BlackthornSide::Right] {
            for secondary in veins(params, side) {
                let progress = 0.9;
                let t = secondary.origin + (secondary.margin - secondary.origin) * progress;
                let x = vein_target_x(params, side, *secondary, progress);
                let u = 0.5 + axis(t) + x;
                let v = 1.0 - (0.09 + t * 0.82);
                assert!(
                    sample(params, u, v).vein,
                    "secondary at t={t} remains coherent"
                );
            }
        }
        assert_ne!(LEFT_VEINS[2].margin, RIGHT_VEINS[2].margin);
    }

    #[test]
    fn generated_channels_are_palette_bounded_alpha_matched_and_mip_complete() {
        let params = &crate::TextureParameters::default();
        let mut images = Assets::<Image>::default();
        let textures = generate(params, &mut images, LeafRecipe::BLACKTHORN);
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

        let opacity = images.get(&textures.opacity).unwrap();
        let front = images.get(&textures.front_albedo).unwrap();
        let back = images.get(&textures.back_albedo).unwrap();
        let opacity_pixels = base_bytes(opacity).as_chunks::<4>().0;
        let front_pixels = base_bytes(front).as_chunks::<4>().0;
        let back_pixels = base_bytes(back).as_chunks::<4>().0;
        assert!(
            opacity_pixels
                .iter()
                .all(|pixel| pixel[0] == 0 || pixel[0] == 255)
        );
        assert!(
            opacity_pixels
                .iter()
                .zip(front_pixels)
                .zip(back_pixels)
                .all(|((opacity, front), back)| opacity[0] == front[3] && opacity[0] == back[3])
        );
        for pixels in [front_pixels, back_pixels] {
            let colors = pixels
                .iter()
                .filter(|pixel| pixel[3] > 0)
                .map(|pixel| [pixel[0], pixel[1], pixel[2]])
                .collect::<BTreeSet<_>>();
            assert_eq!(colors.len(), 2);
        }

        let front_data = images
            .get(&textures.front_albedo)
            .unwrap()
            .data
            .as_ref()
            .unwrap();
        let opacity_data = images
            .get(&textures.opacity)
            .unwrap()
            .data
            .as_ref()
            .unwrap();
        assert!(
            opacity_data
                .as_chunks::<4>()
                .0
                .iter()
                .zip(front_data.as_chunks::<4>().0)
                .all(|(opacity, color)| opacity[0] == color[3]),
            "alpha remains coverage-matched through every mip"
        );
        assert_eq!(
            opacity_data.last(),
            Some(&255),
            "coverage survives distant mips"
        );
    }

    #[test]
    fn generated_relief_is_detailed_and_front_back_normals_are_unit_length() {
        let params = &crate::TextureParameters::default();
        let mut images = Assets::<Image>::default();
        let textures = generate(params, &mut images, LeafRecipe::BLACKTHORN);
        let height = base_bytes(images.get(&textures.height).unwrap());
        assert!(
            height
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| pixel[0])
                .collect::<BTreeSet<_>>()
                .len()
                > 40
        );
        for handle in [&textures.front_normal, &textures.back_normal] {
            for pixel in base_bytes(images.get(handle).unwrap()).as_chunks::<4>().0 {
                let normal = Vec3::new(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32) / 127.5
                    - Vec3::ONE;
                assert!((0.97..=1.03).contains(&normal.length()));
            }
        }
    }

    #[test]
    fn generation_is_repeatable() {
        let params = &crate::TextureParameters::default();
        let mut first_images = Assets::<Image>::default();
        let first = generate(params, &mut first_images, LeafRecipe::BLACKTHORN);
        let mut second_images = Assets::<Image>::default();
        let second = generate(params, &mut second_images, LeafRecipe::BLACKTHORN);
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

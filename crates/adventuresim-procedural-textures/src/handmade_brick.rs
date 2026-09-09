//! Hand-moulded early-modern brickwork with a seamless running bond.
//!
//! Original design evidence:
//! prior-art/handmade-brick/prior-art.md.
//! That report records a research snapshot; this module and its tests define
//! current behavior.

use bevy::{asset::Assets, image::Image, math::Vec3};
use fabelgeist_determinism::inclusive_unit_f32;

use super::{
    MasonryColors, SrgbColor, SurfaceTextureSet, image_rgba_mipped, palette::albedo_image,
};

pub const HANDMADE_BRICK_TEXTURE_SIZE: u32 = 1024;
pub const HANDMADE_BRICK_TILE_METRES: f32 = 2.4;
pub const HANDMADE_BRICK_HEIGHT_RANGE_METRES: f32 = 0.014;

pub const HANDMADE_BRICK_COLORS: MasonryColors<5> = MasonryColors {
    units: [
        SrgbColor([132, 63, 43]),
        SrgbColor([145, 70, 47]),
        SrgbColor([119, 54, 40]),
        SrgbColor([154, 78, 51]),
        SrgbColor([128, 58, 45]),
    ],
    mortar: SrgbColor([142, 134, 116]),
};
const BRICK_ROUGHNESS: u8 = 218;
const MORTAR_ROUGHNESS: u8 = 236;

const COURSES: i32 = 30;
const BRICKS_PER_COURSE: i32 = 10;
const HORIZONTAL_MORTAR_METRES: f32 = 0.014;
const VERTICAL_MORTAR_METRES: f32 = 0.014;

#[derive(Clone, Copy)]
struct BrickSample {
    height: f32,
    brick: bool,
    brick_id: u64,
    brick_coverage: f32,
}

fn hash_unit(params: &crate::TextureParameters, value: u64) -> f32 {
    inclusive_unit_f32(crate::parameters::seeded_hash(params, value))
}

fn brick_id(params: &crate::TextureParameters, row: i32, column: i32) -> u64 {
    let wrapped_row = row.rem_euclid(params.handmade_brick.courses) as u64;
    let wrapped_column = column.rem_euclid(params.handmade_brick.bricks_per_course) as u64;
    crate::parameters::seeded_hash(params, 0x8d31_5a29 ^ (wrapped_row << 32) ^ wrapped_column)
}

fn periodic_delta(value: f32) -> f32 {
    value - value.round()
}

fn face_noise(params: &crate::TextureParameters, local_x: f32, local_y: f32, id: u64) -> f32 {
    let phase_a = hash_unit(params, id ^ 0x3ac7) * std::f32::consts::TAU;
    let phase_b = hash_unit(params, id ^ 0xd159) * std::f32::consts::TAU;
    let broad = (local_x * params.handmade_brick.broad_cross_frequency
        + local_y * params.handmade_brick.broad_long_frequency
        + phase_a)
        .sin();
    let crossed = (local_x * params.handmade_brick.fine_cross_frequency
        - local_y * params.handmade_brick.fine_long_frequency
        + phase_b)
        .sin();
    broad * params.handmade_brick.broad_weight + crossed * params.handmade_brick.fine_weight
}

fn bowed_edge(params: &crate::TextureParameters, coordinate: f32, id: u64, salt: u64) -> f32 {
    let phase = hash_unit(params, id ^ salt) * std::f32::consts::TAU;
    let amplitude = params.handmade_brick.edge_bow_min
        + hash_unit(params, id ^ salt.rotate_left(17)) * params.handmade_brick.edge_bow_variation;
    (coordinate * std::f32::consts::PI + phase).sin() * amplitude
}

fn edge_chip(params: &crate::TextureParameters, coordinate: f32, id: u64, salt: u64) -> f32 {
    if hash_unit(params, id ^ salt) < params.handmade_brick.chip_absence_probability {
        return 0.0;
    }
    let center = hash_unit(params, id ^ salt.rotate_left(11)).mul_add(
        params.handmade_brick.edge_chip_center_1,
        -params.handmade_brick.edge_chip_center_2,
    );
    let half_width = params.handmade_brick.chip_half_width
        + hash_unit(params, id ^ salt.rotate_left(23)) * params.handmade_brick.chip_width_variation;
    let distance = ((coordinate - center) / half_width).abs();
    let profile = (1.0 - distance).max(0.0);
    profile
        * profile
        * (params.handmade_brick.chip_depth
            + hash_unit(params, id ^ salt.rotate_left(37))
                * params.handmade_brick.chip_depth_variation)
}

fn sample_brickwork(params: &crate::TextureParameters, u: f32, v: f32) -> BrickSample {
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let pitch_x = 1.0 / params.handmade_brick.bricks_per_course as f32;
    let pitch_y = 1.0 / params.handmade_brick.courses as f32;
    let nominal_half_width = (pitch_x
        - params.handmade_brick.vertical_mortar_metres / params.handmade_brick.tile_metres)
        * 0.5;
    let nominal_half_height = (pitch_y
        - params.handmade_brick.horizontal_mortar_metres / params.handmade_brick.tile_metres)
        * 0.5;
    let base_row = (v / pitch_y).floor() as i32;
    let mut best = (f32::INFINITY, 0_u64, 0.0_f32, 0.0_f32, 1.0_f32);

    for row in (base_row - 1)..=(base_row + 1) {
        let offset = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
        let base_column = (u / pitch_x - offset).floor() as i32;
        for column in (base_column - 1)..=(base_column + 1) {
            let id = brick_id(params, row, column);
            let center_x = (column as f32 + 0.5 + offset) * pitch_x
                + (hash_unit(params, id ^ 0x31b7) - 0.5)
                    * pitch_x
                    * params.handmade_brick.horizontal_jitter;
            let center_y = (row as f32 + 0.5) * pitch_y
                + (hash_unit(params, id ^ 0x91e5) - 0.5)
                    * pitch_y
                    * params.handmade_brick.vertical_jitter;
            let dx = periodic_delta(u - center_x);
            let dy = periodic_delta(v - center_y);
            let width = nominal_half_width
                * (params.handmade_brick.width_minimum
                    + hash_unit(params, id ^ 0xe421) * params.handmade_brick.width_variation);
            let height = nominal_half_height
                * (params.handmade_brick.height_minimum
                    + hash_unit(params, id ^ 0x72dd) * params.handmade_brick.height_variation);
            let local_x = dx / width;
            let local_y = dy / height;
            let right = 1.0 + bowed_edge(params, local_y, id, 0x44a1)
                - edge_chip(params, local_y, id, 0x5db7);
            let left = 1.0 + bowed_edge(params, local_y, id, 0xb837)
                - edge_chip(params, local_y, id, 0xa251);
            let top = 1.0 + bowed_edge(params, local_x, id, 0x2db9)
                - edge_chip(params, local_x, id, 0x71c3);
            let bottom = 1.0 + bowed_edge(params, local_x, id, 0xf137)
                - edge_chip(params, local_x, id, 0xce29);
            let edge_distance = (local_x - right)
                .max(-local_x - left)
                .max(local_y - top)
                .max(-local_y - bottom);
            if edge_distance < best.0 {
                best = (edge_distance, id, local_x, local_y, width.min(height));
            }
        }
    }

    surface::finish(params, u, v, best)
}

fn brick_color(
    params: &crate::TextureParameters,
    sample: BrickSample,
    colors: &MasonryColors<5>,
) -> ([u8; 3], u8) {
    let unit = colors.units[sample.brick_id as usize % colors.units.len()];
    let color = colors.mortar.covered_by(unit, sample.brick_coverage).0;
    let roughness = if sample.brick {
        params.handmade_brick.brick_roughness
    } else {
        params.handmade_brick.mortar_roughness
    };
    (color, roughness)
}

fn height_at(params: &crate::TextureParameters, heights: &[f32], x: i32, y: i32) -> f32 {
    let size = params.size(HANDMADE_BRICK_TEXTURE_SIZE) as i32;
    heights[(y.rem_euclid(size) * size + x.rem_euclid(size)) as usize]
}

fn ambient_visibility(params: &crate::TextureParameters, heights: &[f32], x: i32, y: i32) -> f32 {
    let center = height_at(params, heights, x, y);
    let mut obstruction = 0.0_f32;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for step in [1, 3, 7, 15] {
            let rise = height_at(params, heights, x + dx * step, y + dy * step) - center;
            obstruction += (rise / step as f32).max(0.0);
        }
    }
    (1.0 - obstruction * params.handmade_brick.occlusion_strength)
        .clamp(params.handmade_brick.minimum_visibility, 1.0)
}

pub fn generate_handmade_brick_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> SurfaceTextureSet {
    let colors = &params.handmade_brick.colors;
    let size = params.size(HANDMADE_BRICK_TEXTURE_SIZE);
    let samples = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                sample_brickwork(
                    params,
                    (x as f32 + 0.5) / size as f32,
                    (y as f32 + 0.5) / size as f32,
                )
            })
        })
        .collect::<Vec<_>>();
    let heights = samples
        .iter()
        .map(|sample| sample.height)
        .collect::<Vec<_>>();
    let mut albedo = Vec::with_capacity((size * size * 4) as usize);
    let mut normal = Vec::with_capacity(albedo.capacity());
    let mut height = Vec::with_capacity(albedo.capacity());
    let mut arm = Vec::with_capacity(albedo.capacity());
    let metres_per_texel = params.handmade_brick.tile_metres / size as f32;
    let slope_scale = params.handmade_brick.height_range_metres / (2.0 * metres_per_texel);

    for y in 0..size {
        for x in 0..size {
            let index = (y * size + x) as usize;
            let sample = samples[index];
            let (color, roughness) = brick_color(params, sample, colors);
            albedo.extend_from_slice(&[color[0], color[1], color[2], 255]);
            let dx = height_at(params, &heights, x as i32 + 1, y as i32)
                - height_at(params, &heights, x as i32 - 1, y as i32);
            let dy = height_at(params, &heights, x as i32, y as i32 + 1)
                - height_at(params, &heights, x as i32, y as i32 - 1);
            let surface_normal =
                crate::normal::from_image_gradient(dx * slope_scale, dy * slope_scale);
            let encoded = ((surface_normal + Vec3::ONE) * 127.5)
                .round()
                .clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal.extend_from_slice(&[encoded.x as u8, encoded.y as u8, encoded.z as u8, 255]);
            let encoded_height = (sample.height * 255.0).round().clamp(0.0, 255.0) as u8;
            height.extend_from_slice(&[encoded_height, encoded_height, encoded_height, 255]);
            let ao = (ambient_visibility(params, &heights, x as i32, y as i32) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            arm.extend_from_slice(&[ao, roughness, 0, 255]);
        }
    }

    let albedo_image = albedo_image(albedo, size);
    SurfaceTextureSet {
        albedo: images.add(albedo_image),
        normal_gl: images.add(image_rgba_mipped(normal, size, true)),
        height: images.add(image_rgba_mipped(height, size, true)),
        arm: images.add(image_rgba_mipped(arm, size, true)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bevy::render::render_resource::TextureFormat;

    use super::*;

    fn generated() -> (Assets<Image>, SurfaceTextureSet) {
        let params = &crate::TextureParameters::default();
        let mut images = Assets::default();
        let textures = generate_handmade_brick_textures(params, &mut images);
        (images, textures)
    }

    #[test]
    fn unit_and_mortar_colors_are_independent_with_antialiased_contacts() {
        let params = &crate::TextureParameters::default();
        let colors = HANDMADE_BRICK_COLORS;
        let mut recolored_units = colors;
        recolored_units.units.fill(SrgbColor([40, 70, 100]));
        let mut recolored_mortar = colors;
        recolored_mortar.mortar = SrgbColor([210, 190, 160]);
        let mut regions = [false; 3];
        for y in 0..128 {
            for x in 0..128 {
                let sample =
                    sample_brickwork(params, (x as f32 + 0.5) / 128.0, (y as f32 + 0.5) / 128.0);
                let original = brick_color(params, sample, &colors);
                let units = brick_color(params, sample, &recolored_units);
                let mortar = brick_color(params, sample, &recolored_mortar);
                assert_eq!(original.1, units.1);
                assert_eq!(original.1, mortar.1);
                if sample.brick_coverage == 0.0 {
                    regions[0] = true;
                    assert_eq!(units, original);
                    assert_eq!(mortar.0, recolored_mortar.mortar.0);
                } else if sample.brick_coverage == 1.0 {
                    regions[1] = true;
                    assert_eq!(mortar, original);
                    assert_eq!(units.0, recolored_units.units[0].0);
                } else {
                    regions[2] = true;
                }
            }
        }
        assert!(regions.into_iter().all(|observed| observed));
    }

    #[test]
    fn generation_is_deterministic() {
        let (first_images, first) = generated();
        let (second_images, second) = generated();
        for (first_handle, second_handle) in [
            (&first.albedo, &second.albedo),
            (&first.normal_gl, &second.normal_gl),
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
    fn analytic_field_tiles_continuously() {
        let params = &crate::TextureParameters::default();
        let epsilon = 0.2 / HANDMADE_BRICK_TEXTURE_SIZE as f32;
        let mut maximum_error = 0.0_f32;
        for index in 0..512 {
            let coordinate = (index as f32 + 0.5) / 512.0;
            maximum_error = maximum_error
                .max(
                    (sample_brickwork(params, epsilon, coordinate).height
                        - sample_brickwork(params, 1.0 - epsilon, coordinate).height)
                        .abs(),
                )
                .max(
                    (sample_brickwork(params, coordinate, epsilon).height
                        - sample_brickwork(params, coordinate, 1.0 - epsilon).height)
                        .abs(),
                );
        }
        assert!(maximum_error < 0.08, "seam height error: {maximum_error}");
    }

    #[test]
    fn running_bond_offsets_vertical_joints_by_half_a_brick() {
        let params = &crate::TextureParameters::default();
        let even_course = (0.5_f32) / COURSES as f32;
        let odd_course = (1.5_f32) / COURSES as f32;
        let pitch = 1.0 / BRICKS_PER_COURSE as f32;
        assert!(sample_brickwork(params, pitch * 0.5, even_course).brick);
        assert!(sample_brickwork(params, pitch, odd_course).brick);
        let mut even_joints = 0;
        let mut odd_joints = 0;
        let mut aligned_joints = 0;
        for sample in 0..1_000 {
            let u = (sample as f32 + 0.5) / 1_000.0;
            let even_joint = !sample_brickwork(params, u, even_course).brick;
            let odd_joint = !sample_brickwork(params, u, odd_course).brick;
            even_joints += usize::from(even_joint);
            odd_joints += usize::from(odd_joint);
            aligned_joints += usize::from(even_joint && odd_joint);
        }
        assert!(even_joints > 35, "even-course joint samples: {even_joints}");
        assert!(odd_joints > 35, "odd-course joint samples: {odd_joints}");
        assert!(
            aligned_joints < 8,
            "staggered courses share {aligned_joints} joint samples"
        );
    }

    #[test]
    fn nominal_dimensions_match_handmade_early_modern_masonry_scale() {
        let course_metres = HANDMADE_BRICK_TILE_METRES / COURSES as f32;
        let brick_length_metres =
            HANDMADE_BRICK_TILE_METRES / BRICKS_PER_COURSE as f32 - VERTICAL_MORTAR_METRES;
        let brick_height_metres = course_metres - HORIZONTAL_MORTAR_METRES;
        assert!((0.21..=0.24).contains(&brick_length_metres));
        assert!((0.06..=0.07).contains(&brick_height_metres));
        assert!((0.010..=0.016).contains(&VERTICAL_MORTAR_METRES));
        assert!((0.010..=0.016).contains(&HORIZONTAL_MORTAR_METRES));
    }

    #[test]
    fn repeat_area_is_large_and_chipped_edges_remain_a_minority() {
        let params = &crate::TextureParameters::default();
        let ids = (0..COURSES)
            .flat_map(|row| (0..BRICKS_PER_COURSE).map(move |column| brick_id(params, row, column)))
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), (COURSES * BRICKS_PER_COURSE) as usize);
        assert!(ids.len() >= 300);

        let edge_salts = [0x5db7, 0xa251, 0x71c3, 0xce29];
        let edge_count = ids.len() * edge_salts.len();
        let chipped_edges = ids
            .iter()
            .flat_map(|id| edge_salts.map(move |salt| hash_unit(params, *id ^ salt)))
            .filter(|selection| *selection >= 0.82)
            .count();
        let chipped_fraction = chipped_edges as f32 / edge_count as f32;
        assert!(
            (0.14..=0.22).contains(&chipped_fraction),
            "chipped edge fraction: {chipped_fraction}"
        );
    }

    #[test]
    fn face_interiors_are_near_planar_without_uniform_inflation() {
        let params = &crate::TextureParameters::default();
        let mut minimum = f32::INFINITY;
        let mut maximum = f32::NEG_INFINITY;
        let mut interior_samples = 0;
        for y in 0..256 {
            for x in 0..256 {
                let sample =
                    sample_brickwork(params, (x as f32 + 0.5) / 256.0, (y as f32 + 0.5) / 256.0);
                if sample.brick_coverage == 1.0 {
                    minimum = minimum.min(sample.height);
                    maximum = maximum.max(sample.height);
                    interior_samples += 1;
                }
            }
        }
        assert!(
            interior_samples > 25_000,
            "interior samples: {interior_samples}"
        );
        assert!(
            maximum - minimum < 0.055,
            "interior relief span: {}",
            maximum - minimum
        );
    }

    #[test]
    fn channels_are_coherent_nonmetallic_and_materially_detailed() {
        let (images, textures) = generated();
        assert_eq!(
            images
                .get(&textures.albedo)
                .unwrap()
                .texture_descriptor
                .format,
            TextureFormat::Rgba8UnormSrgb
        );
        let arm = images.get(&textures.arm).unwrap().data.as_deref().unwrap();
        let base_len = (HANDMADE_BRICK_TEXTURE_SIZE.pow(2) * 4) as usize;
        let arm_base = &arm[..base_len];
        assert!(
            arm_base
                .iter()
                .skip(2)
                .step_by(4)
                .all(|metallic| *metallic == 0)
        );
        assert!(
            arm_base
                .iter()
                .step_by(4)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                > 32
        );
        assert!(
            arm_base
                .iter()
                .skip(1)
                .step_by(4)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                == 2
        );
    }

    #[test]
    fn every_output_has_a_complete_mip_chain() {
        let (images, textures) = generated();
        let expected_levels = HANDMADE_BRICK_TEXTURE_SIZE.ilog2() + 1;
        let expected_bytes = (0..expected_levels)
            .map(|level| {
                let level_size = HANDMADE_BRICK_TEXTURE_SIZE >> level;
                (level_size * level_size * 4) as usize
            })
            .sum::<usize>();
        for handle in [
            textures.albedo,
            textures.normal_gl,
            textures.height,
            textures.arm,
        ] {
            let image = images.get(&handle).unwrap();
            assert_eq!(image.texture_descriptor.mip_level_count, expected_levels);
            assert_eq!(image.data.as_ref().unwrap().len(), expected_bytes);
        }
    }
}

mod controls;
pub use controls::Parameters;

mod surface;

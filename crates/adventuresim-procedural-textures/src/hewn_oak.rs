//! Two-color, hand-hewn structural oak with knot-deflected growth bands, fibers, and restrained adze marks.

use bevy::{asset::Assets, image::Image, math::Vec3};
use fabelgeist_determinism::inclusive_unit_f32;

use super::{SrgbColor, SurfaceTextureSet, image_rgba_mipped, palette::albedo_image};

pub(crate) mod grain;

pub const HEWN_OAK_TEXTURE_SIZE: u32 = 1024;
pub const HEWN_OAK_TILE_METRES: f32 = 2.0;
pub const HEWN_OAK_HEIGHT_RANGE_METRES: f32 = 0.009;

const ADZE_COLUMNS: i32 = 7;
const ADZE_ROWS: i32 = 10;
const CHECK_COLUMNS: i32 = 4;
const CHECK_ROWS: i32 = 5;
const LATEWOOD_RELIEF: f32 = 0.065;
const FIBER_RELIEF: f32 = 0.040;
const KNOT_RING_RELIEF: f32 = 0.035;
const VESSEL_RELIEF: f32 = 0.075;
const RAY_RELIEF: f32 = 0.025;
const ADZE_RELIEF_GAIN: f32 = 2.8;
const ADZE_BLEND_SHARPNESS: f32 = 8.0;
const ADZE_EDGE_RELIEF: f32 = 0.020;
/// Intrinsic wood colors, independent of grain relief and roughness.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct HewnOakColors {
    pub light: SrgbColor,
    pub dark: SrgbColor,
}

pub const HEWN_OAK_COLORS: HewnOakColors = HewnOakColors {
    light: SrgbColor([78, 51, 30]),
    dark: SrgbColor([65, 42, 25]),
};
const OAK_ROUGHNESS: u8 = 202;
const RELIEF_AO_STRENGTH: f32 = 3.0;

#[derive(Clone, Copy, Debug)]
struct HewnOakSample {
    height: f32,
    dark_wood: f32,
    check: f32,
    knot: f32,
    tool_recess: f32,
}

fn hash_unit(params: &crate::TextureParameters, value: u64) -> f32 {
    inclusive_unit_f32(crate::parameters::seeded_hash(params, value))
}

fn smooth(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn periodic_delta(value: f32) -> f32 {
    value - value.round()
}

fn grid_hash(
    params: &crate::TextureParameters,
    x: i32,
    y: i32,
    cells_x: i32,
    cells_y: i32,
    salt: u64,
) -> f32 {
    let x = x.rem_euclid(cells_x) as u64;
    let y = y.rem_euclid(cells_y) as u64;
    hash_unit(params, salt ^ (x << 32) ^ y)
}

fn value_noise(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
    cells_x: i32,
    cells_y: i32,
    salt: u64,
) -> f32 {
    let x = u.rem_euclid(1.0) * cells_x as f32;
    let y = v.rem_euclid(1.0) * cells_y as f32;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = smooth(x.fract());
    let ty = smooth(y.fract());
    let bottom = grid_hash(params, x0, y0, cells_x, cells_y, salt)
        + (grid_hash(params, x0 + 1, y0, cells_x, cells_y, salt)
            - grid_hash(params, x0, y0, cells_x, cells_y, salt))
            * tx;
    let top = grid_hash(params, x0, y0 + 1, cells_x, cells_y, salt)
        + (grid_hash(params, x0 + 1, y0 + 1, cells_x, cells_y, salt)
            - grid_hash(params, x0, y0 + 1, cells_x, cells_y, salt))
            * tx;
    bottom + (top - bottom) * ty
}

fn adze_relief(params: &crate::TextureParameters, u: f32, v: f32) -> (f32, f32) {
    let column = (u * params.hewn_oak.adze_columns as f32).floor() as i32;
    let row = (v * params.hewn_oak.adze_rows as f32).floor() as i32;
    let mut weight_sum = 0.0_f32;
    let mut plane_sum = 0.0_f32;
    let mut strongest_weight = 0.0_f32;
    for cell_y in (row - 2)..=(row + 2) {
        for cell_x in (column - 2)..=(column + 2) {
            let id = crate::parameters::seeded_hash(
                params,
                0x9b71_d453
                    ^ ((cell_x.rem_euclid(params.hewn_oak.adze_columns) as u64) << 32)
                    ^ cell_y.rem_euclid(params.hewn_oak.adze_rows) as u64,
            );
            let center_u = (cell_x as f32
                + params.hewn_oak.center_u_margin
                + hash_unit(params, id ^ 0xb591) * params.hewn_oak.center_u_jitter)
                / params.hewn_oak.adze_columns as f32;
            let center_v = (cell_y as f32
                + params.hewn_oak.center_v_margin
                + hash_unit(params, id ^ 0x40fd) * params.hewn_oak.center_v_jitter)
                / params.hewn_oak.adze_rows as f32;
            let angle = (hash_unit(params, id ^ 0x8851) - 0.5) * params.hewn_oak.angle_spread;
            let (sin, cos) = angle.sin_cos();
            let dx = periodic_delta(u - center_u);
            let dy = periodic_delta(v - center_v);
            let local_x = dx * cos + dy * sin;
            let local_y = -dx * sin + dy * cos;
            let cell_width = 1.0 / params.hewn_oak.adze_columns as f32;
            let cell_length = 1.0 / params.hewn_oak.adze_rows as f32;
            let distance = (local_x / cell_width).powi(2)
                * (params.hewn_oak.cross_facet_weight
                    + hash_unit(params, id ^ 0x23ab) * params.hewn_oak.cross_facet_variation)
                + (local_y / cell_length).powi(2)
                    * (params.hewn_oak.long_facet_weight
                        + hash_unit(params, id ^ 0xf127) * params.hewn_oak.long_facet_variation);
            let slope_x = (hash_unit(params, id ^ 0x6c89) - 0.5) * params.hewn_oak.cross_slope;
            let slope_y =
                (hash_unit(params, id ^ 0xe459) - 0.5) * params.hewn_oak.longitudinal_slope;
            let offset =
                (hash_unit(params, id ^ 0x917d) - 0.5) * params.hewn_oak.facet_height_variation;
            let plane = offset + local_x / cell_width * slope_x + local_y / cell_length * slope_y;
            // Blend every nearby facet continuously. Selecting just the two
            // nearest planes jumps when the second and third exchange order.
            let weight = (-distance * params.hewn_oak.adze_blend_sharpness).exp();
            weight_sum += weight;
            plane_sum += plane * weight;
            strongest_weight = strongest_weight.max(weight);
        }
    }
    let facet = plane_sum / weight_sum;
    let tool_edge = (1.0 - strongest_weight / weight_sum) * params.hewn_oak.adze_edge_relief;
    (facet, tool_edge)
}

fn check_field(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    let column = (u * params.hewn_oak.check_columns as f32).floor() as i32;
    let row = (v * params.hewn_oak.check_rows as f32).floor() as i32;
    let mut check = 0.0_f32;
    for cell_y in (row - 1)..=(row + 1) {
        for cell_x in (column - 1)..=(column + 1) {
            let id = crate::parameters::seeded_hash(
                params,
                0x4c2e_78a1
                    ^ ((cell_x.rem_euclid(params.hewn_oak.check_columns) as u64) << 32)
                    ^ cell_y.rem_euclid(params.hewn_oak.check_rows) as u64,
            );
            if hash_unit(params, id ^ 0x2ab7) < params.hewn_oak.check_absence_probability {
                continue;
            }
            let center_u = (cell_x as f32
                + params.hewn_oak.check_field_center_u_1
                + hash_unit(params, id ^ 0xa447) * params.hewn_oak.check_field_center_u_2)
                / params.hewn_oak.check_columns as f32;
            let center_v = (cell_y as f32
                + params.hewn_oak.check_field_center_v_1
                + hash_unit(params, id ^ 0x77d3) * params.hewn_oak.check_field_center_v_2)
                / params.hewn_oak.check_rows as f32;
            let dx = periodic_delta(u - center_u);
            let dy = periodic_delta(v - center_v);
            let bend = (dy * params.hewn_oak.check_bend_frequency
                + hash_unit(params, id ^ 0xcb31) * params.hewn_oak.check_field_bend)
                .sin()
                * params.hewn_oak.check_bend_amplitude;
            let half_width = params.hewn_oak.check_half_width
                + hash_unit(params, id ^ 0x5167) * params.hewn_oak.check_half_width;
            let half_length = params.hewn_oak.check_half_length
                + hash_unit(params, id ^ 0xea45) * params.hewn_oak.check_length_variation;
            let across = ((dx - bend) / half_width).abs();
            let along = (dy / half_length).abs();
            let profile =
                smooth((1.0 - across).clamp(0.0, 1.0)) * smooth((1.0 - along).clamp(0.0, 1.0));
            check = check.max(profile);
        }
    }
    check
}

fn sample_hewn_oak(params: &crate::TextureParameters, u: f32, v: f32) -> HewnOakSample {
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let growth = grain::filtered_sample(params, u, v);
    let broad_growth = value_noise(params, u, v, 3, 5, 0x81c7) - 0.5;
    let timber_variation = value_noise(params, u, v, 6, 5, 0xd24f) - 0.5;
    let (adze_cut, adze_shoulder) = adze_relief(params, u, v);
    let check = check_field(params, u, v) * params.hewn_oak.check_strength;
    let knot = growth.knot * params.hewn_oak.knot_strength;
    let tool_recess = ((-adze_cut - params.hewn_oak.tool_recess_threshold)
        / params.hewn_oak.tool_recess_range)
        .clamp(0.0, 1.0);
    let height = (params.hewn_oak.base_height
        + broad_growth * params.hewn_oak.broad_growth_relief
        + growth.latewood * params.hewn_oak.latewood_relief
        - growth.fibers * params.hewn_oak.fiber_relief
        + growth.knot_rings * params.hewn_oak.knot_ring_relief
        + timber_variation * params.hewn_oak.timber_relief
        - growth.vessels * params.hewn_oak.vessel_relief
        + growth.rays * params.hewn_oak.ray_relief
        + adze_cut * params.hewn_oak.adze_relief_gain
        + adze_shoulder * params.hewn_oak.adze_relief_gain
        - check * params.hewn_oak.check_depth
        - knot * params.hewn_oak.knot_depth)
        .clamp(0.0, 1.0);
    HewnOakSample {
        height,
        dark_wood: growth.dark_wood,
        check,
        knot,
        tool_recess,
    }
}

fn height_at(params: &crate::TextureParameters, heights: &[f32], x: i32, y: i32) -> f32 {
    let size = params.size(HEWN_OAK_TEXTURE_SIZE) as i32;
    heights[(y.rem_euclid(size) * size + x.rem_euclid(size)) as usize]
}

fn ambient_visibility(params: &crate::TextureParameters, heights: &[f32], x: i32, y: i32) -> f32 {
    let center = height_at(params, heights, x, y);
    let mut obstruction = 0.0;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for step in [1, 3, 7] {
            obstruction += ((height_at(params, heights, x + dx * step, y + dy * step) - center)
                / step as f32)
                .max(0.0);
        }
    }
    (1.0 - obstruction * params.hewn_oak.relief_ao_strength)
        .clamp(params.hewn_oak.minimum_ambient_visibility, 1.0)
}

pub fn generate_hewn_oak_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> SurfaceTextureSet {
    let colors = &params.hewn_oak.colors;
    let size = params.size(HEWN_OAK_TEXTURE_SIZE);
    let samples = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                sample_hewn_oak(
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
    let capacity = (size * size * 4) as usize;
    let mut albedo = Vec::with_capacity(capacity);
    let mut normal = Vec::with_capacity(capacity);
    let mut height = Vec::with_capacity(capacity);
    let mut arm = Vec::with_capacity(capacity);
    let metres_per_texel = params.hewn_oak.tile_metres / size as f32;
    let slope_scale = params.hewn_oak.height_range_metres / (2.0 * metres_per_texel);

    for y in 0..size {
        for x in 0..size {
            let sample = samples[(y * size + x) as usize];
            let color = colors.light.covered_by(colors.dark, sample.dark_wood).0;
            albedo.extend_from_slice(&[color[0], color[1], color[2], 255]);
            let dx = height_at(params, &heights, x as i32 + 1, y as i32)
                - height_at(params, &heights, x as i32 - 1, y as i32);
            let dy = height_at(params, &heights, x as i32, y as i32 + 1)
                - height_at(params, &heights, x as i32, y as i32 - 1);
            let surface_normal = Vec3::new(-dx * slope_scale, -dy * slope_scale, 1.0).normalize();
            let encoded_normal = ((surface_normal + Vec3::ONE) * 127.5)
                .round()
                .clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal.extend_from_slice(&[
                encoded_normal.x as u8,
                encoded_normal.y as u8,
                encoded_normal.z as u8,
                255,
            ]);
            let encoded_height = (sample.height * 255.0).round() as u8;
            height.extend_from_slice(&[encoded_height, encoded_height, encoded_height, 255]);
            let ambient_visibility = (ambient_visibility(params, &heights, x as i32, y as i32)
                - sample.check * params.hewn_oak.check_occlusion
                - sample.knot * params.hewn_oak.knot_occlusion
                - sample.tool_recess * params.hewn_oak.tool_occlusion)
                .clamp(params.hewn_oak.ambient_floor, 1.0);
            let ao = (ambient_visibility * 255.0).round() as u8;
            let roughness = params.hewn_oak.oak_roughness;
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
        let textures = generate_hewn_oak_textures(params, &mut images);
        (images, textures)
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
        let epsilon = 0.1 / HEWN_OAK_TEXTURE_SIZE as f32;
        let mut maximum_error = 0.0_f32;
        for index in 0..1_024 {
            let coordinate = (index as f32 + 0.5) / 1_024.0;
            maximum_error = maximum_error
                .max(
                    (sample_hewn_oak(params, epsilon, coordinate).height
                        - sample_hewn_oak(params, 1.0 - epsilon, coordinate).height)
                        .abs(),
                )
                .max(
                    (sample_hewn_oak(params, coordinate, epsilon).height
                        - sample_hewn_oak(params, coordinate, 1.0 - epsilon).height)
                        .abs(),
                );
        }
        assert!(maximum_error < 0.025, "seam height error: {maximum_error}");
    }

    #[test]
    fn marks_have_plausible_physical_scale_and_restrained_frequency() {
        let params = &crate::TextureParameters::default();
        let adze_width_metres = HEWN_OAK_TILE_METRES / ADZE_COLUMNS as f32;
        let adze_length_metres = HEWN_OAK_TILE_METRES / ADZE_ROWS as f32;
        assert!((0.27..=0.30).contains(&adze_width_metres));
        assert!((0.19..=0.21).contains(&adze_length_metres));

        let mut checks = 0;
        let mut knots = 0;
        for y in 0..256 {
            for x in 0..256 {
                let sample =
                    sample_hewn_oak(params, (x as f32 + 0.5) / 256.0, (y as f32 + 0.5) / 256.0);
                checks += usize::from(sample.check > 0.10);
                knots += usize::from(sample.knot > 0.055);
            }
        }
        assert!((10..=900).contains(&checks), "check pixels: {checks}");
        assert!((20..=500).contains(&knots), "knot pixels: {knots}");
    }

    #[test]
    fn sculptural_detail_does_not_leak_into_color_or_roughness() {
        let (images, textures) = generated();
        assert_eq!(
            images
                .get(&textures.albedo)
                .unwrap()
                .texture_descriptor
                .format,
            TextureFormat::Rgba8UnormSrgb
        );
        let base_len = (HEWN_OAK_TEXTURE_SIZE.pow(2) * 4) as usize;
        let albedo = &images
            .get(&textures.albedo)
            .unwrap()
            .data
            .as_deref()
            .unwrap()[..base_len];
        let arm = &images.get(&textures.arm).unwrap().data.as_deref().unwrap()[..base_len];
        assert!(
            albedo
                .iter()
                .step_by(4)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                > 2
        );
        assert!(
            arm.iter()
                .step_by(4)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                > 6
        );
        assert!(
            arm.iter()
                .skip(1)
                .step_by(4)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                == 1
        );
        let allowed = (0..=16)
            .map(|step| {
                HEWN_OAK_COLORS
                    .light
                    .covered_by(HEWN_OAK_COLORS.dark, step as f32 / 16.0)
                    .0
            })
            .collect::<BTreeSet<_>>();
        assert!(
            albedo
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| allowed.contains(&[pixel[0], pixel[1], pixel[2]]))
        );
        assert!(arm.iter().skip(2).step_by(4).all(|metallic| *metallic == 0));
    }

    #[test]
    fn every_output_has_a_complete_mip_chain() {
        let (images, textures) = generated();
        let expected_levels = HEWN_OAK_TEXTURE_SIZE.ilog2() + 1;
        let expected_bytes = (0..expected_levels)
            .map(|level| {
                let level_size = HEWN_OAK_TEXTURE_SIZE >> level;
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

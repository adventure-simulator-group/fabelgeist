//! Regular early-modern ashlar with chipped, beveled edges, sparse cavities, and granular lime joints.
//!
//! Original design evidence:
//! prior-art/dressed-stone/prior-art.md.
//! That report records a research snapshot; this module and its tests define
//! current behavior.

use bevy::{asset::Assets, image::Image, math::Vec3};
use fabelgeist_determinism::inclusive_unit_f32;

use super::{
    MasonryColors, SrgbColor, SurfaceTextureSet, image_rgba_mipped, palette::albedo_image,
};

pub(crate) mod weathering;

pub const DRESSED_STONE_TEXTURE_SIZE: u32 = 2048;
pub const DRESSED_STONE_TILE_METRES: f32 = 7.2;
pub const DRESSED_STONE_HEIGHT_RANGE_METRES: f32 = 0.024;

const COURSES: i32 = 22;
const BLOCK_CAPACITY: usize = 16;
const MAX_BLOCKS_PER_COURSE: usize = 15;
const MIN_BLOCKS_PER_COURSE: usize = 11;
const BEVEL_RELIEF: f32 = 0.17;
const PORE_RELIEF: f32 = 0.055;
const GRAIN_RELIEF: f32 = 0.008;
const SPALL_RELIEF: f32 = 0.11;
const FACE_RELIEF: f32 = 0.022;
const BLOCK_HEIGHT_VARIATION: f32 = 0.065;
pub const DRESSED_STONE_COLORS: MasonryColors<6> = MasonryColors {
    units: [
        SrgbColor([128, 126, 113]),
        SrgbColor([130, 127, 112]),
        SrgbColor([126, 124, 111]),
        SrgbColor([132, 129, 115]),
        SrgbColor([125, 123, 110]),
        SrgbColor([129, 126, 114]),
    ],
    mortar: SrgbColor([143, 139, 125]),
};
const STONE_ROUGHNESS_PALETTE: [u8; 3] = [218, 222, 226];
const MORTAR_ROUGHNESS: u8 = 236;

#[derive(Clone, Copy, Debug)]
struct StoneSample {
    height: f32,
    stone_coverage: f32,
    stone_id: u64,
    edge_distance: f32,
}

fn hash_unit(params: &crate::TextureParameters, value: u64) -> f32 {
    inclusive_unit_f32(crate::parameters::seeded_hash(params, value))
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn course_id(params: &crate::TextureParameters, course: i32) -> u64 {
    crate::parameters::seeded_hash(
        params,
        0x453a_91d7 ^ course.rem_euclid(params.dressed_stone.courses) as u64,
    )
}

fn block_count(params: &crate::TextureParameters, course: i32) -> usize {
    params.dressed_stone.min_blocks_per_course
        + course_id(params, course) as usize
            % (params.dressed_stone.max_blocks_per_course
                - params.dressed_stone.min_blocks_per_course
                + 1)
}

fn block_id(params: &crate::TextureParameters, course: i32, block: usize) -> u64 {
    crate::parameters::seeded_hash(
        params,
        course_id(params, course) ^ (block as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
    )
}

fn course_weights(params: &crate::TextureParameters) -> ([f32; COURSES as usize], f32) {
    let mut weights = [0.0; COURSES as usize];
    let mut total = 0.0;
    for course in 0..params.dressed_stone.courses {
        let weight = params.dressed_stone.course_weights_weight_1
            + hash_unit(params, course_id(params, course) ^ 0x157d)
                * params.dressed_stone.course_weights_weight_2;
        weights[course as usize] = weight;
        total += weight;
    }
    (weights, total)
}

fn course_at(params: &crate::TextureParameters, v: f32) -> (i32, f32, f32) {
    let (weights, total) = course_weights(params);
    let position = v.rem_euclid(1.0) * total;
    let mut start = 0.0;
    for course in 0..params.dressed_stone.courses {
        let weight = weights[course as usize];
        if position < start + weight || course + 1 == params.dressed_stone.courses {
            return (course, (position - start) / weight, weight / total);
        }
        start += weight;
    }
    unreachable!()
}

fn block_weights(params: &crate::TextureParameters, course: i32) -> ([f32; BLOCK_CAPACITY], f32) {
    let mut weights = [0.0; BLOCK_CAPACITY];
    let mut total = 0.0;
    for (block, weight) in weights
        .iter_mut()
        .take(block_count(params, course))
        .enumerate()
    {
        let id = block_id(params, course, block);
        *weight = 0.79 + hash_unit(params, id ^ 0x6c81) * 0.42;
        total += *weight;
    }
    (weights, total)
}

fn course_offset(params: &crate::TextureParameters, course: i32) -> f32 {
    // A broad deterministic offset gives a convincing bond without a machine-perfect half bond.
    let alternating = if course.rem_euclid(2) == 0 {
        0.0
    } else {
        params.dressed_stone.course_offset_alternating
    };
    (alternating / block_count(params, course) as f32
        + (hash_unit(params, course_id(params, course) ^ 0xc319) - 0.5) * 0.035)
        .rem_euclid(1.0)
}

fn block_at(params: &crate::TextureParameters, course: i32, u: f32) -> (usize, f32, f32) {
    let count = block_count(params, course);
    let (weights, total) = block_weights(params, course);
    let position = (u + course_offset(params, course)).rem_euclid(1.0) * total;
    let mut start = 0.0;
    for (block, weight) in weights.into_iter().take(count).enumerate() {
        if position < start + weight || block + 1 == count {
            return (block, (position - start) / weight, weight / total);
        }
        start += weight;
    }
    unreachable!()
}

fn periodic_wave(params: &crate::TextureParameters, coordinate: f32, id: u64, salt: u64) -> f32 {
    let phase = hash_unit(params, id ^ salt) * std::f32::consts::TAU;
    let phase_two = hash_unit(params, id ^ salt.rotate_left(17)) * std::f32::consts::TAU;
    (coordinate * std::f32::consts::TAU + phase).sin() * 0.54
        + (coordinate * std::f32::consts::TAU * 2.0 + phase_two).sin() * 0.18
}

fn tool_marks(params: &crate::TextureParameters, local_x: f32, local_y: f32, id: u64) -> f32 {
    if hash_unit(params, id ^ 0x49b5) < 0.56 {
        return 0.0;
    }
    let mark_count = 2 + (crate::parameters::seeded_hash(params, id ^ 0x2b8f) % 4) as usize;
    let base_angle = -params.dressed_stone.tool_marks_base_angle_1
        + hash_unit(params, id ^ 0x861d) * params.dressed_stone.tool_marks_base_angle_2;
    let mut relief = 0.0_f32;
    for mark in 0..mark_count {
        let mark_id =
            crate::parameters::seeded_hash(params, id ^ (mark as u64).wrapping_mul(0x9e37_79b9));
        let center_x = hash_unit(params, mark_id ^ 0x13c7).mul_add(
            params.dressed_stone.tool_marks_center_x_1,
            -params.dressed_stone.tool_marks_center_x_2,
        );
        let center_y = hash_unit(params, mark_id ^ 0xb15d).mul_add(
            params.dressed_stone.tool_marks_center_y_1,
            -params.dressed_stone.tool_marks_center_y_2,
        );
        let angle = base_angle
            + (hash_unit(params, mark_id ^ 0xd371) - 0.5) * params.dressed_stone.tool_marks_angle;
        let dx = local_x - center_x;
        let dy = local_y - center_y;
        let along = dx * angle.cos() + dy * angle.sin();
        let mut across = -dx * angle.sin() + dy * angle.cos();
        let half_length = params.dressed_stone.tool_marks_half_length_1
            + hash_unit(params, mark_id ^ 0xa275) * params.dressed_stone.tool_marks_half_length_2;
        across += (along * along - half_length * half_length * 0.33)
            * (hash_unit(params, mark_id ^ 0xe695) - 0.5)
            * 0.62;
        let width = params.dressed_stone.tool_marks_width_1
            + hash_unit(params, mark_id ^ 0x7f29) * params.dressed_stone.tool_marks_width_2;
        let taper = smoothstep(
            0.0,
            params.dressed_stone.tool_marks_taper,
            1.0 - along.abs() / half_length,
        );
        let gap_center = (hash_unit(params, mark_id ^ 0x5d91) - 0.5) * half_length;
        let gap_radius = half_length
            * (params.dressed_stone.tool_marks_gap_radius_1
                + hash_unit(params, mark_id ^ 0xc583)
                    * params.dressed_stone.tool_marks_gap_radius_2);
        let interruption = smoothstep(
            0.0,
            gap_radius,
            (along - gap_center).abs() - gap_radius * params.dressed_stone.tool_marks_interruption,
        );
        let groove = (1.0 - across.abs() / width).max(0.0);
        relief -= groove
            * groove
            * taper
            * interruption
            * (0.004 + hash_unit(params, mark_id ^ 0x4cb7) * 0.005);
    }
    relief
}

fn sample_stonework(params: &crate::TextureParameters, u: f32, v: f32) -> StoneSample {
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let (course, local_y, course_height) = course_at(params, v);
    let (block, local_x, block_width) = block_at(params, course, u);
    let id = block_id(params, course, block);
    let x = local_x * 2.0 - 1.0;
    let y = local_y * 2.0 - 1.0;

    let bed_joint = (params.dressed_stone.sample_stonework_bed_joint_1
        + hash_unit(params, course_id(params, course) ^ 0x9751)
            * params.dressed_stone.sample_stonework_bed_joint_2)
        / params.dressed_stone.tile_metres
        / course_height
        * 2.0;
    let head_joint = (params.dressed_stone.sample_stonework_head_joint_1
        + hash_unit(params, id ^ 0x61df) * params.dressed_stone.sample_stonework_head_joint_2)
        / params.dressed_stone.tile_metres
        / block_width
        * 2.0;
    let horizontal_wobble = periodic_wave(params, local_x, id, 0xb62d)
        * params.dressed_stone.sample_stonework_horizontal_wobble;
    let vertical_wobble = periodic_wave(params, local_y, id, 0x297b)
        * params.dressed_stone.sample_stonework_vertical_wobble;
    let left = -1.0 + head_joint + vertical_wobble;
    let right = 1.0 - head_joint + vertical_wobble;
    let bottom = -1.0 + bed_joint + horizontal_wobble;
    let top = 1.0 - bed_joint + horizontal_wobble;
    let half_size = [
        block_width * params.dressed_stone.tile_metres * 0.5,
        course_height * params.dressed_stone.tile_metres * 0.5,
    ];
    let edge = weathering::edge_profile(
        params,
        [x, y],
        half_size,
        [
            (left - x) * half_size[0],
            (x - right) * half_size[0],
            (bottom - y) * half_size[1],
            (y - top) * half_size[1],
        ],
        id,
    );
    let edge_distance = edge.distance / half_size[0].min(half_size[1]);
    let antialias = params.dressed_stone.tile_metres
        / params.size(DRESSED_STONE_TEXTURE_SIZE) as f32
        * params.dressed_stone.sample_stonework_antialias;
    let stone_coverage = ((antialias - edge.distance) / (antialias * 2.0)).clamp(0.0, 1.0);
    let detail = weathering::face_detail(params, x * half_size[0], y * half_size[1], id);
    let mortar = weathering::mortar_detail(
        params,
        u * params.dressed_stone.tile_metres,
        v * params.dressed_stone.tile_metres,
        edge.distance,
    );

    let planar_tilt = x
        * (hash_unit(params, id ^ 0x158d) - 0.5)
        * params.dressed_stone.sample_stonework_planar_tilt_1
        + y * (hash_unit(params, id ^ 0xb4e7) - 0.5)
            * params.dressed_stone.sample_stonework_planar_tilt_2;
    let broad = periodic_wave(params, x * 0.5 + 0.5, id, 0xc7a9)
        * params.dressed_stone.sample_stonework_broad;
    let tools = tool_marks(params, x, y, id);
    let face_height = params.dressed_stone.sample_stonework_face_height
        + (hash_unit(params, id ^ 0x53f1) - 0.5) * params.dressed_stone.block_height_variation
        + planar_tilt
        + broad
        + tools
        - edge.bevel * params.dressed_stone.bevel_relief
        - detail.pore * params.dressed_stone.pore_relief
        + detail.grain * params.dressed_stone.grain_relief
        + detail.mineral * params.dressed_stone.face_relief
        - edge.exposed_chip * params.dressed_stone.spall_relief;

    StoneSample {
        height: mortar.height + (face_height - mortar.height) * stone_coverage,
        stone_coverage,
        stone_id: id,
        edge_distance,
    }
}

fn stone_color(
    params: &crate::TextureParameters,
    sample: StoneSample,
    colors: &MasonryColors<6>,
) -> ([u8; 3], u8) {
    let unit = colors.units[sample.stone_id as usize % colors.units.len()];
    let color = colors.mortar.covered_by(unit, sample.stone_coverage).0;
    let roughness = if sample.edge_distance > 0.0 {
        params.dressed_stone.mortar_roughness
    } else {
        params.dressed_stone.stone_roughness_palette
            [sample.stone_id as usize % params.dressed_stone.stone_roughness_palette.len()]
    };
    (color, roughness)
}

fn height_at(params: &crate::TextureParameters, heights: &[f32], x: i32, y: i32) -> f32 {
    let size = params.size(DRESSED_STONE_TEXTURE_SIZE) as i32;
    heights[(y.rem_euclid(size) * size + x.rem_euclid(size)) as usize]
}

fn ambient_visibility(params: &crate::TextureParameters, heights: &[f32], x: i32, y: i32) -> f32 {
    let center = height_at(params, heights, x, y);
    let mut obstruction = 0.0;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for step in [1, 3, 7, 15] {
            obstruction += ((height_at(params, heights, x + dx * step, y + dy * step) - center)
                / step as f32)
                .max(0.0);
        }
    }
    (1.0 - obstruction * 2.8).clamp(0.48, 1.0)
}

pub fn generate_dressed_stone_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> SurfaceTextureSet {
    let colors = &params.dressed_stone.colors;
    let size = params.size(DRESSED_STONE_TEXTURE_SIZE);
    let samples = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                sample_stonework(
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
    let metres_per_texel = params.dressed_stone.tile_metres / size as f32;
    let slope_scale = params.dressed_stone.height_range_metres / (2.0 * metres_per_texel);

    for y in 0..size {
        for x in 0..size {
            let sample = samples[(y * size + x) as usize];
            let (color, roughness) = stone_color(params, sample, colors);
            albedo.extend_from_slice(&[color[0], color[1], color[2], 255]);
            let dx = height_at(params, &heights, x as i32 + 1, y as i32)
                - height_at(params, &heights, x as i32 - 1, y as i32);
            let dy = height_at(params, &heights, x as i32, y as i32 + 1)
                - height_at(params, &heights, x as i32, y as i32 - 1);
            let n = crate::normal::from_image_gradient(dx * slope_scale, dy * slope_scale);
            let encoded = ((n + Vec3::ONE) * 127.5)
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
    use bevy::render::render_resource::TextureFormat;
    use std::collections::BTreeSet;

    use super::*;

    fn generated() -> (Assets<Image>, SurfaceTextureSet) {
        let params = &crate::TextureParameters::default();
        let mut images = Assets::default();
        let textures = generate_dressed_stone_textures(params, &mut images);
        (images, textures)
    }

    #[test]
    fn unit_and_mortar_colors_are_independent_with_antialiased_contacts() {
        let params = &crate::TextureParameters::default();
        let colors = DRESSED_STONE_COLORS;
        let mut recolored_units = colors;
        recolored_units.units.fill(SrgbColor([40, 70, 100]));
        let mut recolored_mortar = colors;
        recolored_mortar.mortar = SrgbColor([210, 190, 160]);
        let mut regions = [false; 3];
        for y in 0..128 {
            for x in 0..128 {
                let sample =
                    sample_stonework(params, (x as f32 + 0.5) / 128.0, (y as f32 + 0.5) / 128.0);
                let original = stone_color(params, sample, &colors);
                let units = stone_color(params, sample, &recolored_units);
                let mortar = stone_color(params, sample, &recolored_mortar);
                assert_eq!(original.1, units.1);
                assert_eq!(original.1, mortar.1);
                if sample.stone_coverage == 0.0 {
                    regions[0] = true;
                    assert_eq!(units, original);
                    assert_eq!(mortar.0, recolored_mortar.mortar.0);
                } else if sample.stone_coverage == 1.0 {
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
        let (a_images, a) = generated();
        let (b_images, b) = generated();
        for (a_handle, b_handle) in [
            (&a.albedo, &b.albedo),
            (&a.normal_gl, &b.normal_gl),
            (&a.height, &b.height),
            (&a.arm, &b.arm),
        ] {
            assert_eq!(
                a_images.get(a_handle).unwrap().data,
                b_images.get(b_handle).unwrap().data
            );
        }
    }

    #[test]
    fn analytic_field_tiles_continuously() {
        let params = &crate::TextureParameters::default();
        let mut maximum_error = 0.0_f32;
        for index in 0..512 {
            let coordinate = (index as f32 + 0.5) / 512.0;
            let sample = sample_stonework(params, coordinate, 0.37).height;
            maximum_error = maximum_error
                .max((sample - sample_stonework(params, coordinate + 1.0, 0.37).height).abs())
                .max((sample - sample_stonework(params, coordinate, 1.37).height).abs());
        }
        assert!(
            maximum_error < f32::EPSILON,
            "periodic height error: {maximum_error}"
        );
    }

    #[test]
    fn ashlar_has_declared_period_appropriate_scale() {
        let params = &crate::TextureParameters::default();
        assert_eq!(DRESSED_STONE_TILE_METRES, 7.2);
        assert!((0.020..=0.028).contains(&DRESSED_STONE_HEIGHT_RANGE_METRES));
        let (courses, total_height) = course_weights(params);
        for weight in courses {
            let metres = weight / total_height * DRESSED_STONE_TILE_METRES;
            assert!((0.27..=0.37).contains(&metres), "course height: {metres}");
        }
        for course in 0..COURSES {
            let (weights, total) = block_weights(params, course);
            for weight in weights.into_iter().take(block_count(params, course)) {
                let metres = weight / total * DRESSED_STONE_TILE_METRES;
                assert!((0.32..=0.90).contains(&metres), "block width: {metres}");
            }
        }
    }

    #[test]
    fn joints_are_recessed_while_faces_remain_planar() {
        let params = &crate::TextureParameters::default();
        let mut joints = Vec::new();
        let mut faces = Vec::new();
        for y in 0..256 {
            for x in 0..256 {
                let sample =
                    sample_stonework(params, (x as f32 + 0.5) / 256.0, (y as f32 + 0.5) / 256.0);
                if sample.edge_distance > 0.02 {
                    joints.push(sample.height);
                } else if sample.edge_distance < -0.30 {
                    faces.push(sample.height);
                }
            }
        }
        let joint_max = joints.into_iter().fold(f32::NEG_INFINITY, f32::max);
        let face_min = faces.iter().copied().fold(f32::INFINITY, f32::min);
        let face_span = faces.iter().copied().fold(f32::NEG_INFINITY, f32::max) - face_min;
        assert!(face_min - joint_max > 0.35);
        assert!(face_span < 0.17, "planar face span: {face_span}");
    }

    #[test]
    fn intrinsic_palette_is_independent_of_relief_and_damage() {
        let params = &crate::TextureParameters::default();
        for y in 0..128 {
            for x in 0..128 {
                let sample = sample_stonework(params, x as f32 / 128.0, y as f32 / 128.0);
                let (color, roughness) = stone_color(params, sample, &DRESSED_STONE_COLORS);
                if sample.stone_coverage == 0.0 {
                    assert_eq!(color, DRESSED_STONE_COLORS.mortar.0);
                }
                if sample.stone_coverage == 1.0 {
                    assert!(DRESSED_STONE_COLORS.units.contains(&SrgbColor(color)));
                }
                assert!(
                    STONE_ROUGHNESS_PALETTE.contains(&roughness) || roughness == MORTAR_ROUGHNESS
                );
                assert_eq!(
                    stone_color(
                        params,
                        StoneSample {
                            height: 0.0,
                            ..sample
                        },
                        &DRESSED_STONE_COLORS
                    ),
                    (color, roughness)
                );
            }
        }
    }

    #[test]
    fn channels_are_complete_nonmetallic_and_mipped() {
        let (images, textures) = generated();
        let expected_levels = DRESSED_STONE_TEXTURE_SIZE.ilog2() + 1;
        let expected_bytes = (0..expected_levels)
            .map(|level| {
                let size = DRESSED_STONE_TEXTURE_SIZE >> level;
                (size * size * 4) as usize
            })
            .sum::<usize>();
        for handle in [
            &textures.albedo,
            &textures.normal_gl,
            &textures.height,
            &textures.arm,
        ] {
            let image = images.get(handle).unwrap();
            assert_eq!(image.texture_descriptor.mip_level_count, expected_levels);
            assert_eq!(image.data.as_ref().unwrap().len(), expected_bytes);
        }
        assert_eq!(
            images
                .get(&textures.albedo)
                .unwrap()
                .texture_descriptor
                .format,
            TextureFormat::Rgba8UnormSrgb
        );
        let arm = images.get(&textures.arm).unwrap().data.as_deref().unwrap();
        let base = &arm[..(DRESSED_STONE_TEXTURE_SIZE.pow(2) * 4) as usize];
        assert!(base.iter().skip(2).step_by(4).all(|value| *value == 0));
        assert!(
            base.iter()
                .step_by(4)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                > 16
        );
        assert!(
            base.iter()
                .skip(1)
                .step_by(4)
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                == 4
        );
    }
}

mod controls;
pub use controls::Parameters;

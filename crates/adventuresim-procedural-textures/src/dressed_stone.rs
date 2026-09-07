//! Regular early-modern ashlar with chipped, beveled edges, sparse cavities, and granular lime joints.

use bevy::{asset::Assets, image::Image, math::Vec3, render::render_resource::TextureFormat};
use fabelgeist_determinism::{inclusive_unit_f32, splitmix64};

use super::{SurfaceTextureSet, image_rgba_mipped};

mod weathering;

pub const DRESSED_STONE_TEXTURE_SIZE: u32 = 2048;
pub const DRESSED_STONE_TILE_METRES: f32 = 7.2;
pub const DRESSED_STONE_HEIGHT_RANGE_METRES: f32 = 0.024;

const COURSES: i32 = 22;
const MAX_BLOCKS_PER_COURSE: usize = 16;
const MIN_BLOCKS_PER_COURSE: usize = 11;
const BEVEL_RELIEF: f32 = 0.17;
const PORE_RELIEF: f32 = 0.055;
const GRAIN_RELIEF: f32 = 0.008;
const SPALL_RELIEF: f32 = 0.11;
const FACE_RELIEF: f32 = 0.022;
const BLOCK_HEIGHT_VARIATION: f32 = 0.065;
const STONE_PALETTE: [[u8; 3]; 6] = [
    [128, 126, 113],
    [130, 127, 112],
    [126, 124, 111],
    [132, 129, 115],
    [125, 123, 110],
    [129, 126, 114],
];
const STONE_ROUGHNESS_PALETTE: [u8; 3] = [218, 222, 226];
const MORTAR_ALBEDO: [u8; 3] = [143, 139, 125];
const MORTAR_ROUGHNESS: u8 = 236;

#[derive(Clone, Copy, Debug)]
struct StoneSample {
    height: f32,
    stone_id: u64,
    edge_distance: f32,
}

fn hash_unit(value: u64) -> f32 {
    inclusive_unit_f32(splitmix64(value))
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn course_id(course: i32) -> u64 {
    splitmix64(0x453a_91d7 ^ course.rem_euclid(COURSES) as u64)
}

fn block_count(course: i32) -> usize {
    MIN_BLOCKS_PER_COURSE + course_id(course) as usize % 5
}

fn block_id(course: i32, block: usize) -> u64 {
    splitmix64(course_id(course) ^ (block as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
}

fn course_weights() -> ([f32; COURSES as usize], f32) {
    let mut weights = [0.0; COURSES as usize];
    let mut total = 0.0;
    for course in 0..COURSES {
        let weight = 0.91 + hash_unit(course_id(course) ^ 0x157d) * 0.18;
        weights[course as usize] = weight;
        total += weight;
    }
    (weights, total)
}

fn course_at(v: f32) -> (i32, f32, f32) {
    let (weights, total) = course_weights();
    let position = v.rem_euclid(1.0) * total;
    let mut start = 0.0;
    for course in 0..COURSES {
        let weight = weights[course as usize];
        if position < start + weight || course + 1 == COURSES {
            return (course, (position - start) / weight, weight / total);
        }
        start += weight;
    }
    unreachable!()
}

fn block_weights(course: i32) -> ([f32; MAX_BLOCKS_PER_COURSE], f32) {
    let mut weights = [0.0; MAX_BLOCKS_PER_COURSE];
    let mut total = 0.0;
    for (block, weight) in weights.iter_mut().take(block_count(course)).enumerate() {
        let id = block_id(course, block);
        *weight = 0.79 + hash_unit(id ^ 0x6c81) * 0.42;
        total += *weight;
    }
    (weights, total)
}

fn course_offset(course: i32) -> f32 {
    // A broad deterministic offset gives a convincing bond without a machine-perfect half bond.
    let alternating = if course.rem_euclid(2) == 0 { 0.0 } else { 0.47 };
    (alternating / block_count(course) as f32
        + (hash_unit(course_id(course) ^ 0xc319) - 0.5) * 0.035)
        .rem_euclid(1.0)
}

fn block_at(course: i32, u: f32) -> (usize, f32, f32) {
    let count = block_count(course);
    let (weights, total) = block_weights(course);
    let position = (u + course_offset(course)).rem_euclid(1.0) * total;
    let mut start = 0.0;
    for (block, weight) in weights.into_iter().take(count).enumerate() {
        if position < start + weight || block + 1 == count {
            return (block, (position - start) / weight, weight / total);
        }
        start += weight;
    }
    unreachable!()
}

fn periodic_wave(coordinate: f32, id: u64, salt: u64) -> f32 {
    let phase = hash_unit(id ^ salt) * std::f32::consts::TAU;
    let phase_two = hash_unit(id ^ salt.rotate_left(17)) * std::f32::consts::TAU;
    (coordinate * std::f32::consts::TAU + phase).sin() * 0.54
        + (coordinate * std::f32::consts::TAU * 2.0 + phase_two).sin() * 0.18
}

fn tool_marks(local_x: f32, local_y: f32, id: u64) -> f32 {
    if hash_unit(id ^ 0x49b5) < 0.56 {
        return 0.0;
    }
    let mark_count = 2 + (splitmix64(id ^ 0x2b8f) % 4) as usize;
    let base_angle = -0.30 + hash_unit(id ^ 0x861d) * 1.90;
    let mut relief = 0.0_f32;
    for mark in 0..mark_count {
        let mark_id = splitmix64(id ^ (mark as u64).wrapping_mul(0x9e37_79b9));
        let center_x = hash_unit(mark_id ^ 0x13c7).mul_add(1.30, -0.65);
        let center_y = hash_unit(mark_id ^ 0xb15d).mul_add(1.30, -0.65);
        let angle = base_angle + (hash_unit(mark_id ^ 0xd371) - 0.5) * 0.54;
        let dx = local_x - center_x;
        let dy = local_y - center_y;
        let along = dx * angle.cos() + dy * angle.sin();
        let mut across = -dx * angle.sin() + dy * angle.cos();
        let half_length = 0.11 + hash_unit(mark_id ^ 0xa275) * 0.30;
        across += (along * along - half_length * half_length * 0.33)
            * (hash_unit(mark_id ^ 0xe695) - 0.5)
            * 0.62;
        let width = 0.026 + hash_unit(mark_id ^ 0x7f29) * 0.036;
        let taper = smoothstep(0.0, 0.28, 1.0 - along.abs() / half_length);
        let gap_center = (hash_unit(mark_id ^ 0x5d91) - 0.5) * half_length;
        let gap_radius = half_length * (0.05 + hash_unit(mark_id ^ 0xc583) * 0.10);
        let interruption = smoothstep(
            0.0,
            gap_radius,
            (along - gap_center).abs() - gap_radius * 0.35,
        );
        let groove = (1.0 - across.abs() / width).max(0.0);
        relief -=
            groove * groove * taper * interruption * (0.004 + hash_unit(mark_id ^ 0x4cb7) * 0.005);
    }
    relief
}

fn sample_stonework(u: f32, v: f32) -> StoneSample {
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let (course, local_y, course_height) = course_at(v);
    let (block, local_x, block_width) = block_at(course, u);
    let id = block_id(course, block);
    let x = local_x * 2.0 - 1.0;
    let y = local_y * 2.0 - 1.0;

    let bed_joint = (0.010 + hash_unit(course_id(course) ^ 0x9751) * 0.004)
        / DRESSED_STONE_TILE_METRES
        / course_height
        * 2.0;
    let head_joint =
        (0.008 + hash_unit(id ^ 0x61df) * 0.005) / DRESSED_STONE_TILE_METRES / block_width * 2.0;
    let horizontal_wobble = periodic_wave(local_x, id, 0xb62d) * 0.008;
    let vertical_wobble = periodic_wave(local_y, id, 0x297b) * 0.010;
    let left = -1.0 + head_joint + vertical_wobble;
    let right = 1.0 - head_joint + vertical_wobble;
    let bottom = -1.0 + bed_joint + horizontal_wobble;
    let top = 1.0 - bed_joint + horizontal_wobble;
    let half_size = [
        block_width * DRESSED_STONE_TILE_METRES * 0.5,
        course_height * DRESSED_STONE_TILE_METRES * 0.5,
    ];
    let edge = weathering::edge_profile(
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
    let antialias = DRESSED_STONE_TILE_METRES / DRESSED_STONE_TEXTURE_SIZE as f32 * 0.8;
    let stone_coverage = ((antialias - edge.distance) / (antialias * 2.0)).clamp(0.0, 1.0);
    let detail = weathering::face_detail(x * half_size[0], y * half_size[1], id);
    let mortar = weathering::mortar_detail(
        u * DRESSED_STONE_TILE_METRES,
        v * DRESSED_STONE_TILE_METRES,
        edge.distance,
    );

    let planar_tilt =
        x * (hash_unit(id ^ 0x158d) - 0.5) * 0.030 + y * (hash_unit(id ^ 0xb4e7) - 0.5) * 0.022;
    let broad = periodic_wave(x * 0.5 + 0.5, id, 0xc7a9) * 0.005;
    let tools = tool_marks(x, y, id);
    let face_height = 0.71
        + (hash_unit(id ^ 0x53f1) - 0.5) * BLOCK_HEIGHT_VARIATION
        + planar_tilt
        + broad
        + tools
        - edge.bevel * BEVEL_RELIEF
        - detail.pore * PORE_RELIEF
        + detail.grain * GRAIN_RELIEF
        + detail.mineral * FACE_RELIEF
        - edge.exposed_chip * SPALL_RELIEF;

    StoneSample {
        height: mortar.height + (face_height - mortar.height) * stone_coverage,
        stone_id: id,
        edge_distance,
    }
}

fn stone_color(sample: StoneSample) -> ([u8; 3], u8) {
    // Intrinsic substrate IDs own color and finish. Geometry, cavities and
    // damage never brighten/darken albedo or add high-frequency roughness.
    if sample.edge_distance > 0.0 {
        return (MORTAR_ALBEDO, MORTAR_ROUGHNESS);
    }
    (
        STONE_PALETTE[sample.stone_id as usize % STONE_PALETTE.len()],
        STONE_ROUGHNESS_PALETTE[sample.stone_id as usize % STONE_ROUGHNESS_PALETTE.len()],
    )
}

fn height_at(heights: &[f32], x: i32, y: i32) -> f32 {
    let size = DRESSED_STONE_TEXTURE_SIZE as i32;
    heights[(y.rem_euclid(size) * size + x.rem_euclid(size)) as usize]
}

fn ambient_visibility(heights: &[f32], x: i32, y: i32) -> f32 {
    let center = height_at(heights, x, y);
    let mut obstruction = 0.0;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for step in [1, 3, 7, 15] {
            obstruction += ((height_at(heights, x + dx * step, y + dy * step) - center)
                / step as f32)
                .max(0.0);
        }
    }
    (1.0 - obstruction * 2.8).clamp(0.48, 1.0)
}

pub fn generate_dressed_stone_textures(images: &mut Assets<Image>) -> SurfaceTextureSet {
    let size = DRESSED_STONE_TEXTURE_SIZE;
    let samples = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                sample_stonework(
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
    let metres_per_texel = DRESSED_STONE_TILE_METRES / size as f32;
    let slope_scale = DRESSED_STONE_HEIGHT_RANGE_METRES / (2.0 * metres_per_texel);

    for y in 0..size {
        for x in 0..size {
            let sample = samples[(y * size + x) as usize];
            let (color, roughness) = stone_color(sample);
            albedo.extend_from_slice(&[color[0], color[1], color[2], 255]);
            let dx = height_at(&heights, x as i32 + 1, y as i32)
                - height_at(&heights, x as i32 - 1, y as i32);
            let dy = height_at(&heights, x as i32, y as i32 + 1)
                - height_at(&heights, x as i32, y as i32 - 1);
            let n = Vec3::new(-dx * slope_scale, -dy * slope_scale, 1.0).normalize();
            let encoded = ((n + Vec3::ONE) * 127.5)
                .round()
                .clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal.extend_from_slice(&[encoded.x as u8, encoded.y as u8, encoded.z as u8, 255]);
            let encoded_height = (sample.height * 255.0).round().clamp(0.0, 255.0) as u8;
            height.extend_from_slice(&[encoded_height, encoded_height, encoded_height, 255]);
            let ao = (ambient_visibility(&heights, x as i32, y as i32) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            arm.extend_from_slice(&[ao, roughness, 0, 255]);
        }
    }

    let mut albedo_image = image_rgba_mipped(albedo, size, true);
    albedo_image.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
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

    use super::*;

    fn generated() -> (Assets<Image>, SurfaceTextureSet) {
        let mut images = Assets::default();
        let textures = generate_dressed_stone_textures(&mut images);
        (images, textures)
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
        let mut maximum_error = 0.0_f32;
        for index in 0..512 {
            let coordinate = (index as f32 + 0.5) / 512.0;
            let sample = sample_stonework(coordinate, 0.37).height;
            maximum_error = maximum_error
                .max((sample - sample_stonework(coordinate + 1.0, 0.37).height).abs())
                .max((sample - sample_stonework(coordinate, 1.37).height).abs());
        }
        assert!(
            maximum_error < f32::EPSILON,
            "periodic height error: {maximum_error}"
        );
    }

    #[test]
    fn ashlar_has_declared_period_appropriate_scale() {
        assert_eq!(DRESSED_STONE_TILE_METRES, 7.2);
        assert!((0.020..=0.028).contains(&DRESSED_STONE_HEIGHT_RANGE_METRES));
        let (courses, total_height) = course_weights();
        for weight in courses {
            let metres = weight / total_height * DRESSED_STONE_TILE_METRES;
            assert!((0.27..=0.37).contains(&metres), "course height: {metres}");
        }
        for course in 0..COURSES {
            let (weights, total) = block_weights(course);
            for weight in weights.into_iter().take(block_count(course)) {
                let metres = weight / total * DRESSED_STONE_TILE_METRES;
                assert!((0.32..=0.90).contains(&metres), "block width: {metres}");
            }
        }
    }

    #[test]
    fn joints_are_recessed_while_faces_remain_planar() {
        let mut joints = Vec::new();
        let mut faces = Vec::new();
        for y in 0..256 {
            for x in 0..256 {
                let sample = sample_stonework((x as f32 + 0.5) / 256.0, (y as f32 + 0.5) / 256.0);
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
        for y in 0..128 {
            for x in 0..128 {
                let sample = sample_stonework(x as f32 / 128.0, y as f32 / 128.0);
                let (color, roughness) = stone_color(sample);
                assert!(STONE_PALETTE.contains(&color) || color == MORTAR_ALBEDO);
                assert!(
                    STONE_ROUGHNESS_PALETTE.contains(&roughness) || roughness == MORTAR_ROUGHNESS
                );
                assert_eq!(
                    stone_color(StoneSample {
                        height: 0.0,
                        ..sample
                    }),
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

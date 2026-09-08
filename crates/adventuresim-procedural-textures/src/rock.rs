use super::*;

pub const ROCK_TEXTURE_SIZE: u32 = 1024;
pub const ROCK_TILE_METRES: f32 = 2.0;
pub const ROCK_HEIGHT_RANGE_METRES: f32 = 0.032;

const ROCK_DOMAIN_COLUMNS: i32 = 8;
const ROCK_DOMAIN_ROWS: i32 = 8;
const HORIZON_DIRECTIONS: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];
const HORIZON_STEPS: [i32; 5] = [1, 3, 7, 15, 31];
const ROCK_PALETTE: [[u8; 3]; 4] = [
    [120, 119, 114],
    [124, 122, 117],
    [128, 125, 119],
    [132, 128, 122],
];
const ROCK_ROUGHNESS: [u8; 4] = [201, 209, 215, 205];

#[derive(Clone, Copy)]
struct RockFieldSample {
    height: f32,
    palette_index: usize,
    cavity: f32,
}

fn periodic_value_field(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
    columns: i32,
    rows: i32,
    salt: u64,
) -> f32 {
    let x = u.rem_euclid(1.0) * columns as f32;
    let y = v.rem_euclid(1.0) * rows as f32;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let blend_x = smoothstep(0.0, 1.0, x - x.floor());
    let blend_y = smoothstep(0.0, 1.0, y - y.floor());
    let value = |cell_x: i32, cell_y: i32| {
        let wrapped_x = cell_x.rem_euclid(columns) as u64;
        let wrapped_y = cell_y.rem_euclid(rows) as u64;
        unit_hash(crate::parameters::seeded_hash(
            params,
            wrapped_x | (wrapped_y << 16) | salt.rotate_left(29),
        )) * 2.0
            - 1.0
    };
    let lower = value(x0, y0).lerp(value(x0 + 1, y0), blend_x);
    let upper = value(x0, y0 + 1).lerp(value(x0 + 1, y0 + 1), blend_x);
    lower.lerp(upper, blend_y)
}

fn rock_field(params: &crate::TextureParameters, u: f32, v: f32) -> RockFieldSample {
    let broad = periodic_value_field(params, u, v, 3, 3, 0x4ad3);
    let structure = periodic_value_field(
        params,
        u,
        v,
        params.rock.domain_columns,
        params.rock.domain_rows,
        0xd513,
    );
    let aggregate = periodic_value_field(params, u, v, 19, 19, 0xb175);
    let grain = periodic_value_field(params, u, v, 41, 41, 0x8c29);
    // Smooth value-noise octaves make irregular pore/crystal grain without
    // the connected Voronoi edge graph that read as repeated U/Y/L stamps
    // once the tile was projected over broad cliff faces.
    let uv = Vec2::new(u, v);
    let facets = params.rock.facets.sample(params, uv, 0x1ab3);
    let pores = params.rock.pores.sample(params, uv, 0x7719);
    let height = (facets.facet - pores.bowl
        + params.rock.rock_field_height_1 * broad
        + params.rock.rock_field_height_2 * structure
        + params.rock.rock_field_height_3 * aggregate
        + params.rock.rock_field_height_4 * grain)
        .clamp(-0.5, 0.5);
    let material =
        broad * params.rock.rock_field_material_1 + structure * params.rock.rock_field_material_2;
    let palette_index = if material < -0.28 {
        0
    } else if material < -0.02 {
        1
    } else if material < 0.27 {
        2
    } else {
        3
    };
    RockFieldSample {
        height,
        palette_index: palette_index.min(params.rock.palette.len() - 1),
        cavity: pores.bowl / params.rock.pores.depth.max(f32::EPSILON),
    }
}

fn rock_horizon_ao(params: &crate::TextureParameters, heights: &[f32], x: i32, y: i32) -> f32 {
    let centre = periodic_sample(heights, params.size(ROCK_TEXTURE_SIZE), x, y)
        * params.rock.height_range_metres;
    let texel_metres = params.rock.tile_metres / params.size(ROCK_TEXTURE_SIZE) as f32;
    let mut visibility = 0.0;
    for (direction_x, direction_y) in HORIZON_DIRECTIONS {
        let direction_length =
            ((direction_x * direction_x + direction_y * direction_y) as f32).sqrt();
        let mut maximum_slope = 0.0_f32;
        for step in HORIZON_STEPS {
            let neighbor = periodic_sample(
                heights,
                params.size(ROCK_TEXTURE_SIZE),
                x + direction_x * step,
                y + direction_y * step,
            ) * params.rock.height_range_metres;
            let run = step as f32 * direction_length * texel_metres;
            maximum_slope = maximum_slope.max(((neighbor - centre) / run).max(0.0));
        }
        visibility += 1.0 / (1.0 + maximum_slope * maximum_slope).sqrt();
    }
    (visibility / HORIZON_DIRECTIONS.len() as f32).clamp(0.55, 1.0)
}

fn encode_normal(normal: Vec3) -> [u8; 4] {
    let encoded = ((normal + Vec3::ONE) * 127.5).clamp(Vec3::ZERO, Vec3::splat(255.0));
    [
        encoded.x.round() as u8,
        encoded.y.round() as u8,
        encoded.z.round() as u8,
        255,
    ]
}

fn base_levels(params: &crate::TextureParameters) -> [Vec<u8>; 4] {
    let pixel_count = (params.size(ROCK_TEXTURE_SIZE) * params.size(ROCK_TEXTURE_SIZE)) as usize;
    let texel = 1.0 / params.size(ROCK_TEXTURE_SIZE) as f32;
    let samples = (0..params.size(ROCK_TEXTURE_SIZE))
        .flat_map(|y| {
            (0..params.size(ROCK_TEXTURE_SIZE)).map(move |x| {
                rock_field(params, (x as f32 + 0.5) * texel, (y as f32 + 0.5) * texel)
            })
        })
        .collect::<Vec<_>>();
    let heights = samples
        .iter()
        .map(|sample| sample.height)
        .collect::<Vec<_>>();
    let mut albedo = Vec::with_capacity(pixel_count * 4);
    let mut normal = Vec::with_capacity(pixel_count * 4);
    let mut height = Vec::with_capacity(pixel_count * 4);
    let mut arm = Vec::with_capacity(pixel_count * 4);
    for y in 0..params.size(ROCK_TEXTURE_SIZE) {
        for x in 0..params.size(ROCK_TEXTURE_SIZE) {
            let index = (y * params.size(ROCK_TEXTURE_SIZE) + x) as usize;
            let sample = samples[index];
            albedo.extend_from_slice(&[
                params.rock.palette[sample.palette_index][0],
                params.rock.palette[sample.palette_index][1],
                params.rock.palette[sample.palette_index][2],
                255,
            ]);
            let height_x = periodic_sample(
                &heights,
                params.size(ROCK_TEXTURE_SIZE),
                x as i32 + 1,
                y as i32,
            ) - periodic_sample(
                &heights,
                params.size(ROCK_TEXTURE_SIZE),
                x as i32 - 1,
                y as i32,
            );
            let height_y = periodic_sample(
                &heights,
                params.size(ROCK_TEXTURE_SIZE),
                x as i32,
                y as i32 + 1,
            ) - periodic_sample(
                &heights,
                params.size(ROCK_TEXTURE_SIZE),
                x as i32,
                y as i32 - 1,
            );
            let slope_scale =
                params.rock.height_range_metres / (2.0 * texel * params.rock.tile_metres);
            normal.extend_from_slice(&encode_normal(crate::normal::from_image_gradient(
                height_x * slope_scale,
                height_y * slope_scale,
            )));
            let encoded_height = ((sample.height + 0.5) * 255.0).round() as u8;
            height.extend_from_slice(&[encoded_height, encoded_height, encoded_height, 255]);
            let ao = (rock_horizon_ao(params, &heights, x as i32, y as i32)
                * (1.0 - sample.cavity * params.rock.cavity_occlusion)
                * 255.0)
                .round() as u8;
            arm.extend_from_slice(&[ao, params.rock.roughness[sample.palette_index], 0, 255]);
        }
    }
    [albedo, normal, height, arm]
}

fn decode_normal(pixel: &[u8]) -> Vec3 {
    Vec3::new(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32) / 127.5 - Vec3::ONE
}

fn srgb_to_linear(value: u8) -> f32 {
    (value as f32 / 255.0).powf(2.2)
}

fn linear_to_srgb(value: f32) -> u8 {
    (value.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8
}

fn downsample_levels(
    params: &crate::TextureParameters,
    previous: [&[u8]; 4],
    previous_size: u32,
) -> [Vec<u8>; 4] {
    let next_size = previous_size / 2;
    let mut next =
        core::array::from_fn(|_| Vec::with_capacity((next_size * next_size * 4) as usize));
    for y in 0..next_size {
        for x in 0..next_size {
            let indices = [
                ((y * 2 * previous_size + x * 2) * 4) as usize,
                ((y * 2 * previous_size + x * 2 + 1) * 4) as usize,
                ((((y * 2 + 1) * previous_size) + x * 2) * 4) as usize,
                ((((y * 2 + 1) * previous_size) + x * 2 + 1) * 4) as usize,
            ];
            let mut linear_color = Vec3::ZERO;
            let mut normal_sum = Vec3::ZERO;
            let mut ao = 0.0;
            let mut roughness_squared = 0.0;
            let mut height = 0_u32;
            for index in indices {
                linear_color += Vec3::new(
                    srgb_to_linear(previous[0][index]),
                    srgb_to_linear(previous[0][index + 1]),
                    srgb_to_linear(previous[0][index + 2]),
                );
                normal_sum += decode_normal(&previous[1][index..index + 4]);
                height += previous[2][index] as u32;
                ao += previous[3][index] as f32 / 255.0;
                let roughness = previous[3][index + 1] as f32 / 255.0;
                roughness_squared += roughness * roughness;
            }
            linear_color *= 0.25;
            next[0].extend_from_slice(&[
                linear_to_srgb(linear_color.x),
                linear_to_srgb(linear_color.y),
                linear_to_srgb(linear_color.z),
                255,
            ]);
            let average_normal = normal_sum * 0.25;
            let normal_variance = (1.0 - average_normal.length()).max(0.0);
            next[1].extend_from_slice(&encode_normal(average_normal.normalize_or(Vec3::Z)));
            let encoded_height = ((height + 2) / 4) as u8;
            next[2].extend_from_slice(&[encoded_height, encoded_height, encoded_height, 255]);
            let average_ao = ao * params.rock.downsample_levels_average_ao;
            let filtered_ao = average_ao + (1.0 - average_ao) * normal_variance.min(1.0);
            let filtered_roughness = (roughness_squared
                * params.rock.downsample_levels_filtered_roughness_1
                + normal_variance * params.rock.downsample_levels_filtered_roughness_2)
                .sqrt()
                .clamp(0.0, 1.0);
            next[3].extend_from_slice(&[
                (filtered_ao * 255.0).round() as u8,
                (filtered_roughness * 255.0).round() as u8,
                0,
                255,
            ]);
        }
    }
    next
}

fn complete_mips(params: &crate::TextureParameters, base: [Vec<u8>; 4]) -> [Vec<u8>; 4] {
    let mut complete = base.clone();
    let mut previous = base;
    let mut previous_size = params.size(ROCK_TEXTURE_SIZE);
    while previous_size > 1 {
        let next = downsample_levels(
            params,
            [&previous[0], &previous[1], &previous[2], &previous[3]],
            previous_size,
        );
        for channel in 0..4 {
            complete[channel].extend_from_slice(&next[channel]);
        }
        previous = next;
        previous_size /= 2;
    }
    complete
}

fn rock_image(params: &crate::TextureParameters, data: Vec<u8>, srgb: bool) -> Image {
    let base_level_length =
        (params.size(ROCK_TEXTURE_SIZE) * params.size(ROCK_TEXTURE_SIZE) * 4) as usize;
    let mut image = Image::new(
        Extent3d {
            width: params.size(ROCK_TEXTURE_SIZE),
            height: params.size(ROCK_TEXTURE_SIZE),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data[..base_level_length].to_vec(),
        if srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(data);
    image.texture_descriptor.mip_level_count = params.size(ROCK_TEXTURE_SIZE).ilog2() + 1;
    use bevy::image::{ImageAddressMode, ImageSamplerDescriptor};
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        anisotropy_clamp: 8,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

pub(super) fn generate_rock_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> SurfaceTextureSet {
    let [albedo, normal, height, arm] = complete_mips(params, base_levels(params));
    SurfaceTextureSet {
        albedo: images.add(rock_image(params, albedo, true)),
        normal_gl: images.add(rock_image(params, normal, false)),
        height: images.add(rock_image(params, height, false)),
        arm: images.add(rock_image(params, arm, false)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn field_is_periodic_deterministic_and_physically_scaled() {
        let params = &crate::TextureParameters::default();
        for (u, v) in [(0.0, 0.17), (0.23, 0.51), (0.61, 0.97), (0.91, 0.08)] {
            let sample = rock_field(params, u, v);
            assert_eq!(
                sample.height.to_bits(),
                rock_field(params, u, v).height.to_bits()
            );
            assert!((sample.height - rock_field(params, u + 1.0, v).height).abs() < 1.0e-5);
            assert!((sample.height - rock_field(params, u, v + 1.0).height).abs() < 1.0e-5);
        }
        assert_eq!(ROCK_TILE_METRES / ROCK_DOMAIN_COLUMNS as f32, 0.25);
        assert!((0.024..=0.040).contains(&ROCK_HEIGHT_RANGE_METRES));
        assert!(ROCK_TILE_METRES / ROCK_TEXTURE_SIZE as f32 <= 0.008);
    }

    #[test]
    fn outputs_are_deterministic_mipped_and_channel_correct() {
        let params = &crate::TextureParameters::default();
        let mut first_images = Assets::<Image>::default();
        let first = generate_rock_textures(params, &mut first_images);
        let mut second_images = Assets::<Image>::default();
        let second = generate_rock_textures(params, &mut second_images);
        for (first_handle, second_handle) in [
            (&first.albedo, &second.albedo),
            (&first.normal_gl, &second.normal_gl),
            (&first.height, &second.height),
            (&first.arm, &second.arm),
        ] {
            let first_image = first_images.get(first_handle).unwrap();
            let second_image = second_images.get(second_handle).unwrap();
            assert_eq!(first_image.data, second_image.data);
            assert_eq!(
                first_image.texture_descriptor.mip_level_count,
                ROCK_TEXTURE_SIZE.ilog2() + 1
            );
            let mip_texels = (0..=ROCK_TEXTURE_SIZE.ilog2())
                .map(|level| (ROCK_TEXTURE_SIZE >> level).pow(2))
                .sum::<u32>();
            assert_eq!(
                first_image.data.as_ref().unwrap().len(),
                (mip_texels * 4) as usize
            );
        }
        let arm = first_images.get(&first.arm).unwrap().data.as_ref().unwrap();
        assert!(
            arm.as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[2] == 0 && pixel[3] == 255)
        );
    }

    #[test]
    fn dense_microrelief_is_smooth_without_fracture_graph_jumps() {
        let params = &crate::TextureParameters::default();
        let sample_count = 128;
        let texel = 1.0 / sample_count as f32;
        let mut minimum = f32::INFINITY;
        let mut maximum = f32::NEG_INFINITY;
        let mut maximum_jump = 0.0_f32;
        for y in 0..sample_count {
            for x in 0..sample_count {
                let u = x as f32 * texel;
                let v = y as f32 * texel;
                let height = rock_field(params, u, v).height;
                minimum = minimum.min(height);
                maximum = maximum.max(height);
                maximum_jump = maximum_jump
                    .max((height - rock_field(params, u + texel, v).height).abs())
                    .max((height - rock_field(params, u, v + texel).height).abs());
            }
        }
        assert!(
            maximum - minimum < 0.55,
            "height span: {}",
            maximum - minimum
        );
        assert!(maximum_jump < 0.08, "local height jump: {maximum_jump}");
    }

    #[test]
    fn base_color_and_roughness_use_restrained_solid_regions() {
        let params = &crate::TextureParameters::default();
        let [albedo, _, _, arm] = base_levels(params);
        let colors = albedo
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<BTreeSet<_>>();
        let roughness = arm
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[1])
            .collect::<BTreeSet<_>>();
        assert_eq!(colors, ROCK_PALETTE.into_iter().collect());
        assert_eq!(roughness, ROCK_ROUGHNESS.into_iter().collect());
    }
}

mod controls;
pub use controls::Parameters;

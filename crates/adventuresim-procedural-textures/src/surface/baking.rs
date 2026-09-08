use super::*;
pub(crate) fn periodic_sample(field: &[f32], size: u32, x: i32, y: i32) -> f32 {
    let size = size as i32;
    let wrapped_x = x.rem_euclid(size) as usize;
    let wrapped_y = y.rem_euclid(size) as usize;
    field[wrapped_y * size as usize + wrapped_x]
}

pub(crate) fn oak_bark_horizon_ao(
    params: &crate::TextureParameters,
    field: &[f32],
    x: i32,
    y: i32,
) -> f32 {
    debug_assert_eq!(
        field.len(),
        (params.size(OAK_BARK_TEXTURE_SIZE).pow(2)) as usize
    );
    let source_scale = (params.size(OAK_BARK_TEXTURE_SIZE) / params.size(OAK_BARK_AO_SIZE)) as i32;
    let source_x = x * source_scale + source_scale / 2;
    let source_y = y * source_scale + source_scale / 2;
    let centre = periodic_sample(
        field,
        params.size(OAK_BARK_TEXTURE_SIZE),
        source_x,
        source_y,
    ) * params.surface.oak_bark_height_range_metres;
    let ao_texel_metres =
        params.surface.oak_bark_tile_metres / params.size(OAK_BARK_AO_SIZE) as f32;
    let mut visibility = 0.0;
    for (direction_x, direction_y) in OAK_BARK_AO_DIRECTIONS {
        let mut maximum_slope = 0.0_f32;
        for ao_step in OAK_BARK_AO_STEPS {
            let source_step = ao_step * source_scale;
            let neighbor = periodic_sample(
                field,
                params.size(OAK_BARK_TEXTURE_SIZE),
                source_x + direction_x * source_step,
                source_y + direction_y * source_step,
            ) * params.surface.oak_bark_height_range_metres;
            let run = ao_step as f32 * ao_texel_metres;
            maximum_slope = maximum_slope.max(((neighbor - centre) / run).max(0.0));
        }
        visibility += 1.0 / (1.0 + maximum_slope * maximum_slope).sqrt();
    }
    (visibility / OAK_BARK_AO_DIRECTIONS.len() as f32).clamp(0.36, 1.0)
}

pub(crate) fn periodic_bilinear_sample(field: &[f32], size: u32, u: f32, v: f32) -> f32 {
    let x = u * size as f32 - 0.5;
    let y = v * size as f32 - 0.5;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let blend_x = x - x.floor();
    let blend_y = y - y.floor();
    let lower = periodic_sample(field, size, x0, y0)
        .lerp(periodic_sample(field, size, x0 + 1, y0), blend_x);
    let upper = periodic_sample(field, size, x0, y0 + 1)
        .lerp(periodic_sample(field, size, x0 + 1, y0 + 1), blend_x);
    lower.lerp(upper, blend_y)
}

pub(crate) fn oak_bark_local_cavity(
    params: &crate::TextureParameters,
    field: &[f32],
    x: i32,
    y: i32,
) -> f32 {
    let centre = periodic_sample(field, params.size(OAK_BARK_TEXTURE_SIZE), x, y);
    let neighbors = periodic_sample(field, params.size(OAK_BARK_TEXTURE_SIZE), x - 1, y)
        + periodic_sample(field, params.size(OAK_BARK_TEXTURE_SIZE), x + 1, y)
        + periodic_sample(field, params.size(OAK_BARK_TEXTURE_SIZE), x, y - 1)
        + periodic_sample(field, params.size(OAK_BARK_TEXTURE_SIZE), x, y + 1);
    let cavity = (neighbors * params.surface.oak_bark_local_cavity_cavity - centre).max(0.0);
    (1.0 - cavity * 1.5).clamp(0.72, 1.0)
}

pub(crate) fn generate_oak_bark_texture(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> BarkTextureSet {
    let size = params.size(OAK_BARK_TEXTURE_SIZE);
    let pixel_count = (size * size) as usize;
    let texel = 1.0 / size as f32;
    let heights = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                oak_bark_height(params, (x as f32 + 0.5) * texel, (y as f32 + 0.5) * texel)
            })
        })
        .collect::<Vec<_>>();
    let horizon_ao = (0..params.size(OAK_BARK_AO_SIZE))
        .flat_map(|y| {
            let heights = &heights;
            (0..params.size(OAK_BARK_AO_SIZE))
                .map(move |x| oak_bark_horizon_ao(params, heights, x as i32, y as i32))
        })
        .collect::<Vec<_>>();
    let mut height_ao = Vec::with_capacity(pixel_count * 2);
    for y in 0..size {
        for x in 0..size {
            let height = periodic_sample(&heights, size, x as i32, y as i32);
            let encoded_height = ((height + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8;
            let u = (x as f32 + 0.5) / size as f32;
            let v = (y as f32 + 0.5) / size as f32;
            let broad_visibility =
                periodic_bilinear_sample(&horizon_ao, params.size(OAK_BARK_AO_SIZE), u, v);
            let local_visibility = oak_bark_local_cavity(params, &heights, x as i32, y as i32);
            let ao = (broad_visibility * local_visibility * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            height_ao.extend_from_slice(&[encoded_height, ao]);
        }
    }
    BarkTextureSet {
        height_ao: images.add(image_rg_mipped(height_ao, size, true)),
    }
}

//! Shared leaf image packing and species palette selection.
use super::*;
pub(super) enum LeafMipSemantic {
    Coverage,
    ColorCoverage,
    Normal,
    Scalar,
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
                        let total: u32 = source_pixels.iter().map(|p| u32::from(p[0])).sum();
                        [((total + 2) / 4) as u8; 4]
                    }
                    LeafMipSemantic::ColorCoverage => {
                        let coverage: u32 = source_pixels.iter().map(|p| u32::from(p[3])).sum();
                        let mut result = [0; 4];
                        for channel in 0..3 {
                            result[channel] = (source_pixels
                                .iter()
                                .map(|p| u32::from(p[channel]) * u32::from(p[3]))
                                .sum::<u32>()
                                + coverage / 2)
                                .checked_div(coverage)
                                .unwrap_or(0) as u8;
                        }
                        result[3] = ((coverage + 2) / 4) as u8;
                        result
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
    crate::leaf::generate(params, images, species, recipe)
}

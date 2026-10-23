//! One textile rasterizer for preview materials and portable exported textures.
use anyhow::Result;
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use fabelgeist_armor::TextileAppearance;
use image::ImageEncoder;
pub struct TextileMaps {
    pub base_color_png: Vec<u8>,
    pub normal_png: Vec<u8>,
}

pub fn material(
    fabric: &TextileAppearance,
    images: &mut Assets<Image>,
) -> Result<StandardMaterial> {
    fabric.validate()?;
    let surface = fabric.material();
    let mut material = StandardMaterial {
        base_color: Color::srgba(
            surface.base_color[0],
            surface.base_color[1],
            surface.base_color[2],
            1.0,
        ),
        metallic: surface.metallic,
        perceptual_roughness: surface.roughness,
        reflectance: 0.5 * fabric.finish.specular_factor(),
        ..Default::default()
    };
    if !fabric.layers.0.is_empty() {
        let pixels = fabric.pixels()?;
        let mut image = Image::new(
            Extent3d {
                width: pixels.width,
                height: pixels.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels.color,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::ClampToEdge,
            ..ImageSamplerDescriptor::linear()
        });
        material.base_color = Color::WHITE;
        material.base_color_texture = Some(images.add(image));
    }
    Ok(material)
}

pub fn textures(fabric: &TextileAppearance) -> Result<Option<TextileMaps>> {
    fabric.validate()?;
    if fabric.layers.0.is_empty() {
        return Ok(None);
    }
    let pixels = fabric.pixels()?;
    Ok(Some(TextileMaps {
        base_color_png: encode(&pixels.color, pixels.width, pixels.height)?,
        normal_png: encode(&[128, 128, 255, 255], 1, 1)?,
    }))
}

#[cfg(feature = "offline-creator")]
impl From<TextileMaps> for crate::export::SurfaceTextures {
    fn from(maps: TextileMaps) -> Self {
        Self {
            base_color_png: Some(maps.base_color_png),
            normal_png: maps.normal_png,
            occlusion_png: None,
            cutout: false,
            sampling: crate::export::SurfaceSampling::WrappedGarment,
        }
    }
}

fn encode(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes).write_image(
        pixels,
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(bytes)
}

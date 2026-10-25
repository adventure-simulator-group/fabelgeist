//! One textile rasterizer for preview materials and portable exported textures.
use anyhow::Result;
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use fabelgeist_armor::{TextileAppearance, TextileChartBoundary, TextileFinish};
use image::ImageEncoder;
pub struct TextileMaps {
    pub base_color_png: Vec<u8>,
    pub normal_png: Vec<u8>,
    pub translucent: bool,
    pub chart_boundary: TextileChartBoundary,
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
            surface.base_color[3],
        ),
        metallic: surface.metallic,
        perceptual_roughness: surface.roughness,
        reflectance: 0.5 * fabric.finish.specular_factor(),
        alpha_mode: if fabric.is_translucent() {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        },
        // A closed thin shell renders its outward-facing wall once. Rendering
        // both sides would accumulate the same cloth opacity twice.
        cull_mode: Some(bevy::render::render_resource::Face::Back),
        ..Default::default()
    };
    if !fabric.layers.0.is_empty() || fabric.relief.is_some() || fabric.needlework.is_some() {
        let pixels = fabric.pixels()?;
        if fabric.finish == TextileFinish::MetallicThread
            || fabric.relief.is_some()
            || fabric.needlework.is_some()
        {
            material.normal_map_texture = Some(images.add(texture_image(
                pixels.width,
                pixels.height,
                pixels.surface_normals(fabric)?,
                TextureFormat::Rgba8Unorm,
                fabric.chart_boundary,
            )));
        }
        let image = texture_image(
            pixels.width,
            pixels.height,
            pixels.color,
            TextureFormat::Rgba8UnormSrgb,
            fabric.chart_boundary,
        );
        material.base_color = Color::WHITE;
        material.base_color_texture = Some(images.add(image));
    }
    Ok(material)
}

pub fn textures(fabric: &TextileAppearance) -> Result<Option<TextileMaps>> {
    fabric.validate()?;
    if fabric.layers.0.is_empty()
        && !fabric.is_translucent()
        && fabric.relief.is_none()
        && fabric.needlework.is_none()
    {
        return Ok(None);
    }
    let pixels = fabric.pixels()?;
    Ok(Some(TextileMaps {
        base_color_png: encode(&pixels.color, pixels.width, pixels.height)?,
        normal_png: if fabric.finish == TextileFinish::MetallicThread
            || fabric.relief.is_some()
            || fabric.needlework.is_some()
        {
            encode(
                &pixels.surface_normals(fabric)?,
                pixels.width,
                pixels.height,
            )?
        } else {
            encode(&[128, 128, 255, 255], 1, 1)?
        },
        translucent: fabric.is_translucent(),
        chart_boundary: fabric.chart_boundary,
    }))
}

#[cfg(feature = "offline-creator")]
impl From<TextileMaps> for crate::export::SurfaceTextures {
    fn from(maps: TextileMaps) -> Self {
        Self {
            base_color_png: Some(maps.base_color_png),
            normal_png: maps.normal_png,
            occlusion_png: None,
            alpha: if maps.translucent {
                crate::export::SurfaceAlpha::Blend
            } else {
                crate::export::SurfaceAlpha::Opaque
            },
            sides: crate::export::SurfaceSides::Front,
            sampling: match maps.chart_boundary {
                TextileChartBoundary::CyclicAcross => {
                    crate::export::SurfaceSampling::WrappedGarment
                }
                TextileChartBoundary::Open => crate::export::SurfaceSampling::OpenGarment,
            },
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

fn texture_image(
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    format: TextureFormat,
    boundary: TextileChartBoundary,
) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        format,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: match boundary {
            TextileChartBoundary::CyclicAcross => ImageAddressMode::Repeat,
            TextileChartBoundary::Open => ImageAddressMode::ClampToEdge,
        },
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

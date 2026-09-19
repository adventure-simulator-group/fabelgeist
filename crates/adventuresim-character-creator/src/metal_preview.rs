//! Preview materials for scratched armor metal, shared by plate and catalog armor.
use super::*;
use bevy::{
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use fabelgeist_armor::material::Metal;

/// A metal's baked scratch maps, uploaded for the preview.
pub(super) struct MetalImages {
    normal: Handle<Image>,
    metal_roughness: Handle<Image>,
}

impl MetalImages {
    pub(super) fn new(images: &mut Assets<Image>, metal: &Metal) -> Result<Self, String> {
        let textures = metal.textures(Metal::TEXTURE_SIZE)?;
        let mut upload = |data| {
            let mut image = Image::new(
                Extent3d {
                    width: textures.size,
                    height: textures.size,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                data,
                TextureFormat::Rgba8Unorm,
                RenderAssetUsages::default(),
            );
            image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..ImageSamplerDescriptor::linear()
            });
            images.add(image)
        };
        Ok(Self {
            normal: upload(textures.normal),
            metal_roughness: upload(textures.metal_roughness),
        })
    }

    /// The metal's color with its scratch maps; roughness comes from the map.
    pub(super) fn material(&self, metal: &Metal, metallic: f32) -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgb(metal.color[0], metal.color[1], metal.color[2]),
            metallic,
            perceptual_roughness: 1.0,
            normal_map_texture: Some(self.normal.clone()),
            metallic_roughness_texture: Some(self.metal_roughness.clone()),
            ..default()
        }
    }
}

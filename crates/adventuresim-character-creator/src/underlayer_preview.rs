//! Preview uses the same embedded atlas and cutout policy as exported equipment.
use super::*;
use adventuresim_character_creator::{armor_recipes::ParametricDesign, underlayer_material};
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::BTreeMap;

#[derive(Resource, Default)]
pub(super) struct EquipmentMaps {
    mail: Option<MailImages>,
    generated_normals: BTreeMap<[u8; 32], Handle<Image>>,
}

struct MailImages {
    color: Handle<Image>,
    normal: Handle<Image>,
    occlusion: Option<Handle<Image>>,
}

impl EquipmentMaps {
    pub(super) fn material(
        &mut self,
        images: &mut Assets<Image>,
        material: adventuresim_character_creator::item_catalog_schema::EquipmentMaterial,
        design: Option<&ParametricDesign>,
        armor: &adventuresim_armor_model::GeneratedArmor,
    ) -> StandardMaterial {
        let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
        let mut result = StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
            metallic,
            perceptual_roughness: roughness,
            ..default()
        };
        if let Some(textures) = underlayer_material::textures(design) {
            let maps = self.mail.get_or_insert_with(|| {
                let mut load = |bytes, srgb| {
                    images.add(
                        Image::from_buffer(
                            bytes,
                            ImageType::Extension("png"),
                            CompressedImageFormats::NONE,
                            srgb,
                            ImageSampler::default(),
                            RenderAssetUsages::default(),
                        )
                        .expect("authored mail atlas is a valid PNG"),
                    )
                };
                MailImages {
                    color: load(
                        textures
                            .base_color_png
                            .as_deref()
                            .expect("mail supplies a base-color texture"),
                        true,
                    ),
                    normal: load(&textures.normal_png, false),
                    occlusion: textures
                        .occlusion_png
                        .as_deref()
                        .map(|bytes| load(bytes, false)),
                }
            });
            result.base_color = Color::WHITE;
            result.base_color_texture = Some(maps.color.clone());
            result.normal_map_texture = Some(maps.normal.clone());
            result.occlusion_texture = maps.occlusion.clone();
            if textures.cutout {
                result.alpha_mode = AlphaMode::Mask(0.5);
            }
        } else if let Some(map) = &armor.normal_map {
            let normal = self
                .generated_normals
                .entry(armor.design_hash)
                .or_insert_with(|| {
                    images.add(Image::new(
                        Extent3d {
                            width: map.width,
                            height: map.height,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        map.rgba8.clone(),
                        TextureFormat::Rgba8Unorm,
                        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                    ))
                });
            result.normal_map_texture = Some(normal.clone());
        }
        result
    }
}

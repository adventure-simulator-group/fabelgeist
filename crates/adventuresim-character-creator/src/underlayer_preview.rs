//! Preview uses the same embedded atlas and cutout policy as exported equipment.
use super::*;
use adventuresim_character_creator::{armor_recipes::ParametricDesign, underlayer_material};
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};

#[derive(Resource, Default)]
pub(super) struct MailMaps {
    maps: Option<MailImages>,
}

struct MailImages {
    color: Handle<Image>,
    normal: Handle<Image>,
    occlusion: Option<Handle<Image>>,
}

impl MailMaps {
    pub(super) fn material(
        &mut self,
        images: &mut Assets<Image>,
        material: adventuresim_character_creator::item_catalog_schema::EquipmentMaterial,
        design: Option<&ParametricDesign>,
    ) -> StandardMaterial {
        let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
        let mut result = StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
            metallic,
            perceptual_roughness: roughness,
            ..default()
        };
        if let Some(textures) = underlayer_material::textures(design) {
            let maps = self.maps.get_or_insert_with(|| {
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
                    color: load(textures.base_color_png, true),
                    normal: load(textures.normal_png, false),
                    occlusion: textures.occlusion_png.map(|bytes| load(bytes, false)),
                }
            });
            result.base_color = Color::WHITE;
            result.base_color_texture = Some(maps.color.clone());
            result.normal_map_texture = Some(maps.normal.clone());
            result.occlusion_texture = maps.occlusion.clone();
            if textures.cutout {
                result.alpha_mode = AlphaMode::Mask(0.5);
            }
        }
        result
    }
}

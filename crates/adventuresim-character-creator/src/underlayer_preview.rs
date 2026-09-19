//! Catalog armor preview materials: the embedded mail atlas and cutout policy of
//! exported equipment, and the scratched metal shared with plate armor.
use super::*;
use adventuresim_character_creator::{
    armor_metal, armor_recipes::ParametricDesign, item_catalog_schema::EquipmentMaterial,
    underlayer_material,
};
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};

#[derive(Resource, Default)]
pub(super) struct EquipmentMaps {
    maps: Option<MailImages>,
    /// Scratch maps per catalog steel, baked once.
    metals: Vec<(EquipmentMaterial, metal_preview::MetalImages)>,
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
        material: EquipmentMaterial,
        design: Option<&ParametricDesign>,
    ) -> StandardMaterial {
        let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
        if let Some(metal) = armor_metal::metal(material) {
            let index = match self.metals.iter().position(|(known, _)| *known == material) {
                Some(index) => index,
                None => {
                    let baked = metal_preview::MetalImages::new(images, &metal)
                        .expect("catalog steels are valid metals");
                    self.metals.push((material, baked));
                    self.metals.len() - 1
                }
            };
            return self.metals[index].1.material(&metal, metallic);
        }
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

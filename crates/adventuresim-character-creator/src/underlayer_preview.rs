//! Catalog armor preview materials: the embedded mail atlas and cutout policy of
//! exported equipment, and the scratched metal shared with plate armor.
use super::*;
use adventuresim_character_creator::{
    armor_metal, armor_recipes::ParametricDesign, item_catalog_schema::EquipmentMaterial,
    underlayer_material,
};
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use fabelgeist_armor::{engraving::Engraving, material::Metal};

#[derive(Resource, Default)]
pub(super) struct EquipmentMaps {
    maps: Option<MailImages>,
    /// Baked steels, kept while a worn article still uses them.
    metals: Vec<BakedMetal>,
}

struct BakedMetal {
    metal: Metal,
    images: metal_preview::MetalImages,
    /// Whether an article used it since the generation began.
    used: bool,
}

struct MailImages {
    color: Handle<Image>,
    normal: Handle<Image>,
    occlusion: Option<Handle<Image>>,
}

impl EquipmentMaps {
    /// Forget the steels no article used during the last generation.
    pub(super) fn begin_generation(&mut self) {
        self.metals.retain(|baked| baked.used);
        for baked in &mut self.metals {
            baked.used = false;
        }
    }

    /// The preview material of a catalog article; baking a steel reads its
    /// engraving image.
    pub(super) fn material(
        &mut self,
        images: &mut Assets<Image>,
        material: EquipmentMaterial,
        design: Option<&ParametricDesign>,
        engraving: Option<&Engraving>,
    ) -> Result<StandardMaterial, String> {
        let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
        if let Some(metal) = armor_metal::metal(material, engraving) {
            let index = match self.metals.iter().position(|baked| baked.metal == metal) {
                Some(index) => index,
                None => {
                    self.metals.push(BakedMetal {
                        images: metal_preview::MetalImages::new(images, &metal)?,
                        metal,
                        used: false,
                    });
                    self.metals.len() - 1
                }
            };
            let baked = &mut self.metals[index];
            baked.used = true;
            return Ok(baked.images.material(&baked.metal, metallic));
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
        Ok(result)
    }
}

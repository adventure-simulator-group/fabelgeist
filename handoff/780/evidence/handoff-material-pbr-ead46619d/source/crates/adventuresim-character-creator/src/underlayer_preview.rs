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
    mail: Option<MailImages>,
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
    ) -> anyhow::Result<StandardMaterial> {
        let appearance = adventuresim_character_creator::equipment_pbr(material);
        if let Some(metal) = armor_metal::metal(material, engraving) {
            return self.metal(images, &metal, appearance.metallic);
        }
        let mut result = adventuresim_character_creator::surface_projection::standard_material(appearance);
        if let Some(textures) = underlayer_material::textures(design) {
            let maps = match self.mail {
                Some(ref maps) => maps,
                None => self.mail.insert(MailImages::new(images, &textures)?),
            };
            result.base_color = Color::WHITE;
            result.base_color_texture = Some(maps.color.clone());
            result.normal_map_texture = Some(maps.normal.clone());
            result.occlusion_texture = maps.occlusion.clone();
            result.alpha_mode = match textures.alpha {
                adventuresim_character_creator::export::SurfaceAlpha::Opaque => AlphaMode::Opaque,
                adventuresim_character_creator::export::SurfaceAlpha::Mask => AlphaMode::Mask(0.5),
                adventuresim_character_creator::export::SurfaceAlpha::Blend => AlphaMode::Blend,
            };
            result.cull_mode = match textures.sides {
                adventuresim_character_creator::export::SurfaceSides::Front => {
                    Some(bevy::render::render_resource::Face::Back)
                }
                adventuresim_character_creator::export::SurfaceSides::Both => None,
            };
        }
        Ok(result)
    }

    /// The preview material of a scratched metal, baked once while used.
    pub(super) fn metal(
        &mut self,
        images: &mut Assets<Image>,
        metal: &Metal,
        metallic: fabelgeist_math::units::Ratio,
    ) -> anyhow::Result<StandardMaterial> {
        let index = match self.metals.iter().position(|baked| baked.metal == *metal) {
            Some(index) => index,
            None => {
                self.metals.push(BakedMetal {
                    images: metal_preview::MetalImages::new(images, metal)?,
                    metal: metal.clone(),
                    used: false,
                });
                self.metals.len() - 1
            }
        };
        let baked = &mut self.metals[index];
        baked.used = true;
        Ok(baked.images.material(&baked.metal, metallic))
    }
}

impl MailImages {
    fn new(
        images: &mut Assets<Image>,
        textures: &adventuresim_character_creator::export::SurfaceTextures,
    ) -> anyhow::Result<Self> {
        let decode = |bytes: &[u8], srgb| {
            Image::from_buffer(
                bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                srgb,
                ImageSampler::default(),
                RenderAssetUsages::default(),
            )
            .context("decoding the mail atlas")
        };
        let color = decode(
            textures
                .base_color_png
                .as_deref()
                .context("mail supplies no base-color texture")?,
            true,
        )?;
        let normal = decode(&textures.normal_png, false)?;
        let occlusion = textures
            .occlusion_png
            .as_deref()
            .map(|bytes| decode(bytes, false))
            .transpose()?;
        Ok(Self {
            color: images.add(color),
            normal: images.add(normal),
            occlusion: occlusion.map(|image| images.add(image)),
        })
    }
}

//! Meshes, materials and the card atlas the previewed hair is drawn with.
use super::*;
use adventuresim_character_creator::hair::{HAIR_ROUGHNESS, HairGeometry};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, VertexAttributeValues},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// The assets hair is made of.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HairAssets<'w> {
    pub(super) meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    pub(super) images: ResMut<'w, Assets<Image>>,
}

impl HairAssets<'_> {
    /// `form` of the placed hair as a mesh skinned wholly to the head joint,
    /// its baked occlusion as vertex colour.
    pub(super) fn hair_mesh(&mut self, placed: &PlacedStyle, form: Form) -> Result<Handle<Mesh>> {
        let head = placed.hair.placement().head_joint;
        Ok(self.meshes.add(mesh(placed.hair.geometry(form), head)?))
    }

    /// Cards cut their painted strands out of the atlas; ribbons are each
    /// one strand, solid.
    pub(super) fn hair_material(
        &mut self,
        design: HairDesign,
        form: Form,
        atlas: Handle<Image>,
    ) -> Handle<StandardMaterial> {
        self.materials.add(material(design, form, atlas))
    }
}

/// The material `form` of hair of `design` is drawn with. A card's atlas
/// coverage becomes a share of each pixel's samples, as the hair crate's
/// renderer draws cards: a cut at half coverage would drop the fine,
/// half-covered strands a coarse card is painted with, and leave the scalp
/// bare between them.
pub(super) fn material(design: HairDesign, form: Form, atlas: Handle<Image>) -> StandardMaterial {
    let [red, green, blue] = design.pigment.color();
    let cards = matches!(form, Form::Cards { .. });
    StandardMaterial {
        base_color: Color::linear_rgb(red, green, blue),
        base_color_texture: cards.then_some(atlas),
        alpha_mode: if cards {
            AlphaMode::AlphaToCoverage
        } else {
            AlphaMode::Opaque
        },
        perceptual_roughness: HAIR_ROUGHNESS,
        metallic: 0.0,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

pub(super) fn mesh(geometry: HairGeometry, head_joint: u32) -> Result<Mesh> {
    let count = geometry.positions.len();
    let head = u16::try_from(head_joint).context("the head joint index exceeds a skin index")?;
    let colors: Vec<[f32; 4]> = geometry
        .occlusion
        .iter()
        .map(|&visible| [visible, visible, visible, 1.0])
        .collect();
    Ok(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, geometry.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, geometry.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, geometry.texcoords)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[head, 0, 0, 0]; count]),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        vec![[1.0f32, 0.0, 0.0, 0.0]; count],
    )
    .with_inserted_indices(Indices::U32(geometry.faces.into_flattened())))
}

/// The card atlas with its mip chain, shade as grey and coverage as alpha.
pub(super) fn atlas_image(hair: &PlacedHair) -> Image {
    let levels = hair.atlas_levels();
    let (width, height) = levels
        .first()
        .map_or((1, 1), |level| (level.width, level.height));
    let base = levels
        .first()
        .map_or_else(Vec::new, |level| level.rgba.clone());
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        base,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    // The whole chain, largest level first, as wgpu uploads mipmaps.
    image.data = Some(
        levels
            .iter()
            .flat_map(|level| level.rgba.iter().copied())
            .collect(),
    );
    image.texture_descriptor.mip_level_count = levels.len() as u32;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

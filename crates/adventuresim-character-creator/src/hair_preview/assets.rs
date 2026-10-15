//! Meshes, materials and the card atlas the previewed hair is drawn with.
use super::strand_shading::{HairShading, StrandShading};
use super::*;
use adventuresim_character_creator::hair::{HAIR_ROUGHNESS, HairGeometry, ScalpCap};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, VertexAttributeValues},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// How far hair's highlights lean from white towards its pigment's hue.
const HIGHLIGHT_HUE: f32 = 0.4;
/// Coverage under which a card casts no shadow and writes no depth.
const SHADOW_COVERAGE: f32 = 0.5;

/// The assets hair is made of.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HairAssets<'w> {
    pub(super) meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    hair_materials: ResMut<'w, Assets<HairShading>>,
    pub(super) images: ResMut<'w, Assets<Image>>,
}

impl HairAssets<'_> {
    /// `form` of the placed hair as a mesh skinned wholly to the head joint,
    /// its baked occlusion as vertex colour.
    pub(super) fn hair_mesh(&mut self, placed: &PlacedStyle, form: Form) -> Result<Handle<Mesh>> {
        let head = placed.hair.placement().head_joint;
        Ok(self.meshes.add(mesh(placed.hair.geometry(form), head)?))
    }

    /// The scalp cap under `placed`, skinned wholly to the head joint and
    /// shaded for `design`, blended over the skin.
    pub(super) fn scalp_cap(
        &mut self,
        placed: &PlacedStyle,
        design: HairDesign,
    ) -> Result<(Handle<Mesh>, Handle<StandardMaterial>)> {
        let hair = &placed.hair;
        let mesh = cap_mesh(hair.cap(), design, hair.placement().head_joint)?;
        Ok((
            self.meshes.add(mesh),
            self.materials.add(scalp_shade::material()),
        ))
    }

    /// Cards cut their painted strands out of the atlas; ribbons are each
    /// one strand, solid.
    pub(super) fn hair_material(
        &mut self,
        design: HairDesign,
        form: Form,
        atlas: Handle<Image>,
    ) -> Handle<HairShading> {
        self.hair_materials.add(material(design, form, atlas))
    }
}

/// The material `form` of hair of `design` is drawn with, shaded as strands
/// ([`HairShading`]). A card is cut out of `atlas`: its coverage becomes a
/// share of each pixel's samples, hashed per card as the hair crate's
/// renderer does, while shadows and depth cut it at half. A ribbon is one
/// strand, solid. Hair's highlights lean towards its pigment's hue: light
/// reflected inside a strand is coloured by it, and white highlights over
/// dark pigment read as grey hair.
pub(super) fn material(design: HairDesign, form: Form, atlas: Handle<Image>) -> HairShading {
    let [red, green, blue] = design.pigment.color();
    let brightest = red.max(green).max(blue).max(f32::EPSILON);
    let tint = |channel: f32| 1.0 + (channel / brightest - 1.0) * HIGHLIGHT_HUE;
    let cards = matches!(form, Form::Cards { .. });
    HairShading {
        base: StandardMaterial {
            base_color: Color::linear_rgb(red, green, blue),
            base_color_texture: cards.then_some(atlas),
            specular_tint: Color::linear_rgb(tint(red), tint(green), tint(blue)),
            alpha_mode: AlphaMode::Mask(SHADOW_COVERAGE),
            perceptual_roughness: HAIR_ROUGHNESS,
            metallic: 0.0,
            double_sided: true,
            cull_mode: None,
            ..default()
        },
        extension: StrandShading {},
    }
}

pub(super) fn mesh(geometry: HairGeometry, head_joint: u32) -> Result<Mesh> {
    let count = geometry.positions.len();
    let tangents: Vec<[f32; 4]> = geometry
        .tangents
        .iter()
        .map(|&[x, y, z]| [x, y, z, 1.0])
        .collect();
    // Each card's random value rides in the second texture coordinates.
    let randoms: Vec<[f32; 2]> = geometry
        .randoms
        .iter()
        .map(|&random| [random, 0.0])
        .collect();
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, randoms)
    .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, tangents)
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

/// The scalp `cap` as a mesh skinned wholly to `head_joint`, its shade for
/// hair of `design` as vertex colour.
pub(super) fn cap_mesh(cap: &ScalpCap, design: HairDesign, head_joint: u32) -> Result<Mesh> {
    let count = cap.positions.len();
    let head = u16::try_from(head_joint).context("the head joint index exceeds a skin index")?;
    Ok(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, cap.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, cap.normals.clone())
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_COLOR,
        scalp_shade::colors(design, &cap.cover, &cap.opacity),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[head, 0, 0, 0]; count]),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        vec![[1.0f32, 0.0, 0.0, 0.0]; count],
    )
    .with_inserted_indices(Indices::U32(cap.faces.as_flattened().to_vec())))
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

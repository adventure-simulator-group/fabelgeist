//! The hair on the previewed character.
//!
//! A hairstyle is grown once, on a worker thread, in the hair crate's bust
//! frame; every body the studio generates then only places the grown cards
//! on its head. The hair is skinned wholly to the head joint, so it follows
//! the walk preview as the head does.
use super::*;
use adventuresim_character_creator::hair::{
    GroomedHair, HAIR_ROUGHNESS, HairCards, HairDesign, HeadPlacement, HeadSkin, preset_label,
};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, VertexAttributeValues},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use fabelgeist_hair::Preset;
use std::sync::Arc;
use std::thread::JoinHandle;

/// Atlas texels with less coverage than this are cut away.
const COVERAGE_CUTOFF: f32 = 0.5;

/// The hair mesh shown on the character, and the design it shows.
#[derive(Component)]
pub(super) struct HairMesh(HairDesign);

/// The grown hairstyle and the worker growing the next one.
#[derive(Resource, Default)]
pub(super) struct HairPreview {
    grown: Option<GrownStyle>,
    worker: Option<(Preset, JoinHandle<GroomedHair>)>,
}

struct GrownStyle {
    hair: Arc<GroomedHair>,
    atlas: Handle<Image>,
}

impl HairPreview {
    /// The grown hairstyle, if it is `preset`.
    fn grown(&self, preset: Preset) -> Option<&GrownStyle> {
        self.grown
            .as_ref()
            .filter(|grown| grown.hair.preset == preset)
    }

    /// Start growing `preset` unless it is grown or growing already; true
    /// when a worker started. A worker growing another style is left to
    /// finish and its hair dropped.
    fn request(&mut self, preset: Preset) -> bool {
        let growing = self.worker.as_ref().map(|(growing, _)| *growing);
        if self.grown(preset).is_some() || growing == Some(preset) {
            return false;
        }
        self.worker = Some((
            preset,
            std::thread::spawn(move || GroomedHair::grow(preset)),
        ));
        true
    }

    /// Take the worker's hair once it has finished.
    fn collect(&mut self, images: &mut Assets<Image>) -> Result<()> {
        if !self
            .worker
            .as_ref()
            .is_some_and(|(_, worker)| worker.is_finished())
        {
            return Ok(());
        }
        let Some((_, worker)) = self.worker.take() else {
            return Ok(());
        };
        let hair = worker
            .join()
            .map_err(|_| anyhow::anyhow!("the hair worker panicked"))?;
        let atlas = images.add(atlas_image(&hair));
        self.grown = Some(GrownStyle {
            hair: Arc::new(hair),
            atlas,
        });
        Ok(())
    }
}

/// The assets hair is made of.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct HairAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

/// Keep the shown hair in step with the recipe: grow a new style, recolour,
/// and place it again on every body the studio regenerates (which despawns
/// it with the rest of the character).
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the studio, body caches and the assets the hair is made of"
)]
pub(super) fn refresh(
    mut commands: Commands,
    mut preview: ResMut<HairPreview>,
    mut studio: ResMut<Studio>,
    mut cache: ResMut<studio_cache::StudioCache>,
    model: Res<BodyModel>,
    walk: Res<WalkPreview>,
    shown: Query<(Entity, &HairMesh)>,
    mut assets: HairAssets,
) {
    let wanted = studio.recipe.hair;
    if let Some(design) = wanted
        && preview.request(design.preset)
    {
        studio.status = format!(
            "Growing {} hair",
            preset_label(design.preset).to_lowercase()
        );
    }
    if let Err(error) = preview.collect(&mut assets.images) {
        studio.status = format!("{error:#}");
    }
    let current = shown.iter().find(|(_, shown)| Some(shown.0) == wanted);
    for (entity, _) in shown.iter().filter(|(_, shown)| Some(shown.0) != wanted) {
        commands.entity(entity).despawn();
    }
    // Wait for the body the hair goes on, and for the hair itself.
    let (Some(design), None) = (wanted, current) else {
        return;
    };
    if studio.dirty || walk.joints.is_empty() {
        return;
    }
    let Some(grown) = preview.grown(design.preset) else {
        return;
    };
    let recipe = studio.recipe.clone();
    let placed = cache.body(&model, &recipe).and_then(|generated| {
        let (cards, placement) = place(&grown.hair, &model, &generated)?;
        hair_mesh(cards, placement.head_joint)
    });
    let mesh = match placed {
        Ok(placed) => placed,
        Err(error) => {
            studio.status = format!("Hair could not be placed: {error:#}");
            return;
        }
    };
    let Some(skin) = animation_preview::skin(&walk) else {
        return;
    };
    commands.spawn((
        CharacterMesh,
        HairMesh(design),
        Name::new("Hair"),
        skin,
        Mesh3d(assets.meshes.add(mesh)),
        MeshMaterial3d(
            assets
                .materials
                .add(hair_material(design, grown.atlas.clone())),
        ),
    ));
}

/// `hair`'s cards placed on the head of the `generated` body.
pub(super) fn place(
    hair: &GroomedHair,
    model: &BodyModel,
    generated: &GeneratedCharacter,
) -> Result<(HairCards, HeadPlacement)> {
    let character = &model.mhr.character;
    let skin = HeadSkin {
        positions: &generated.positions,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
        global_joint_states: &generated.global_joint_states,
    };
    let placement = HeadPlacement::fit(&skin, hair.head())?;
    Ok((hair.place(&placement), placement))
}

/// The placed cards as a mesh skinned wholly to the head joint, their baked
/// occlusion as vertex colour.
fn hair_mesh(cards: HairCards, head_joint: u32) -> Result<Mesh> {
    let count = cards.positions.len();
    let head = u16::try_from(head_joint).context("the head joint index exceeds a skin index")?;
    let colors: Vec<[f32; 4]> = cards
        .occlusion
        .iter()
        .map(|&visible| [visible, visible, visible, 1.0])
        .collect();
    Ok(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, cards.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, cards.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, cards.texcoords)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[head, 0, 0, 0]; count]),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        vec![[1.0f32, 0.0, 0.0, 0.0]; count],
    )
    .with_inserted_indices(Indices::U32(cards.faces.into_flattened())))
}

fn hair_material(design: HairDesign, atlas: Handle<Image>) -> StandardMaterial {
    let [red, green, blue] = design.pigment.color();
    StandardMaterial {
        base_color: Color::linear_rgb(red, green, blue),
        base_color_texture: Some(atlas),
        alpha_mode: AlphaMode::Mask(COVERAGE_CUTOFF),
        perceptual_roughness: HAIR_ROUGHNESS,
        metallic: 0.0,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

/// The card atlas with its mip chain, shade as grey and coverage as alpha.
fn atlas_image(hair: &GroomedHair) -> Image {
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

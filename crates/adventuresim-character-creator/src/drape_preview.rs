use super::*;
use adventuresim_character_creator::{
    garment::{DrapeCheckpoints, DrapeInput, DrapedGarment, OutfitOutcome, drape_outfit},
    garment_material::{CLOTH_PBR, MailMaps, MailWeave},
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Recent weaves keep their materials while a chainmail slider is dragged.
const MAIL_MATERIAL_CACHE: usize = 4;

#[derive(Resource, Default)]
pub struct DrapeJob {
    handle: Option<std::thread::JoinHandle<OutfitOutcome>>,
    pending: Option<Vec<DrapeInput>>,
    cancel: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<Vec<DrapedGarment>>>>,
    pub ready: Option<Vec<DrapedGarment>>,
    /// Completed stages of the most recent drape, reused by the next request.
    checkpoints: Vec<DrapeCheckpoints>,
}
impl Drop for DrapeJob {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl DrapeJob {
    pub fn request(&mut self, input: Vec<DrapeInput>) {
        self.cancel.store(true, Ordering::Relaxed);
        self.pending = (!input.is_empty()).then_some(input);
        self.ready = None;
        *self.latest.lock().unwrap() = None;
    }

    /// Make the next request simulate every stage again.
    pub fn restart_from_placement(&mut self) {
        self.checkpoints.clear();
    }
}

/// A draped garment's preview mesh, by its index in the recipe.
#[derive(Component)]
pub struct DrapeMesh(usize);

/// Preview materials for recently shown chainmail weaves.
#[derive(Resource, Default)]
pub struct MailMaterials {
    recent: Vec<(MailWeave, StandardMaterial)>,
}

impl MailMaterials {
    /// The same generated maps, ring repeat and factors the GLB export writes.
    fn material(&mut self, images: &mut Assets<Image>, weave: MailWeave) -> StandardMaterial {
        use bevy::image::{
            CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor,
            ImageType,
        };
        if let Some((_, material)) = self.recent.iter().find(|(cached, _)| *cached == weave) {
            return material.clone();
        }
        let maps = MailMaps::new(&weave).expect("the recipe validated the weave");
        let textures = maps.textures();
        let mut load = |bytes: &[u8], srgb| {
            images.add(
                Image::from_buffer(
                    bytes,
                    ImageType::Extension("png"),
                    CompressedImageFormats::NONE,
                    srgb,
                    ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        ..default()
                    }),
                    RenderAssetUsages::default(),
                )
                .expect("generated weave is a valid PNG"),
            )
        };
        let (color, metallic, roughness) = weave.pbr();
        let [across, up] = weave.repeat_m();
        let material = StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
            base_color_texture: Some(load(textures.base_color_png, true)),
            normal_map_texture: Some(load(textures.normal_png, false)),
            occlusion_texture: textures.occlusion_png.map(|bytes| load(bytes, false)),
            // Draped UVs are pattern metres; one texture repeat is one ring tile.
            uv_transform: bevy::math::Affine2::from_scale(Vec2::new(1.0 / across, 1.0 / up)),
            metallic,
            perceptual_roughness: roughness,
            alpha_mode: AlphaMode::Mask(0.5),
            cull_mode: None,
            double_sided: true,
            ..default()
        };
        if self.recent.len() == MAIL_MATERIAL_CACHE {
            self.recent.remove(0);
        }
        self.recent.push((weave, material.clone()));
        material
    }
}

fn cloth_material() -> StandardMaterial {
    let (color, metallic, roughness) = CLOTH_PBR;
    StandardMaterial {
        base_color: Color::srgba(color[0], color[1], color[2], color[3]),
        metallic,
        perceptual_roughness: roughness,
        cull_mode: None,
        double_sided: true,
        ..default()
    }
}

pub fn input(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    selection: adventuresim_character_creator::garment::GarmentSelection,
) -> DrapeInput {
    let character = &model.mhr.character;
    DrapeInput {
        armor: None,
        selection,
        obstacles: vec![],
        positions: generated.positions.clone(),
        faces: character.mesh.faces.clone(),
        names: character.skeleton.names.clone(),
        joints: generated.global_joint_states.clone(),
        indices: character.skin_weights.index.clone(),
        weights: character.skin_weights.weight.clone(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the drape job, scene and asset resources into this system"
)]
pub fn poll(
    mut job: ResMut<DrapeJob>,
    mut studio: ResMut<Studio>,
    mut commands: Commands,
    old: Query<Entity, With<DrapeMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut mail: ResMut<MailMaterials>,
) {
    let mut snapshot = if job.cancel.load(Ordering::Relaxed) {
        None
    } else {
        job.latest.lock().unwrap().take()
    };
    if job
        .handle
        .as_ref()
        .is_some_and(|handle| handle.is_finished())
    {
        let outcome = job
            .handle
            .take()
            .unwrap()
            .join()
            .unwrap_or_else(|_| OutfitOutcome {
                garments: Err(anyhow::anyhow!("Drape worker failed")),
                checkpoints: Vec::new(),
            });
        // Keep completed stages even from a failed or cancelled drape.
        job.checkpoints = outcome.checkpoints;
        if !job.cancel.load(Ordering::Relaxed) {
            match outcome.garments {
                Ok(garments) => {
                    studio.status = format!(
                        "Draped {} vertices; ready to export",
                        garments.iter().map(|g| g.positions.len()).sum::<usize>()
                    );
                    snapshot = Some(garments.clone());
                    job.ready = Some(garments);
                }
                Err(error) => {
                    studio.status = format!("Draping failed: {error:#}");
                    snapshot = None;
                }
            }
        }
    }
    if job.handle.is_none()
        && let Some(input) = job.pending.take()
    {
        let cancel = Arc::new(AtomicBool::new(false));
        job.cancel = cancel.clone();
        let latest = Arc::new(Mutex::new(None));
        job.latest = latest.clone();
        let previous = std::mem::take(&mut job.checkpoints);
        job.handle = Some(std::thread::spawn(move || {
            drape_outfit(input, previous, &cancel, |garments| {
                if !cancel.load(Ordering::Relaxed) {
                    *latest.lock().unwrap() = Some(garments);
                }
            })
        }));
        studio.status = "Measuring body and preparing cloth...".into();
    }
    if let Some(garments) = snapshot {
        for entity in &old {
            commands.entity(entity).despawn();
        }
        for (index, garment) in garments.into_iter().enumerate() {
            let cloth_faces = garment.faces.clone();
            if job.ready.is_none() {
                studio.status = format!("Draping: {}", garment.stage);
            }
            let has_skin = garment.indices.len() == garment.positions.len()
                && garment.weights.len() == garment.positions.len();
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, garment.positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, garment.normals.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, garment.texcoords.clone())
            .with_inserted_indices(Indices::U32(garment.faces.into_iter().flatten().collect()));
            let weave = studio.recipe.garments.get(index).map(|s| s.mail);
            let material = match weave {
                Some(weave) if garment.fabric == FabricPreset::Chainmail => {
                    mail.material(&mut images, weave)
                }
                _ => cloth_material(),
            };
            mesh.generate_tangents()
                .expect("draped panels have material UVs");
            let entity = commands
                .spawn((
                    CharacterMesh,
                    DrapeMesh(index),
                    Name::new(garment.name),
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(material)),
                ))
                .id();
            if has_skin {
                commands.entity(entity).insert(
                    animation_preview::ClothSkin::new(
                        garment.preset,
                        garment.positions,
                        garment.normals,
                        cloth_faces,
                        garment.indices,
                        garment.weights,
                    )
                    .weld_seams(),
                );
            }
        }
    }
}

/// Apply edited chainmail appearance to the shown garments without draping.
pub fn refresh_mail(
    studio: Res<Studio>,
    mut applied: Local<Vec<MailWeave>>,
    mut mail: ResMut<MailMaterials>,
    drapes: Query<(&DrapeMesh, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let weaves: Vec<_> = studio.recipe.garments.iter().map(|g| g.mail).collect();
    if *applied == weaves {
        return;
    }
    for (drape, handle) in &drapes {
        if let Some(selection) = studio.recipe.garments.get(drape.0)
            && selection.fabric == FabricPreset::Chainmail
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            *material = mail.material(&mut images, selection.mail);
        }
    }
    *applied = weaves;
}

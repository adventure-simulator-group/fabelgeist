use super::*;
use adventuresim_character_creator::{
    garment::{DrapeCheckpoints, DrapeInput, DrapedGarment, OutfitOutcome, drape_outfit},
    garment_material::{CLOTH_PBR, MailMaps, MailWeave},
    inventory::{Article, InventoryItemId},
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
    pending: Option<Vec<(InventoryItemId, DrapeInput)>>,
    /// The inventory article of each garment the running drape produces.
    draping: Vec<InventoryItemId>,
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
    /// Drape worn garments, innermost first.
    pub fn request(&mut self, input: Vec<(InventoryItemId, DrapeInput)>) {
        self.cancel.store(true, Ordering::Relaxed);
        self.pending = (!input.is_empty()).then_some(input);
        self.ready = None;
        *self.latest.lock().unwrap() = None;
    }

    /// Make the next request simulate every stage again.
    pub fn restart_from_placement(&mut self) {
        self.checkpoints.clear();
    }

    /// Whether a drape is queued or running. Only this holds back animation and
    /// export; a finished drape with problems does not.
    pub fn running(&self) -> bool {
        self.handle.is_some() || self.pending.is_some()
    }
}

/// A draped garment's preview mesh, by its inventory article.
#[derive(Component)]
pub struct DrapeMesh(InventoryItemId);

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
                garments: Vec::new(),
                warnings: Vec::new(),
                error: Some(anyhow::anyhow!("Drape worker failed")),
                checkpoints: Vec::new(),
            });
        if !job.cancel.load(Ordering::Relaxed) {
            let vertices = outcome
                .garments
                .iter()
                .map(|g| g.positions.len())
                .sum::<usize>();
            studio.status = match outcome.problems() {
                None => format!("Draped {vertices} vertices; ready to export"),
                Some(problems) => format!(
                    "Draped {} of {} garments with problems: {problems}. Press Drape again to retry.",
                    outcome.garments.len(),
                    job.draping.len()
                ),
            };
            // Whatever draped is shown and usable, even when some of it failed.
            snapshot = Some(outcome.garments.clone());
            job.ready = Some(outcome.garments);
        }
        // Keep completed stages even from a failed or cancelled drape.
        job.checkpoints = outcome.checkpoints;
    }
    if job.handle.is_none()
        && let Some(pending) = job.pending.take()
    {
        let (ids, input): (Vec<_>, Vec<_>) = pending.into_iter().unzip();
        job.draping = ids;
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
        for (&id, garment) in job.draping.iter().zip(garments) {
            if job.ready.is_none() {
                studio.status = format!("Draping: {}", garment.stage);
            }
            let material = match draped_selection(&studio, id) {
                Some(selection) if garment.fabric == FabricPreset::Chainmail => {
                    mail.material(&mut images, selection.mail)
                }
                _ => cloth_material(),
            };
            spawn_garment(
                &mut commands,
                &mut meshes,
                &mut materials,
                id,
                garment,
                material,
            );
        }
    }
}

fn spawn_garment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    id: InventoryItemId,
    garment: DrapedGarment,
    material: StandardMaterial,
) {
    let cloth_faces = garment.faces.clone();
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
    mesh.generate_tangents()
        .expect("draped panels have material UVs");
    let entity = commands
        .spawn((
            CharacterMesh,
            DrapeMesh(id),
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

/// Apply edited chainmail appearance to the shown garments without draping.
pub fn refresh_mail(
    studio: Res<Studio>,
    mut applied: Local<Vec<MailWeave>>,
    mut mail: ResMut<MailMaterials>,
    drapes: Query<(&DrapeMesh, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let weaves: Vec<_> = studio
        .recipe
        .inventory
        .items()
        .iter()
        .filter_map(|item| match &item.article {
            Article::Draped(selection) => Some(selection.mail),
            Article::Catalog(_) | Article::Plate(_) => None,
        })
        .collect();
    if *applied == weaves {
        return;
    }
    for (drape, handle) in &drapes {
        if let Some(selection) = draped_selection(&studio, drape.0)
            && selection.fabric == FabricPreset::Chainmail
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            *material = mail.material(&mut images, selection.mail);
        }
    }
    *applied = weaves;
}

fn draped_selection(
    studio: &Studio,
    id: InventoryItemId,
) -> Option<&adventuresim_character_creator::garment::GarmentSelection> {
    match &studio.recipe.inventory.get(id)?.article {
        Article::Draped(selection) => Some(selection),
        Article::Catalog(_) | Article::Plate(_) => None,
    }
}

use super::*;
use adventuresim_character_creator::{
    garment::{DrapeInput, DrapedGarment},
    garment_material::{CLOTH_PBR, MailMaps, MailWeave},
    inventory::InventoryItemId,
};
use drape_worker::DrapeWorker;

/// Recent weaves keep their materials while a chainmail slider is dragged.
const MAIL_MATERIAL_CACHE: usize = 4;

/// Drapes the worn cloth on the character.
#[derive(Resource, Default)]
pub struct DrapeJob {
    worker: DrapeWorker<InventoryItemId>,
    pub ready: Option<Vec<DrapedGarment>>,
    /// Drape again at the next regeneration, even if nothing it lies on
    /// changed.
    again: bool,
}

impl DrapeJob {
    /// Drape worn garments, innermost first.
    pub fn request(&mut self, input: Vec<(InventoryItemId, DrapeInput)>) {
        self.worker.request(input);
        self.ready = None;
    }

    /// Make the next request simulate every stage again, and make one.
    pub fn restart_from_placement(&mut self) {
        self.worker.restart_from_placement();
        self.again = true;
    }

    /// Whether a drape was asked for again since the last request.
    pub fn take_again(&mut self) -> bool {
        std::mem::take(&mut self.again)
    }

    /// Whether a drape is queued or running. Only this holds back animation and
    /// export; a finished drape with problems does not.
    pub fn running(&self) -> bool {
        self.worker.running()
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
    /// The material a garment of `selection` is shown in.
    pub fn material_for(
        &mut self,
        images: &mut Assets<Image>,
        selection: &adventuresim_character_creator::garment::GarmentSelection,
    ) -> StandardMaterial {
        if selection.fabric == FabricPreset::Chainmail {
            self.material(images, selection.mail)
        } else {
            cloth_material()
        }
    }

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
            base_color_texture: textures
                .base_color_png
                .as_deref()
                .map(|bytes| load(bytes, true)),
            normal_map_texture: Some(load(&textures.normal_png, false)),
            occlusion_texture: textures
                .occlusion_png
                .as_deref()
                .map(|bytes| load(bytes, false)),
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
        under_plate: None,
        selection,
        settled: None,
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
    let progress = job.worker.poll();
    if let Some(outcome) = progress.finished {
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
                job.worker.draping().len()
            ),
        };
        job.ready = Some(outcome.garments);
    }
    if progress.started {
        studio.status = "Measuring body and preparing cloth...".into();
    }
    let Some(garments) = progress.preview else {
        return;
    };
    for entity in &old {
        commands.entity(entity).despawn();
    }
    for (&id, garment) in job.worker.draping().iter().zip(garments) {
        if job.ready.is_none() {
            studio.status = format!("Draping: {}", garment.stage);
        }
        let material = match studio
            .recipe
            .inventory
            .get(id)
            .and_then(|item| item.article.garment())
        {
            Some(selection) => mail.material_for(&mut images, selection),
            None => cloth_material(),
        };
        spawn_garment(
            &mut commands,
            &mut meshes,
            &mut materials,
            garment,
            material,
            (CharacterMesh, DrapeMesh(id)),
        );
    }
}

/// Show a draped garment, skinned for the animation preview when it can be.
pub fn spawn_garment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    garment: DrapedGarment,
    material: StandardMaterial,
    marker: impl Bundle,
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
            marker,
            Name::new(garment.name),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
        ))
        .id();
    if has_skin {
        commands.entity(entity).insert(
            animation_preview::ClothSkin::new(
                garment.form,
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
        .filter_map(|item| item.article.garment().map(|selection| selection.mail))
        .collect();
    if *applied == weaves {
        return;
    }
    for (drape, handle) in &drapes {
        if let Some(selection) = studio
            .recipe
            .inventory
            .get(drape.0)
            .and_then(|item| item.article.garment())
            && selection.fabric == FabricPreset::Chainmail
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            *material = mail.material(&mut images, selection.mail);
        }
    }
    *applied = weaves;
}

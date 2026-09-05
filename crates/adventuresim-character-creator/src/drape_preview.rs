use super::*;
use adventuresim_character_creator::garment::{DrapeInput, DrapedGarment, drape_outfit};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Resource, Default)]
pub struct DrapeJob {
    handle: Option<std::thread::JoinHandle<Result<Vec<DrapedGarment>, String>>>,
    pending: Option<Vec<DrapeInput>>,
    cancel: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<Vec<DrapedGarment>>>>,
    pub ready: Option<Vec<DrapedGarment>>,
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
}
#[derive(Component)]
pub struct DrapeMesh;

pub fn input(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    selection: adventuresim_character_creator::garment::GarmentSelection,
) -> DrapeInput {
    let character = &model.mhr.character;
    DrapeInput {
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

pub fn poll(
    mut job: ResMut<DrapeJob>,
    mut studio: ResMut<Studio>,
    mut commands: Commands,
    old: Query<Entity, With<DrapeMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        let result = job
            .handle
            .take()
            .unwrap()
            .join()
            .unwrap_or_else(|_| Err("Drape worker failed".into()));
        if !job.cancel.load(Ordering::Relaxed) {
            match result {
                Ok(garments) => {
                    studio.status = format!(
                        "Draped {} vertices; ready to export",
                        garments.iter().map(|g| g.positions.len()).sum::<usize>()
                    );
                    snapshot = Some(garments.clone());
                    job.ready = Some(garments);
                }
                Err(error) => {
                    studio.status = format!("Draping failed: {error}");
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
        job.handle = Some(std::thread::spawn(move || {
            drape_outfit(input, &cancel, |garments| {
                if !cancel.load(Ordering::Relaxed) {
                    *latest.lock().unwrap() = Some(garments);
                }
            })
            .map_err(|error| format!("{error:#}"))
        }));
        studio.status = "Measuring body and preparing cloth...".into();
    }
    if let Some(garments) = snapshot {
        for entity in &old {
            commands.entity(entity).despawn();
        }
        for garment in garments {
            let cloth_name = garment.name.clone();
            let cloth_faces = garment.faces.clone();
            if job.ready.is_none() {
                studio.status = format!("Draping: step {}", garment.frame);
            }
            let has_skin = garment.indices.len() == garment.positions.len()
                && garment.weights.len() == garment.positions.len();
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, garment.positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, garment.normals.clone())
            .with_inserted_indices(Indices::U32(garment.faces.into_iter().flatten().collect()));
            let entity = commands
                .spawn((
                    CharacterMesh,
                    DrapeMesh,
                    Name::new(garment.name),
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.52, 0.42, 0.28),
                        perceptual_roughness: 0.85,
                        cull_mode: None,
                        double_sided: true,
                        ..default()
                    })),
                ))
                .id();
            if has_skin {
                commands
                    .entity(entity)
                    .insert(animation_preview::ClothSkin::new(
                        cloth_name,
                        garment.positions,
                        garment.normals,
                        cloth_faces,
                        garment.indices,
                        garment.weights,
                    ));
            }
        }
    }
}

use super::*;
use adventuresim_character_creator::garment::{DrapeInput, DrapedGarment, drape};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Resource, Default)]
pub struct DrapeJob {
    handle: Option<std::thread::JoinHandle<Result<DrapedGarment, String>>>,
    pending: Option<DrapeInput>,
    cancel: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<DrapedGarment>>>,
    pub ready: Option<DrapedGarment>,
}
impl Drop for DrapeJob {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl DrapeJob {
    pub fn request(&mut self, input: Option<DrapeInput>) {
        self.cancel.store(true, Ordering::Relaxed);
        self.pending = input;
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
                Ok(garment) => {
                    studio.status = format!(
                        "Draped {} vertices; ready to export",
                        garment.positions.len()
                    );
                    snapshot = Some(garment.clone());
                    job.ready = Some(garment);
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
            drape(input, &cancel, |garment| {
                if !cancel.load(Ordering::Relaxed) {
                    *latest.lock().unwrap() = Some(garment);
                }
            })
            .map_err(|error| format!("{error:#}"))
        }));
        studio.status = "Measuring body and preparing cloth...".into();
    }
    if let Some(garment) = snapshot {
        if job.ready.is_none() {
            studio.status = format!("Draping: step {}", garment.frame);
        }
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, garment.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, garment.normals)
        .with_inserted_indices(Indices::U32(garment.faces.into_iter().flatten().collect()));
        for entity in &old {
            commands.entity(entity).despawn();
        }
        commands.spawn((
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
        ));
    }
}

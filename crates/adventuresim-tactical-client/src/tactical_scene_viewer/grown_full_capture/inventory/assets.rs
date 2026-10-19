//! Diagnostic CPU seals and the normal ShaderBuffer upload-data lifecycle.

use super::*;
use bevy::{
    asset::{Asset, AssetEvent},
    render::storage::ShaderBuffer,
};
use std::sync::Mutex;

#[derive(SystemParam)]
pub(super) struct Stores<'w> {
    meshes: Res<'w, Assets<Mesh>>,
    buffers: Res<'w, Assets<ShaderBuffer>>,
    wood: Option<Res<'w, Assets<GrownWoodMaterial>>>,
    leaves: Option<Res<'w, Assets<GrownLeafMaterial>>>,
}

pub(super) struct Snapshot {
    meshes: Vec<MeshValue>,
    materials: Vec<MaterialValue>,
    buffers: Vec<BufferValue>,
}

struct MeshValue {
    id: AssetId<Mesh>,
    value: Mesh,
}

struct BufferValue {
    id: AssetId<ShaderBuffer>,
    descriptor: ShaderBuffer,
    upload: Mutex<UploadState>,
}

enum MaterialValue {
    Wood {
        id: AssetId<GrownWoodMaterial>,
        value: GrownWoodMaterial,
    },
    Leaves {
        id: AssetId<GrownLeafMaterial>,
        value: GrownLeafMaterial,
    },
}

enum UploadState {
    Pending,
    Extracted,
}

impl Snapshot {
    pub(super) fn capture(world: &World, assets: &GrownFullAssets) -> Result<Self> {
        let mut meshes = Vec::new();
        let mut materials = Vec::new();
        let mut buffers = Vec::new();
        for part in assets.parts() {
            let id = part.mesh().id();
            meshes.push(MeshValue {
                id,
                value: world
                    .resource::<Assets<Mesh>>()
                    .get(id)
                    .context("initial grown mesh")?
                    .clone(),
            });
            match part.material() {
                GrownMaterialHandle::Wood(handle) => materials.push(MaterialValue::Wood {
                    id: handle.id(),
                    value: world
                        .resource::<Assets<GrownWoodMaterial>>()
                        .get(handle)
                        .context("initial grown wood")?
                        .clone(),
                }),
                GrownMaterialHandle::Leaves(handle) => materials.push(MaterialValue::Leaves {
                    id: handle.id(),
                    value: world
                        .resource::<Assets<GrownLeafMaterial>>()
                        .get(handle)
                        .context("initial grown leaves")?
                        .clone(),
                }),
            }
        }
        for source in assets.buffers() {
            let current = world
                .resource::<Assets<ShaderBuffer>>()
                .get(source.handle())
                .context("initial grown source upload")?;
            ensure!(
                current.data.as_deref() == Some(source.bytes()),
                "initial source bytes differ from bank packing"
            );
            buffers.push(BufferValue {
                id: source.handle().id(),
                descriptor: ShaderBuffer {
                    data: None,
                    buffer_description: current.buffer_description.clone(),
                    asset_usage: current.asset_usage,
                    copy_on_resize: current.copy_on_resize,
                },
                upload: Mutex::new(UploadState::Pending),
            });
        }
        Ok(Self {
            meshes,
            materials,
            buffers,
        })
    }

    pub(super) fn check(&self, stores: &Stores, assets: &GrownFullAssets) -> Result<()> {
        for held in &self.meshes {
            let current = stores
                .meshes
                .get(held.id)
                .context("held grown mesh missing")?;
            ensure!(
                current.primitive_topology() == held.value.primitive_topology()
                    && current.indices() == held.value.indices()
                    && current.attributes().eq(held.value.attributes())
                    && current.asset_usage == held.value.asset_usage,
                "held grown mesh content drift"
            );
        }
        for held in &self.materials {
            match held {
                MaterialValue::Wood { id, value } => ensure!(
                    stores
                        .wood
                        .as_ref()
                        .context("grown wood stores missing")?
                        .get(*id)
                        == Some(value),
                    "held grown wood material drift"
                ),
                MaterialValue::Leaves { id, value } => ensure!(
                    stores
                        .leaves
                        .as_ref()
                        .context("grown leaf stores missing")?
                        .get(*id)
                        == Some(value),
                    "held grown leaf material drift"
                ),
            }
        }
        for held in &self.buffers {
            let current = stores
                .buffers
                .get(held.id)
                .context("held grown source buffer missing")?;
            ensure!(
                current.buffer_description == held.descriptor.buffer_description
                    && current.asset_usage == held.descriptor.asset_usage
                    && current.copy_on_resize == held.descriptor.copy_on_resize,
                "held grown source descriptor drift"
            );
            let source = assets
                .buffers()
                .find(|source| source.handle().id() == held.id)
                .context("held source owner missing")?;
            let mut upload = held
                .upload
                .lock()
                .map_err(|_| anyhow::anyhow!("grown upload seal lock"))?;
            match (&current.data, &*upload) {
                (Some(bytes), UploadState::Pending) => ensure!(
                    bytes.as_slice() == source.bytes(),
                    "held initial source upload changed"
                ),
                (None, _) => *upload = UploadState::Extracted,
                (Some(_), UploadState::Extracted) => {
                    anyhow::bail!("held source upload reappeared after extraction")
                }
            }
        }
        Ok(())
    }
}

pub(in super::super) fn watch_mutations(
    mut buffers: MessageReader<AssetEvent<ShaderBuffer>>,
    mut meshes: MessageReader<AssetEvent<Mesh>>,
    mut wood: MessageReader<AssetEvent<GrownWoodMaterial>>,
    mut leaves: MessageReader<AssetEvent<GrownLeafMaterial>>,
    owned: Option<Res<OwnedGrown>>,
    evidence: Res<Evidence>,
    state: Option<Res<super::super::super::capture_state::SceneCaptureState>>,
) -> bevy::prelude::Result {
    let Some(owned) = owned else { return Ok(()) };
    let changed_buffer = buffers
        .read()
        .filter_map(mutation)
        .any(|id| owned.owns_buffer(id));
    let changed_mesh = meshes
        .read()
        .filter_map(mutation)
        .any(|id| owned.owns_mesh(id));
    let changed_wood = wood
        .read()
        .filter_map(mutation)
        .any(|id| owned.owns_material(id.untyped()));
    let changed_leaves = leaves
        .read()
        .filter_map(mutation)
        .any(|id| owned.owns_material(id.untyped()));
    if changed_buffer || changed_mesh || changed_wood || changed_leaves {
        evidence.input_fault(
            state.as_ref().map_or(0, |state| state.view),
            "owned grown asset modified after construction".to_owned(),
        )?;
        return Err("owned grown asset modified after construction".into());
    }
    Ok(())
}

fn mutation<A: Asset>(event: &AssetEvent<A>) -> Option<AssetId<A>> {
    match event {
        AssetEvent::Modified { id } | AssetEvent::Removed { id } | AssetEvent::Unused { id } => {
            Some(*id)
        }
        _ => None,
    }
}

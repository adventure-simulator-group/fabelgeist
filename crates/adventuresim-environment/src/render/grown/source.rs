//! Immutable grown record uploads and their two ordinary material bindings.

use super::{
    GrownBankAssets, GrownBufferRole, GrownLeafMaterial, GrownMaterialHandle, GrownSourceBuffer,
    GrownWoodMaterial,
};
use crate::{
    prepared::{GrownGeometry, TreeBank},
    render::{ForestPart, materials::SpeciesRecord},
};
use anyhow::{Context, Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{renderer::RenderDevice, storage::ShaderBuffer},
};
use std::sync::Arc;

pub(super) struct SourceMaterials {
    wood: Option<Handle<GrownWoodMaterial>>,
    leaves: Option<Handle<GrownLeafMaterial>>,
}

// Raw bytes are handed to the renderer only at this upload boundary.
struct SourceUpload<'a> {
    role: GrownBufferRole,
    bytes: &'a [u8],
}

impl GrownBankAssets {
    /// Retained assets may be reused only in their original presenting world.
    pub fn validate_world(&self, world: &World) -> Result<()> {
        ensure!(
            self.world == world.id(),
            "grown bank assets belong to another world"
        );
        Ok(())
    }

    /// Explicit asset-store lifetime: call once, then reuse this Arc.
    /// This does not perform process-global or device-global deduplication.
    pub fn upload(source: Arc<TreeBank>, world: &mut World) -> Result<Arc<Self>> {
        ensure!(
            world.contains_resource::<Assets<ShaderBuffer>>(),
            "grown bank upload requires renderer assets"
        );
        let library = source.library();
        let species: Vec<SpeciesRecord> = library.species.iter().map(SpeciesRecord::from).collect();
        let uploads = [
            SourceUpload {
                role: GrownBufferRole::Wood,
                bytes: bytemuck::cast_slice(&library.branches),
            },
            SourceUpload {
                role: GrownBufferRole::Leaves,
                bytes: bytemuck::cast_slice(&library.leaves),
            },
            SourceUpload {
                role: GrownBufferRole::Species,
                bytes: bytemuck::cast_slice(&species),
            },
            SourceUpload {
                role: GrownBufferRole::Plants,
                bytes: bytemuck::cast_slice(&library.plants),
            },
        ];
        for upload in &uploads {
            if !upload.bytes.is_empty() {
                upload.byte_size(world)?;
            }
        }
        let mut buffers = Vec::new();
        for upload in uploads {
            if !upload.bytes.is_empty() {
                buffers.push(GrownSourceBuffer::upload(upload.role, upload.bytes, world)?);
            }
        }
        Ok(Arc::new(Self {
            source,
            world: world.id(),
            buffers,
        }))
    }
}

impl SourceMaterials {
    pub fn from_geometry(
        geometry: &GrownGeometry,
        bank_assets: &GrownBankAssets,
        world: &mut World,
    ) -> Result<Self> {
        let encoding = geometry.encoding();
        let library = bank_assets.bank().library();
        let model = library
            .models
            .get(geometry.variant().index())
            .context("grown material model")?;
        let shape = geometry.level().shape(&library.detail);
        let packed_shape = UVec4::new(shape.sides, shape.spans, shape.blades, 0);
        let buffers = &bank_assets.buffers;
        let species_buffer = buffer_handle(buffers, GrownBufferRole::Species)?;
        let wood = if model.branches.count > 0 {
            Some(
                world
                    .resource_mut::<Assets<GrownWoodMaterial>>()
                    .add(GrownWoodMaterial {
                        shape: packed_shape,
                        records: buffer_handle(buffers, GrownBufferRole::Wood)?,
                        species: species_buffer.clone(),
                        plants: buffer_handle(buffers, GrownBufferRole::Plants)?,
                        encoding,
                    }),
            )
        } else {
            None
        };
        let leaves = if model.leaves.count > 0 {
            Some(
                world
                    .resource_mut::<Assets<GrownLeafMaterial>>()
                    .add(GrownLeafMaterial {
                        shape: packed_shape,
                        records: buffer_handle(buffers, GrownBufferRole::Leaves)?,
                        species: species_buffer,
                        encoding,
                    }),
            )
        } else {
            None
        };
        Ok(Self { wood, leaves })
    }

    pub fn material(&self, role: ForestPart) -> Result<GrownMaterialHandle> {
        match role {
            ForestPart::Branches => Ok(GrownMaterialHandle::Wood(
                self.wood.as_ref().context("grown wood material")?.clone(),
            )),
            ForestPart::Leaves => Ok(GrownMaterialHandle::Leaves(
                self.leaves.as_ref().context("grown leaf material")?.clone(),
            )),
        }
    }
}

impl SourceUpload<'_> {
    // Native GPU buffer-size boundary; no allocation precedes this check.
    fn byte_size(&self, world: &World) -> Result<u64> {
        let limits = world
            .get_resource::<RenderDevice>()
            .context("grown material render device")?
            .limits();
        let size = u64::try_from(self.bytes.len())?;
        ensure!(
            size > 0
                && size <= limits.max_buffer_size
                && size <= limits.max_storage_buffer_binding_size,
            "grown material source exceeds device storage limit"
        );
        Ok(size)
    }
}

impl GrownSourceBuffer {
    fn upload(role: GrownBufferRole, bytes: &[u8], world: &mut World) -> Result<Self> {
        let size = SourceUpload { role, bytes }.byte_size(world)?;
        let mut asset = ShaderBuffer::new(bytes, RenderAssetUsages::RENDER_WORLD);
        asset.buffer_description.size = size;
        let handle = world.resource_mut::<Assets<ShaderBuffer>>().add(asset);
        Ok(Self { role, handle })
    }
}

pub(super) fn buffer_handle(
    buffers: &[GrownSourceBuffer],
    role: GrownBufferRole,
) -> Result<Handle<ShaderBuffer>> {
    buffers
        .iter()
        .find(|buffer| buffer.role == role)
        .map(|buffer| buffer.handle.clone())
        .context("grown source buffer missing")
}

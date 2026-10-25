//! Card drawing retains canonical topology and checked native vertex/index ports.

use super::{HairGpuError, HairRenderer, Result};
use crate::cards::{CardLayout, CardMesh, CardVertex};
use fabelgeist_gpu::prelude::{BufferByteLength, BufferPrefixError};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[derive(Clone)]
pub(super) struct GpuMesh {
    pub vertices: Arc<wgpu::Buffer>,
    pub indices: Arc<wgpu::Buffer>,
}

pub(super) struct CardDrawMesh {
    layout: CardLayout,
    mesh: GpuMesh,
}

/// The native mesh port that failed admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshBufferRole {
    Vertices,
    Indices,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderMeshError {
    #[error("cards belong to another native device")]
    ForeignDevice,
    #[error("card buffer does not contain its logical data prefix")]
    Prefix(#[from] BufferPrefixError),
    #[error("{role:?} needs {required} bytes, native buffer has {physical}")]
    Extent {
        role: MeshBufferRole,
        required: BufferByteLength,
        physical: BufferByteLength,
    },
    #[error("{role:?} needs {required} bytes, device buffer limit is {limit}")]
    Limit {
        role: MeshBufferRole,
        required: BufferByteLength,
        limit: BufferByteLength,
    },
    #[error("{role:?} requires {required:?}, native usage is {actual:?}")]
    Usage {
        role: MeshBufferRole,
        required: wgpu::BufferUsages,
        actual: wgpu::BufferUsages,
    },
}

impl CardDrawMesh {
    pub fn layout(&self) -> CardLayout {
        self.layout
    }

    pub fn record<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_vertex_buffer(
            0,
            self.mesh
                .vertices
                .slice(..u64::from(self.layout.vertices_binding_bytes())),
        );
        pass.set_index_buffer(
            self.mesh
                .indices
                .slice(..u64::from(self.layout.indices_binding_bytes())),
            wgpu::IndexFormat::Uint32,
        );
        pass.draw_indexed(0..self.layout.corners().native_elements(), 0, 0..1);
    }
}

impl HairRenderer {
    pub(super) fn upload_cards(&self, source: &CardMesh) -> Result<CardDrawMesh> {
        let layout = source.layout();
        self.validate_mesh_limit(MeshBufferRole::Vertices, layout.vertices_binding_bytes())?;
        self.validate_mesh_limit(MeshBufferRole::Indices, layout.indices_binding_bytes())?;
        let empty_vertex = [CardVertex::default()];
        let vertices = if source.vertices().is_empty() {
            &empty_vertex
        } else {
            source.vertices()
        };
        let indices = if source.indices().is_empty() {
            &[0u32]
        } else {
            source.indices()
        };
        let upload = |label, contents, usage| {
            Arc::new(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(label),
                        contents,
                        usage,
                    }),
            )
        };
        Ok(CardDrawMesh {
            layout,
            mesh: GpuMesh {
                vertices: upload(
                    "hair cards",
                    bytemuck::cast_slice(vertices),
                    wgpu::BufferUsages::VERTEX,
                ),
                indices: upload(
                    "hair card indices",
                    bytemuck::cast_slice(indices),
                    wgpu::BufferUsages::INDEX,
                ),
            },
        })
    }

    pub(super) fn receive_gpu_cards(&self, source: &crate::gpu::GpuCards) -> Result<CardDrawMesh> {
        if source.device() != self.device.as_ref() {
            return Err(RenderMeshError::ForeignDevice.into());
        }
        let layout = source.layout();
        let vertices = source
            .vertices()
            .prefix(layout.vertices_bytes())
            .map_err(RenderMeshError::from)?;
        let indices = source
            .indices()
            .prefix(layout.indices_bytes())
            .map_err(RenderMeshError::from)?;
        self.validate_mesh_buffer(
            MeshBufferRole::Vertices,
            &vertices.buffer,
            layout.vertices_binding_bytes(),
        )?;
        self.validate_mesh_buffer(
            MeshBufferRole::Indices,
            &indices.buffer,
            layout.indices_binding_bytes(),
        )?;
        Ok(CardDrawMesh {
            layout,
            mesh: GpuMesh {
                vertices: vertices.buffer,
                indices: indices.buffer,
            },
        })
    }

    fn validate_mesh_buffer(
        &self,
        role: MeshBufferRole,
        buffer: &wgpu::Buffer,
        required: BufferByteLength,
    ) -> Result<()> {
        self.validate_mesh_limit(role, required)?;
        let physical = BufferByteLength::from(buffer.size());
        if physical < required {
            return Err(RenderMeshError::Extent {
                role,
                required,
                physical,
            }
            .into());
        }
        let required = match role {
            MeshBufferRole::Vertices => wgpu::BufferUsages::VERTEX,
            MeshBufferRole::Indices => wgpu::BufferUsages::INDEX,
        };
        let actual = buffer.usage();
        if !actual.contains(required) {
            return Err(RenderMeshError::Usage {
                role,
                required,
                actual,
            }
            .into());
        }
        Ok(())
    }

    fn validate_mesh_limit(&self, role: MeshBufferRole, required: BufferByteLength) -> Result<()> {
        let limit = BufferByteLength::from(self.device.limits().max_buffer_size);
        if required > limit {
            return Err(RenderMeshError::Limit {
                role,
                required,
                limit,
            }
            .into());
        }
        Ok(())
    }
}

impl From<RenderMeshError> for HairGpuError {
    fn from(cause: RenderMeshError) -> Self {
        Self::Backend(cause.into())
    }
}

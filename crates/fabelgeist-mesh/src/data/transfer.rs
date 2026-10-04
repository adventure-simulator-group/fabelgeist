use super::MeshData;
use crate::{GpuMesh, MeshAttribute, MeshReadbackError, MeshUploadError};
use fabelgeist_gpu::prelude::{
    Buffer, BufferDefinition, BufferUpload, BufferUse, ReadbackError, WgpuContext,
};

impl MeshData {
    pub fn upload(&self, context: &WgpuContext) -> Result<GpuMesh, MeshUploadError> {
        self.validate()?;
        let vertex = || -> BufferDefinition {
            BufferDefinition::storage()
                .with_usage(BufferUse::Vertex)
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
        };
        Ok(GpuMesh {
            positions: MeshAttribute::Positions.upload(
                context,
                BufferUpload::from_elements(&self.positions),
                vertex(),
            )?,
            normals: MeshAttribute::Normals.upload(
                context,
                BufferUpload::from_elements(&self.normals),
                vertex(),
            )?,
            tex_coords: MeshAttribute::TextureCoordinates.upload(
                context,
                BufferUpload::from_elements(&self.tex_coords),
                vertex(),
            )?,
            indices: match &self.indices {
                Some(values) => Some(
                    MeshAttribute::Indices.upload(
                        context,
                        BufferUpload::from_elements(values),
                        BufferDefinition::storage()
                            .with_usage(BufferUse::Index)
                            .with_usage(BufferUse::CopySource)
                            .with_usage(BufferUse::CopyDestination),
                    )?,
                ),
                None => None,
            },
            joints: match &self.joints {
                Some(values) => Some(MeshAttribute::Joints.upload(
                    context,
                    BufferUpload::from_elements(values),
                    vertex(),
                )?),
                None => None,
            },
            weights: match &self.weights {
                Some(values) => Some(MeshAttribute::Weights.upload(
                    context,
                    BufferUpload::from_elements(values),
                    vertex(),
                )?),
                None => None,
            },
            vertex_count: crate::DrawVertexCount::from(self.positions.len() as u32),
            topology: self.topology,
            front_face: self.front_face,
        })
    }
}

impl GpuMesh {
    pub async fn readback(&self, context: &WgpuContext) -> Result<MeshData, MeshReadbackError> {
        let mut mesh = MeshData {
            positions: self.positions.read(context).await.map_err(
                |source: ReadbackError| -> MeshReadbackError {
                    MeshReadbackError {
                        attribute: MeshAttribute::Positions,
                        source,
                    }
                },
            )?,
            normals: self.normals.read(context).await.map_err(
                |source: ReadbackError| -> MeshReadbackError {
                    MeshReadbackError {
                        attribute: MeshAttribute::Normals,
                        source,
                    }
                },
            )?,
            tex_coords: self.tex_coords.read(context).await.map_err(
                |source: ReadbackError| -> MeshReadbackError {
                    MeshReadbackError {
                        attribute: MeshAttribute::TextureCoordinates,
                        source,
                    }
                },
            )?,
            indices: match &self.indices {
                Some(v) => Some(v.read(context).await.map_err(
                    |source: ReadbackError| -> MeshReadbackError {
                        MeshReadbackError {
                            attribute: MeshAttribute::Indices,
                            source,
                        }
                    },
                )?),
                None => None,
            },
            joints: match &self.joints {
                Some(v) => Some(v.read(context).await.map_err(
                    |source: ReadbackError| -> MeshReadbackError {
                        MeshReadbackError {
                            attribute: MeshAttribute::Joints,
                            source,
                        }
                    },
                )?),
                None => None,
            },
            weights: match &self.weights {
                Some(v) => Some(v.read(context).await.map_err(
                    |source: ReadbackError| -> MeshReadbackError {
                        MeshReadbackError {
                            attribute: MeshAttribute::Weights,
                            source,
                        }
                    },
                )?),
                None => None,
            },
            topology: self.topology,
            front_face: self.front_face,
        };
        let count = usize::from(self.vertex_count);
        mesh.positions.truncate(count);
        mesh.normals.truncate(count);
        mesh.tex_coords.truncate(count);
        if let Some(joints) = &mut mesh.joints {
            joints.truncate(count);
        }
        if let Some(weights) = &mut mesh.weights {
            weights.truncate(count);
        }
        Ok(mesh)
    }
}

impl MeshAttribute {
    // Empty attributes retain the original backing while drawing no vertices.
    fn upload(
        self,
        context: &WgpuContext,
        payload: BufferUpload<'_>,
        definition: BufferDefinition,
    ) -> Result<Buffer, MeshUploadError> {
        let result = match payload.occupancy() {
            fabelgeist_gpu::prelude::BufferUploadOccupancy::Empty => {
                Buffer::empty_result(context, 16u64.into(), definition)
            }
            fabelgeist_gpu::prelude::BufferUploadOccupancy::Populated => {
                Buffer::from_upload(context, payload, definition)
            }
        };
        result.map_err(
            |source: fabelgeist_gpu::prelude::BufferCreationError| -> MeshUploadError {
                MeshUploadError::allocation(self, source)
            },
        )
    }
}

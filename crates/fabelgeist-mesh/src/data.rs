use crate::{FrontFace, GpuMesh, PrimitiveTopology};
use anyhow::{Result, ensure};
use fabelgeist_gpu::prelude::{BufferCreationResult, BufferUpload, BufferUse};
use fabelgeist_gpu::{
    data::gpu::buffer::{Buffer, BufferDefinition},
    globals::WgpuContext,
};

// wgpu requires nonzero allocation sizes. Buffer::size remains the logical
// attribute length, so an empty mesh draws no vertices and roundtrips as empty.
fn upload<T: bytemuck::NoUninit>(
    context: &WgpuContext,
    data: &[T],
    definition: BufferDefinition,
) -> BufferCreationResult<Buffer> {
    if data.is_empty() {
        let mut buffer = Buffer::new(context, (16u64).into(), definition)?;
        buffer.size = 0u64.into();
        Ok(buffer)
    } else {
        Buffer::from_upload(context, BufferUpload::from_elements(data), definition)
    }
}
async fn read<T: bytemuck::AnyBitPattern>(
    buffer: &Buffer,
    context: &WgpuContext,
) -> Result<Vec<T>> {
    if u64::from(buffer.size) == 0 {
        Ok(vec![])
    } else {
        buffer.read(context).await
    }
}

/// CPU mesh attributes. Empty normal/UV arrays mean those attributes are absent.
/// Indices are optional; an unindexed mesh uses consecutive vertices.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub indices: Option<Vec<u32>>,
    pub joints: Option<Vec<[u32; 4]>>,
    pub weights: Option<Vec<[f32; 4]>>,
    pub topology: PrimitiveTopology,
    pub front_face: FrontFace,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            positions: vec![],
            normals: vec![],
            tex_coords: vec![],
            indices: None,
            joints: None,
            weights: None,
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Ccw,
        }
    }
}

impl MeshData {
    pub fn validate(&self) -> Result<()> {
        let n = self.positions.len();
        ensure!(n <= u32::MAX as usize, "too many mesh vertices");
        ensure!(
            self.positions.iter().flatten().all(|x| x.is_finite()),
            "non-finite mesh position"
        );
        ensure!(
            self.normals.is_empty() || self.normals.len() == n,
            "normal count must match positions"
        );
        ensure!(
            self.tex_coords.is_empty() || self.tex_coords.len() == n,
            "UV count must match positions"
        );
        ensure!(
            self.normals
                .iter()
                .flatten()
                .chain(self.tex_coords.iter().flatten())
                .all(|x| x.is_finite()),
            "non-finite mesh attribute"
        );
        ensure!(
            self.joints.is_some() == self.weights.is_some(),
            "joints and weights must be supplied together"
        );
        if let Some(j) = &self.joints {
            ensure!(j.len() == n, "joint count must match positions");
        }
        if let Some(w) = &self.weights {
            ensure!(
                w.len() == n && w.iter().flatten().all(|x| x.is_finite() && *x >= 0.0),
                "invalid skin weights"
            );
        }
        if let Some(indices) = &self.indices {
            ensure!(
                indices.iter().all(|i| (*i as usize) < n),
                "mesh index out of bounds"
            );
        }
        let count = self.indices.as_ref().map_or(n, Vec::len);
        match self.topology {
            PrimitiveTopology::TriangleList => {
                ensure!(count.is_multiple_of(3), "triangle list needs triples")
            }
            PrimitiveTopology::LineList => {
                ensure!(count.is_multiple_of(2), "line list needs pairs")
            }
            _ => {}
        }
        Ok(())
    }

    /// Expand triangle lists and strips into a consistent triangle list.
    pub fn triangles(&self) -> Result<Vec<[u32; 3]>> {
        self.validate()?;
        let indices = self
            .indices
            .clone()
            .unwrap_or_else(|| (0..self.positions.len() as u32).collect());
        let triangles = match self.topology {
            PrimitiveTopology::TriangleList => indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|t| [t[0], t[1], t[2]])
                .collect(),
            PrimitiveTopology::TriangleStrip => indices
                .windows(3)
                .enumerate()
                .filter_map(|(i, t)| {
                    if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                        return None;
                    }
                    Some(if i % 2 == 0 {
                        [t[0], t[1], t[2]]
                    } else {
                        [t[1], t[0], t[2]]
                    })
                })
                .collect(),
            _ => anyhow::bail!("operation requires triangle geometry"),
        };
        Ok(triangles)
    }

    pub fn upload(&self, context: &WgpuContext) -> Result<GpuMesh> {
        self.validate()?;
        let vertex = || {
            BufferDefinition::storage()
                .with_usage(BufferUse::Vertex)
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
        };
        Ok(GpuMesh {
            positions: upload(context, &self.positions, vertex())?,
            normals: upload(context, &self.normals, vertex())?,
            tex_coords: upload(context, &self.tex_coords, vertex())?,
            indices: self
                .indices
                .as_ref()
                .map(|v| {
                    upload(
                        context,
                        v,
                        BufferDefinition::storage()
                            .with_usage(BufferUse::Index)
                            .with_usage(BufferUse::CopySource)
                            .with_usage(BufferUse::CopyDestination),
                    )
                })
                .transpose()?,
            joints: self
                .joints
                .as_ref()
                .map(|v| upload(context, v, vertex()))
                .transpose()?,
            weights: self
                .weights
                .as_ref()
                .map(|v| upload(context, v, vertex()))
                .transpose()?,
            vertex_count: self.positions.len() as u32,
            topology: self.topology,
            front_face: self.front_face,
        })
    }
}

impl GpuMesh {
    pub async fn readback(&self, context: &WgpuContext) -> Result<MeshData> {
        let mut mesh = MeshData {
            positions: read(&self.positions, context).await?,
            normals: read(&self.normals, context).await?,
            tex_coords: read(&self.tex_coords, context).await?,
            indices: match &self.indices {
                Some(v) => Some(read(v, context).await?),
                None => None,
            },
            joints: match &self.joints {
                Some(v) => Some(read(v, context).await?),
                None => None,
            },
            weights: match &self.weights {
                Some(v) => Some(read(v, context).await?),
                None => None,
            },
            topology: self.topology,
            front_face: self.front_face,
        };
        let count = self.vertex_count as usize;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn gpu_roundtrip_handles_empty_optional_and_skinned_attributes() -> Result<()> {
        let context = WgpuContext::new().await?;
        let position_only = MeshData {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            ..Default::default()
        };
        let skinned = MeshData {
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tex_coords: vec![[0.0; 2]; 3],
            indices: Some(vec![0, 1, 2]),
            joints: Some(vec![[0; 4]; 3]),
            weights: Some(vec![[1.0, 0.0, 0.0, 0.0]; 3]),
            ..position_only.clone()
        };
        for mesh in [
            MeshData::default(),
            MeshData {
                indices: Some(vec![]),
                ..Default::default()
            },
            position_only,
            skinned,
        ] {
            let gpu = mesh.upload(&context)?;
            assert_eq!(gpu.readback(&context).await?, mesh);
        }
        Ok(())
    }
    #[test]
    fn rejects_invalid_attributes_and_indices() {
        let mut mesh = MeshData {
            positions: vec![[0.0; 3]; 3],
            indices: Some(vec![0, 1, 3]),
            ..Default::default()
        };
        assert!(mesh.validate().is_err());
        mesh.indices = Some(vec![0, 1, 2]);
        mesh.normals = vec![[0.0; 3]];
        assert!(mesh.validate().is_err());
    }
    #[test]
    fn strip_alternates_winding() {
        let mesh = MeshData {
            positions: vec![[0.0; 3]; 4],
            topology: PrimitiveTopology::TriangleStrip,
            ..Default::default()
        };
        assert_eq!(mesh.triangles().unwrap(), vec![[0, 1, 2], [2, 1, 3]]);
    }
}

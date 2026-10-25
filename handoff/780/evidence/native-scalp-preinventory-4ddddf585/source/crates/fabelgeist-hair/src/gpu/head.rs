//! The immutable head source and its native coverage resources.

use super::buffers::upload;
use super::error::Result;
use super::GpuGroomer;
use crate::surface::{Surface, SurfaceRenderLayout};
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferDefinition, BufferUse, WgpuContext};
use fabelgeist_math::Vec3;
use std::sync::Arc;

/// GPU coverage for the exact shared surface that produced its vertex order.
pub struct GpuHeadCover {
    source: Arc<Surface>,
    buffer: Buffer,
    device: wgpu::Device,
}

/// The head's mesh on the GPU, for measuring how much hair covers it.
pub(super) struct HeadMesh {
    pub(super) source: Arc<Surface>,
    pub(super) layout: SurfaceRenderLayout,
    pub(super) points: Buffer,
    pub(super) normals: Buffer,
    pub(super) adjacency_start: Buffer,
    pub(super) adjacency: Buffer,
}

impl GpuHeadCover {
    pub fn source(&self) -> &Arc<Surface> {
        &self.source
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}

impl HeadMesh {
    pub(super) fn allocate_cover(&self, context: &WgpuContext) -> Result<GpuHeadCover> {
        let bytes = BufferByteLength::from(
            u64::from(self.layout.vertices().native_elements()) * size_of::<[f32; 2]>() as u64,
        );
        let mut buffer = Buffer::new(
            context,
            bytes.max((size_of::<[f32; 2]>() as u64).into()),
            BufferDefinition::storage()
                .with_usage(BufferUse::Vertex)
                .with_label("hair head cover".into()),
        )?;
        buffer.size = bytes;
        Ok(GpuHeadCover {
            source: self.source.clone(),
            buffer,
            device: context.device.as_ref().clone(),
        })
    }
}

impl GpuGroomer {
    /// The head the hair is drawn on, to measure how much hair covers each
    /// of its vertices ([`crate::gpu::GpuLods::cover`]): its points, normals, and which
    /// vertices neighbour which.
    pub fn set_head_mesh(&mut self, surface: Arc<Surface>) -> Result<()> {
        let layout = SurfaceRenderLayout::try_from(surface.as_ref()).map_err(anyhow::Error::new)?;
        let n = usize::from(layout.vertices());
        let mut neighbors: Vec<Vec<u32>> = vec![Vec::new(); n];
        for t in &surface.triangles {
            for &a in t {
                for &b in t {
                    if a != b && !neighbors[a as usize].contains(&b) {
                        neighbors[a as usize].push(b);
                    }
                }
            }
        }
        let mut start = Vec::with_capacity(n + 1);
        let mut adjacency = Vec::new();
        for list in &neighbors {
            start.push(u32::try_from(adjacency.len()).map_err(anyhow::Error::new)?);
            adjacency.extend_from_slice(list);
        }
        start.push(u32::try_from(adjacency.len()).map_err(anyhow::Error::new)?);
        let four = |v: &[Vec3]| {
            v.iter()
                .map(|p| [p.x, p.y, p.z, 0.0])
                .collect::<Vec<[f32; 4]>>()
        };
        self.head_mesh = Some(HeadMesh {
            layout,
            points: upload(
                &self.context,
                &four(&surface.positions),
                "hair head points".into(),
            )?,
            normals: upload(
                &self.context,
                &four(&surface.normals),
                "hair head normals".into(),
            )?,
            adjacency_start: upload(&self.context, &start, "hair head adjacency start".into())?,
            adjacency: upload(&self.context, &adjacency, "hair head adjacency".into())?,
            source: surface,
        });
        Ok(())
    }
}

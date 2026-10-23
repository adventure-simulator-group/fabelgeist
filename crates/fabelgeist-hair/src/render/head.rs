//! Head geometry and coverage retained for one immutable surface capture.

use super::{GpuMesh, HairRenderer};
use crate::gpu::GpuHeadCover;
use crate::gpu::error::{HairGpuError, Result};
use crate::lod::HairLods;
use crate::surface::{Surface, SurfaceRenderError, SurfaceRenderLayout};
use fabelgeist_gpu::prelude::BufferByteLength;
use std::sync::Arc;
use wgpu::util::DeviceExt;

type HeadResult<T> = std::result::Result<T, RenderSourceError>;

#[derive(Clone)]
pub(super) struct HeadDraw {
    pub source: Arc<Surface>,
    layout: SurfaceRenderLayout,
    pub mesh: GpuMesh,
    pub cover: Arc<wgpu::Buffer>,
}

/// A head or its coverage cannot be associated with this renderer's scene.
#[derive(Debug, thiserror::Error)]
pub enum RenderSourceError {
    #[error("head render layout is invalid")]
    Layout(#[from] SurfaceRenderError),
    #[error("the GPU groom belongs to another native device")]
    ForeignGroomDevice,
    #[error("head coverage belongs to another native device")]
    ForeignCoverDevice,
    #[error("GPU head coverage requires its head to be installed first")]
    MissingHead,
    #[error("GPU head coverage belongs to another surface capture")]
    ForeignHeadSource,
    #[error(
        "head coverage needs {expected} logical bytes and {required_physical} physical bytes; received {logical} logical and {physical} physical bytes"
    )]
    CoverExtent {
        expected: BufferByteLength,
        required_physical: BufferByteLength,
        logical: BufferByteLength,
        physical: BufferByteLength,
    },
}

impl HeadDraw {
    fn new(device: &wgpu::Device, source: Arc<Surface>) -> HeadResult<Self> {
        let layout = SurfaceRenderLayout::try_from(source.as_ref())?;
        // Admission proves exact pairing and every triangle's membership.
        let vertices: Vec<[f32; 6]> = source
            .positions
            .iter()
            .zip(&source.normals)
            .map(|(point, normal)| [point.x, point.y, point.z, normal.x, normal.y, normal.z])
            .collect();
        let indices: Vec<u32> = source.triangles.iter().flatten().copied().collect();
        // Native bindings stay nonempty even for an empty logical head.
        let vertex_data: &[[f32; 6]] = if vertices.is_empty() {
            &[[0.0; 6]]
        } else {
            &vertices
        };
        let index_data: &[u32] = if indices.is_empty() { &[0] } else { &indices };
        let mesh = GpuMesh {
            vertices: Arc::new(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("hair head"),
                    contents: bytemuck::cast_slice(vertex_data),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            ),
            indices: Arc::new(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("hair head indices"),
                    contents: bytemuck::cast_slice(index_data),
                    usage: wgpu::BufferUsages::INDEX,
                }),
            ),
            count: layout.corners().native_elements(),
        };
        let bare = vec![[0.0; 2]; usize::from(layout.vertices()).max(1)];
        Ok(Self {
            source,
            layout,
            mesh,
            cover: Self::coverage_buffer(device, &bare),
        })
    }

    pub(super) fn with_gpu_cover(
        &self,
        device: &wgpu::Device,
        cover: Option<&GpuHeadCover>,
    ) -> HeadResult<Self> {
        let buffer = if let Some(cover) = cover {
            if cover.device() != device {
                return Err(RenderSourceError::ForeignCoverDevice);
            }
            if !Arc::ptr_eq(&self.source, cover.source()) {
                return Err(RenderSourceError::ForeignHeadSource);
            }
            let expected = BufferByteLength::from(
                u64::from(self.layout.vertices().native_elements()) * size_of::<[f32; 2]>() as u64,
            );
            let required_physical = expected.max((size_of::<[f32; 2]>() as u64).into());
            let logical = cover.buffer().size;
            let physical = BufferByteLength::from(cover.buffer().buffer.size());
            if logical != expected || physical < required_physical {
                return Err(RenderSourceError::CoverExtent {
                    expected,
                    required_physical,
                    logical,
                    physical,
                });
            }
            cover.buffer().buffer.clone()
        } else {
            let bare = vec![[0.0; 2]; usize::from(self.layout.vertices()).max(1)];
            Self::coverage_buffer(device, &bare)
        };
        Ok(Self {
            cover: buffer,
            ..self.clone()
        })
    }

    /// Measure this exact paired surface, retaining the existing smoothing.
    pub(super) fn with_cpu_cover(&self, device: &wgpu::Device, lods: &HairLods) -> Self {
        let mut cover = lods.cover.at(&self.source.positions, &self.source.normals);
        let n = cover.len();
        let mut sum = vec![[0.0f32; 2]; n];
        let mut count = vec![0.0f32; n];
        for _ in 0..3 {
            sum.iter_mut().for_each(|sum| *sum = [0.0; 2]);
            count.iter_mut().for_each(|count| *count = 0.0);
            for triangle in &self.source.triangles {
                for &a in triangle {
                    for &b in triangle {
                        let (a, b) = (a as usize, b as usize);
                        sum[a][0] += cover[b][0];
                        sum[a][1] += cover[b][1];
                        count[a] += 1.0;
                    }
                }
            }
            for i in 0..n {
                if count[i] > 0.0 {
                    cover[i] = [sum[i][0] / count[i], sum[i][1] / count[i]];
                }
            }
        }
        if cover.is_empty() {
            cover.push([0.0; 2]);
        }
        Self {
            cover: Self::coverage_buffer(device, &cover),
            ..self.clone()
        }
    }

    fn coverage_buffer(device: &wgpu::Device, cover: &[[f32; 2]]) -> Arc<wgpu::Buffer> {
        Arc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("hair head cover"),
                contents: bytemuck::cast_slice(cover),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
        )
    }
}

impl HairRenderer {
    /// Install an immutable head capture after native topology admission.
    ///
    /// Coverage starts bare. Allocation failures follow the device's configured
    /// error handling. A malformed layout leaves the previous head installed.
    pub fn set_head(&mut self, surface: Arc<Surface>) -> Result<()> {
        let head = HeadDraw::new(&self.device, surface)?;
        self.head = Some(head);
        self.invalidate_view();
        Ok(())
    }
}

impl From<RenderSourceError> for HairGpuError {
    fn from(cause: RenderSourceError) -> Self {
        Self::Backend(cause.into())
    }
}

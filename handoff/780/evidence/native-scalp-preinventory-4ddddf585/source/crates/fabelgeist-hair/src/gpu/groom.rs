//! Native groom ownership, uploads, and exact logical host reads.

use super::buffers::{storage, upload};
use super::error::{HairGpuError, Result};
use super::{GpuGroomer, GpuGuides, StrandGpu, StrandLayout};
use crate::head::Head;
use crate::identity::{PartingGroup, StrandId};
use crate::strands::{Attachment, Strands};
use fabelgeist_gpu::prelude::{Buffer, WgpuContext};
use fabelgeist_math::Vec3;

/// A groom whose native buffers and cardinality were produced together.
pub struct GpuGroom {
    pub(super) layout: StrandLayout,
    pub(super) device: wgpu::Device,
    /// Strand-major positions, one native vec4 per point.
    pub(super) positions: Buffer,
    pub(super) attributes: Buffer,
    pub(super) roots: Buffer,
    pub(super) bounds: Option<[Vec3; 2]>,
    pub(super) root_bounds: [Vec3; 2],
    pub(super) head: Option<Head>,
    pub(super) guides: Option<GpuGuides>,
}

/// Owned destination storage for a native transform of a checked layout.
/// The buffers retain their allocating device and cannot be replaced separately.
pub struct GpuGroomTarget {
    layout: StrandLayout,
    device: wgpu::Device,
    positions: Buffer,
    attributes: Buffer,
    roots: Buffer,
}

impl GpuGroomTarget {
    pub fn layout(&self) -> StrandLayout {
        self.layout
    }

    pub fn positions(&self) -> &Buffer {
        &self.positions
    }

    pub fn attributes(&self) -> &Buffer {
        &self.attributes
    }

    pub fn roots(&self) -> &Buffer {
        &self.roots
    }

    /// Finish a native producer after submitting its writes.
    /// `root_bounds` is its root-grid box in the destination head frame.
    /// Actual point bounds are measured when the groom is prepared for drawing.
    pub fn into_groom(self, root_bounds: [Vec3; 2]) -> GpuGroom {
        GpuGroom {
            layout: self.layout,
            device: self.device,
            positions: self.positions,
            attributes: self.attributes,
            roots: self.roots,
            bounds: None,
            root_bounds,
            head: None,
            guides: None,
        }
    }
}

impl GpuGroom {
    pub fn layout(&self) -> StrandLayout {
        self.layout
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn positions(&self) -> &Buffer {
        &self.positions
    }

    pub fn attributes(&self) -> &Buffer {
        &self.attributes
    }

    pub fn roots(&self) -> &Buffer {
        &self.roots
    }

    pub fn bounds(&self) -> Option<[Vec3; 2]> {
        self.bounds
    }

    pub fn root_bounds(&self) -> [Vec3; 2] {
        self.root_bounds
    }

    pub fn head(&self) -> Option<&Head> {
        self.head.as_ref()
    }

    pub fn guides(&self) -> Option<&GpuGuides> {
        self.guides.as_ref()
    }

    /// The strands back on the host, with their exact logical population.
    pub fn read(&self, context: &WgpuContext) -> Result<Strands> {
        self.ensure_device(context)?;
        read_strands(context, &self.positions, &self.attributes, self.layout)
    }

    pub(super) fn ensure_device(&self, context: &WgpuContext) -> Result<()> {
        if &self.device != context.device.as_ref() {
            return Err(HairGpuError::DeviceMismatch);
        }
        Ok(())
    }
}

impl GpuGroomer {
    /// Allocate a destination for native kernels on this groomer's device.
    pub fn allocate(&self, layout: StrandLayout) -> Result<GpuGroomTarget> {
        Ok(GpuGroomTarget {
            layout,
            device: self.context.device.as_ref().clone(),
            positions: storage(&self.context, layout.positions_bytes(), "hair positions".into())?,
            attributes: storage(&self.context, layout.attributes_bytes(), "hair strands".into())?,
            roots: storage(&self.context, layout.roots_bytes(), "hair roots".into())?,
        })
    }

    /// Strands already grown -- a DiffLocks sample, say -- onto the GPU, to
    /// be prepared for drawing like a groom grown there.
    pub fn upload(&self, strands: &Strands, head: Option<&Head>) -> Result<GpuGroom> {
        let layout = StrandLayout::try_from(strands)?;
        let n = layout.strands().count();
        let positions: Vec<[f32; 4]> = strands
            .positions
            .iter()
            .map(|p| [p.x, p.y, p.z, 0.0])
            .collect();
        let attributes: Vec<StrandGpu> = (0..n)
            .map(|s| {
                let a = strands.attachment(s);
                StrandGpu {
                    normal: a.normal.to_array(),
                    id: (a.id).word(),
                    uv: a.uv,
                    group: (a.group).gpu_word(),
                    radius: a.radius,
                }
            })
            .collect();
        let roots: Vec<[f32; 4]> = (0..n)
            .map(|s| {
                let r = strands.root(s);
                [r.x, r.y, r.z, 1.0]
            })
            .collect();
        let bounds = if n == 0 {
            [Vec3::default(); 2]
        } else {
            strands.bounds()
        };
        let (mut root_lo, mut root_hi) = if n == 0 {
            (Vec3::default(), Vec3::default())
        } else {
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY))
        };
        for s in 0..n {
            root_lo = root_lo.min(strands.root(s));
            root_hi = root_hi.max(strands.root(s));
        }
        Ok(GpuGroom {
            layout,
            device: self.context.device.as_ref().clone(),
            positions: upload(&self.context, &positions, "hair positions".into())?,
            attributes: upload(&self.context, &attributes, "hair strands".into())?,
            roots: upload(&self.context, &roots, "hair roots".into())?,
            bounds: Some(bounds),
            root_bounds: [root_lo, root_hi],
            head: head.cloned(),
            guides: None,
        })
    }

}

/// Exact logical strand data, independent of padded storage bindings.
pub(super) fn read_strands(
    context: &WgpuContext,
    positions: &Buffer,
    attributes: &Buffer,
    layout: StrandLayout,
) -> Result<Strands> {
    let count = layout.strands().count();
    let points = layout.points().count();
    if count == 0 {
        return Ok(Strands {
            points,
            ..Default::default()
        });
    }
    let positions = positions.prefix(layout.positions_bytes()).map_err(anyhow::Error::new)?;
    let attributes = attributes.prefix(layout.attributes_bytes()).map_err(anyhow::Error::new)?;
    let positions: Vec<[f32; 4]> = pollster::block_on(positions.read(context))?;
    let attributes: Vec<StrandGpu> = pollster::block_on(attributes.read(context))?;
    let mut strands = Strands::with_capacity(points, count);
    let mut strand = vec![Vec3::default(); points];
    for s in 0..count {
        for (j, p) in strand.iter_mut().enumerate() {
            let q = positions[s * points + j];
            *p = Vec3::new(q[0], q[1], q[2]);
        }
        let a = attributes[s];
        strands.push(
            &strand,
            Attachment {
                uv: a.uv,
                normal: Vec3::from_array(a.normal),
                id: StrandId::from_word(a.id),
                group: PartingGroup::from_gpu_word(a.group),
                radius: a.radius,
            },
        );
    }
    Ok(strands)
}


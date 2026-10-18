//! Hair on the GPU kept off a body of the caller's choosing: the
//! mannequin's shapes, or any body sampled as a distance field
//! ([`FieldHeader`]) -- a character's, say -- in the `head_shape` buffer
//! every colliding kernel reads.

use crate::gpu::error::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_math::Vec3;

use super::grow::{PingPong, StrandSet, params};
use super::{GpuGroom, GpuGroomer, GpuLods, head_shape, storage, upload};
use crate::lod::LodSettings;
use crate::ops::{Op, STANDARD_GRAVITY};

/// A distance field sampled on a grid, as the kernels' `head_shape` reads
/// it in place of the mannequin's shapes: three entries of header, then the
/// distances, four to an entry, `x` fastest, at the centres of cells `cell`
/// metres wide from `lo`. Outside the grid everything is `reach` from the
/// body; a kernel that collides with the field collides with whatever body
/// it was sampled from.
#[derive(Clone, Copy, Debug)]
pub struct FieldHeader {
    pub lo: Vec3,
    pub cell: f32,
    pub dims: [u32; 3],
    pub reach: f32,
}

impl FieldHeader {
    /// Entries of header before the distances.
    pub const ENTRIES: usize = 3;

    /// The header's entries.
    pub fn entries(&self) -> [[f32; 4]; Self::ENTRIES] {
        let [nx, ny, nz] = self.dims.map(|n| n as f32);
        [
            [self.lo.x, self.lo.y, self.lo.z, 1.0],
            [self.cell, nx, ny, nz],
            [self.reach, 0.0, 0.0, 0.0],
        ]
    }

    /// Cells in the grid.
    pub fn cells(&self) -> usize {
        self.dims.iter().map(|&n| n as usize).product()
    }

    /// Entries the whole field takes, header and distances.
    pub fn len(&self) -> usize {
        Self::ENTRIES + self.cells().div_ceil(4)
    }

    /// Whether the grid has no cells.
    pub fn is_empty(&self) -> bool {
        self.cells() == 0
    }
}

impl GpuGroomer {
    /// Prepare `groom` for drawing: its strands sorted into the shuffled
    /// order every level shares, occlusion baked, every level of
    /// `settings`, the cards, and the head's cover; `center` is the head's.
    pub fn lods(
        &mut self,
        groom: &GpuGroom,
        center: Vec3,
        settings: &LodSettings,
    ) -> Result<GpuLods> {
        let body = match groom.head.as_ref() {
            Some(head) => Some(upload(
                &self.context,
                &head_shape(head),
                "hair head shape".into(),
            )?),
            None => None,
        };
        self.lods_against(groom, center, settings, body.as_ref())
    }
}
impl GpuGroomer {
    /// Run `ops` -- any but [`Op::Interpolate`] -- over `groom`'s strands
    /// where they lie, colliding with `body`: shapes as `head_shape` lays
    /// them out, or a sampled field ([`super::FieldHeader`]). Recorded and
    /// submitted; nothing waits.
    pub fn apply(&mut self, groom: &GpuGroom, ops: &[Op], body: &Buffer) -> Result<()> {
        let context = self.context.clone();
        let (count, points) = (groom.strands, groom.points);
        let bytes = count as u64 * points as u64 * 16;
        let mut set = StrandSet {
            positions: PingPong {
                own: groom.positions.clone(),
                scratch: self.scratch.positions.fit(
                    &context,
                    (bytes).into(),
                    "hair positions scratch".into(),
                )?,
                flipped: false,
            },
            attributes: groom.attributes.clone(),
            roots: groom.roots.clone(),
            count,
            points,
            root_bounds: groom.root_bounds,
            guide_of: self.scratch.guide_of.fit(
                &context,
                (count as u64 * 4).into(),
                "hair guide map".into(),
            )?,
            mapped: false,
        };
        let mut batch = KernelBatch::labelled(&context, "hair operations");
        self.record_ops(&mut batch, ops, &mut set, body)?;
        set.settle(&mut batch)?;
        batch.submit();
        Ok(())
    }

    /// Where each of `count` strands of `points` points in `positions`
    /// comes to rest under `load` against `body`, as [`crate::ops::Settle`]
    /// solves it before blending: `settle`'s stiffness, iterations and
    /// collision offset. A fresh buffer, as `positions` lays them out;
    /// recorded and submitted, nothing waits.
    pub fn equilibrium(
        &mut self,
        positions: &Buffer,
        count: u32,
        points: u32,
        settle: &crate::ops::Settle,
        load: Vec3,
        body: &Buffer,
    ) -> Result<Buffer> {
        let context = self.context.clone();
        let bytes = count as u64 * points as u64 * 16;
        let solved = storage(&context, (bytes).into(), "hair equilibrium".into())?;
        let moments = self.scratch.settle_moments.fit(
            &context,
            (bytes).into(),
            "hair drape moments".into(),
        )?;
        let dirs = self.scratch.settle_dirs.fit(
            &context,
            (bytes).into(),
            "hair drape directions".into(),
        )?;
        let compliance = 1.0 / (STANDARD_GRAVITY * settle.stiffness.max(1e-3).powi(3));
        let mut batch = KernelBatch::labelled(&context, "hair equilibrium");
        batch.dispatch_folded(
            &self.kernels.settle_solve,
            &params! {
                "src" => positions.clone(),
                "solved" => solved.clone(),
                "moments" => moments,
                "dirs" => dirs,
                "head_shape" => body.clone(),
                "count" => count,
                "points" => points,
                "iterations" => settle.iterations as u32,
                "pad0" => 0u32,
                "load_x" => load.x,
                "load_y" => load.y,
                "load_z" => load.z,
                "compliance" => compliance,
                "collision_offset" => settle.collision_offset,
                "pad1" => 0u32,
                "pad2" => 0u32,
                "pad3" => 0u32,
            },
            count,
        )?;
        batch.submit();
        Ok(solved)
    }
}

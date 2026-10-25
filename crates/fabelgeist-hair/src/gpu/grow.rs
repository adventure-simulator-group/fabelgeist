//! [`GpuGroomer::grow`]: a groom's stack on the GPU, from its guides on.

use crate::gpu::error::{HairGpuError, Result};
use crate::parameters::StrandCount;
use fabelgeist_compute::{BufferCopy, KernelBatch};
use fabelgeist_gpu::prelude::BufferByteOffset;
use fabelgeist_compute::ScanElementCount;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_math::Vec3;

use super::guides::Guides;
use super::{GpuGroom, GpuGroomer, GpuScalp, StrandGpu, StrandLayout, StrandLayoutError, head_shape, storage, upload};
use crate::follicles::Growth;
use crate::groom::Groom;
use crate::head::Head;
use crate::look::Look;
use crate::ops::{DensityMask, Interpolate, Op, Parting, STANDARD_GRAVITY};
use crate::rng;

mod input;
mod maps;
mod operations;
mod profile;

/// Parameters by name, as the compute kernels bind them.
macro_rules! params {
    ($($key:literal => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut p = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
        $(p.insert($key, $value);)*
        p
    }};
}
pub(super) use params;

/// The two position buffers a stack ping-pongs between: the strands' own,
/// and a scratch one.
pub(super) struct PingPong {
    pub(super) own: Buffer,
    pub(super) scratch: Buffer,
    /// The current positions are in `scratch`.
    pub(super) flipped: bool,
}

/// Strands a stack runs over -- the guides before interpolation, the
/// groom after -- and what its operations share.
pub(super) struct StrandSet {
    pub(super) positions: PingPong,
    pub(super) attributes: Buffer,
    /// Each strand's root, for the guide maps.
    pub(super) roots: Buffer,
    pub(super) layout: StrandLayout,
    /// A box the roots are inside.
    pub(super) root_bounds: [Vec3; 2],
    /// The guide map, and whether one has been made.
    pub(super) guide_of: Buffer,
    pub(super) mapped: bool,
}

impl PingPong {
    fn src(&self) -> Buffer {
        if self.flipped {
            self.scratch.clone()
        } else {
            self.own.clone()
        }
    }

    fn dst(&self) -> Buffer {
        if self.flipped {
            self.own.clone()
        } else {
            self.scratch.clone()
        }
    }

    fn swap(&mut self) {
        self.flipped = !self.flipped;
    }
}

impl StrandSet {
    /// The positions back in the strands' own buffer.
    pub(super) fn settle(&mut self, batch: &mut KernelBatch) -> Result<()> {
        if self.positions.flipped {
            batch.copy_buffer(
                &self.positions.scratch,
                &self.positions.own,
                BufferCopy {
                    source_offset: BufferByteOffset::START,
                    destination_offset: BufferByteOffset::START,
                    bytes: self.layout.positions_bytes(),
                },
            )?;
            self.positions.flipped = false;
        }
        Ok(())
    }
}

impl GpuGroomer {
    /// Grow `look` on `head` (its hairline's, [`Look::head`]), rooted on
    /// `scalp` (that head's, uploaded): the guides and the whole groom, on
    /// the GPU.
    pub fn grow_look(&mut self, look: &Look, head: &Head, scalp: &GpuScalp) -> Result<GpuGroom> {
        self.grow(
            &look.groom(head),
            Guides::Hairdo {
                hairdo: &look.hairdo,
                variation: look.variation.ratio(),
                seed: look.seed.word() as u64,
            },
            head,
            scalp,
        )
    }

    /// Grow `groom` from `guides` on `head` (its hairline's), rooted on
    /// `scalp`. Everything runs on the GPU: the guides (grown there, or
    /// brought), the operations before interpolation on them, interpolation
    /// and everything after. The roots' and guides' counts are read back
    /// once, to size what follows; nothing else waits.
    pub fn grow(
        &mut self,
        groom: &Groom,
        guides: Guides,
        head: &Head,
        scalp: &GpuScalp,
    ) -> Result<GpuGroom> {
        let requested_points = crate::parameters::PointsPerStrand::from_count(groom.points.max(2));
        let points = u32::try_from(requested_points.count()).map_err(|source| {
            StrandLayoutError::PointCount { requested: requested_points, source }
        })?;
        let input::Interpolation { at, interpolate } = input::Interpolation::find(groom)?;
        let context = self.context.clone();
        let head_shape = upload(&context, &head_shape(head), "hair head shape".into())?;

        // Roots for the guides and for the strands, then how many of each.
        let mut batch = KernelBatch::labelled(&context, "hair roots");
        let guide_roots = match &guides {
            Guides::Hairdo { hairdo, seed, .. } => {
                Some(self.record_guide_roots(&mut batch, hairdo, *seed, scalp)?)
            }
            Guides::Host(_) => None,
        };
        let (n, candidates, normals, keep) = self.record_roots(&mut batch, interpolate, scalp)?;
        let mut words = vec![(keep.clone(), n)];
        if let Some(roots) = &guide_roots {
            words.push((roots.counts.clone(), 1));
        }
        let counts = self.read_words(batch, &words)?;
        let count = counts[0];
        if count == 0 {
            return Err(HairGpuError::EmptyRoots);
        }
        if count >= 1 << 20 {
            return Err(HairGpuError::StrandLimit {
                requested: StrandCount::from_count(count as usize),
            });
        }

        let layout = StrandLayout::new(StrandCount::from_count(count as usize), requested_points)?;
        let mut batch = KernelBatch::labelled(&context, "hair groom");
        let guides = match guides {
            Guides::Host(strands) => self.upload_guides(strands, points)?,
            Guides::Hairdo {
                hairdo,
                variation,
                seed,
            } => {
                let roots = guide_roots.as_ref().expect("placed above");
                self.record_hairdo_guides(
                    &mut batch,
                    hairdo,
                    variation,
                    seed,
                    head,
                    roots,
                    counts[1],
                    points,
                    &head_shape,
                )?
            }
        };
        if guides.layout.native_strands() == 0 {
            return Err(HairGpuError::EmptyGuides);
        }

        // The operations before interpolation, on the guides.
        let guide_bytes = guides.layout.positions_bytes();
        let mut guide_set = guides.operation_set(self, scalp.bounds)?;
        self.record_ops(&mut batch, &groom.ops[..at], &mut guide_set, &head_shape)?;
        guide_set.settle(&mut batch)?;

        // The guides as interpolation reads them.
        let guide_points = self.scratch.guide_points.fit(
            &context,
            (guide_bytes).into(),
            "hair guide points".into(),
        )?;
        let guide_info = self.scratch.guide_info.fit(
            &context,
            (guides.layout.native_strands() as u64 * 16).into(),
            "hair guide info".into(),
        )?;
        self.record_guide_pack(
            &mut batch,
            &guides,
            &guide_points,
            &guide_info,
            &interpolate.parting,
        )?;

        // The strands' roots, compacted.
        let roots = storage(&context, (count as u64 * 16).into(), "hair roots".into())?;
        let attributes = storage(
            &context,
            (count as u64 * std::mem::size_of::<StrandGpu>() as u64).into(),
            "hair strands".into(),
        )?;
        let mut compact = params! {
            "cand_position" => candidates.clone(),
            "cand_normal" => normals.clone(),
            "keep" => keep.clone(),
            "roots" => roots.clone(),
            "strands" => attributes.clone(),
            "count" => interpolate.count as u32,
            "id_seed" => rng::seed(interpolate.seed, 0x51D),
            "radius" => interpolate.radius,
        };
        for (key, value) in parting_parameters(&interpolate.parting) {
            compact.insert(key, value);
        }
        batch.dispatch_folded(
            &self.kernels.roots_compact,
            &compact,
            interpolate.count as u32,
        )?;

        // Interpolation.
        let neighbors = self.scratch.neighbors.fit(
            &context,
            (count as u64 * 32).into(),
            "hair neighbours".into(),
        )?;
        let weights = self.scratch.weights.fit(
            &context,
            (count as u64 * 32).into(),
            "hair weights".into(),
        )?;
        batch.dispatch_folded(
            &self.kernels.interp_select,
            &params! {
                "roots" => roots.clone(),
                "strands" => attributes.clone(),
                "guide_points" => guide_points.clone(),
                "guide_info" => guide_info.clone(),
                "neighbors" => neighbors.clone(),
                "weights" => weights.clone(),
                "count" => count,
                "guides" => guides.layout.native_strands(),
                "points" => points,
                "wanted" => interpolate.neighbors.clamp(1, 8) as u32,
                "divergence" => interpolate.divergence,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            layout.strand_invocations(),
        )?;
        let bytes = layout.positions_bytes();
        let mut strands = StrandSet {
            positions: PingPong {
                own: storage(&context, (bytes).into(), "hair positions".into())?,
                scratch: self.scratch.positions.fit(
                    &context,
                    (bytes).into(),
                    "hair positions scratch".into(),
                )?,
                flipped: false,
            },
            attributes: attributes.clone(),
            roots: roots.clone(),
            layout,
            root_bounds: scalp.bounds,
            guide_of: self.scratch.guide_of.fit(
                &context,
                (count as u64 * 4).into(),
                "hair guide map".into(),
            )?,
            mapped: false,
        };
        batch.dispatch_folded(
            &self.kernels.interp_points,
            &params! {
                "roots" => roots.clone(),
                "strands" => attributes.clone(),
                "guide_points" => guide_points,
                "guide_info" => guide_info,
                "neighbors" => neighbors,
                "weights" => weights,
                "dst" => strands.positions.own.clone(),
                "count" => count,
                "points" => points,
                "follow" => u32::from(interpolate.follow_surface_normal),
                "pad0" => 0u32,
                "follow_length" => interpolate.follow_length,
                "pad1" => 0u32,
                "pad2" => 0u32,
                "pad3" => 0u32,
            },
            layout.sample_invocations(),
        )?;

        // Everything after.
        self.record_ops(&mut batch, &groom.ops[at + 1..], &mut strands, &head_shape)?;
        strands.settle(&mut batch)?;
        batch.submit()?;

        Ok(GpuGroom {
            layout,
            device: context.device.as_ref().clone(),
            positions: strands.positions.own,
            attributes,
            roots,
            bounds: None,
            root_bounds: scalp.bounds,
            head: Some(head.clone()),
            guides: Some(guides),
        })
    }

    /// The candidate roots over the scalp and which are kept, scanned into
    /// strand slots: how many is in `keep`'s last word.
    fn record_roots(
        &mut self,
        batch: &mut KernelBatch,
        interpolate: &Interpolate,
        scalp: &GpuScalp,
    ) -> Result<(u32, Buffer, Buffer, Buffer)> {
        let context = self.context.clone();
        let n = interpolate.count.max(1) as u32;
        // How far in from the hairline each scalp vertex is.
        let extra = self.scratch.scalp_extra.fit(
            &context,
            (scalp.vertices as u64 * 16).into(),
            "hair scalp uv and edge".into(),
        )?;
        batch.dispatch_folded(
            &self.kernels.edge_distances,
            &params! {
                "scalp_points" => scalp.points.clone(),
                "scalp_uvs" => scalp.uvs.clone(),
                "boundary" => scalp.boundary_fine.clone(),
                "extra" => extra.clone(),
                "count" => scalp.vertices,
                "boundary_count" => scalp.boundary_fine_count,
                "edge" => interpolate.edge,
                "pad0" => 0u32,
            },
            scalp.vertices,
        )?;

        let (balding, [crown, temples, top]) = match &interpolate.density {
            DensityMask::Balding {
                crown,
                temples,
                top,
            } => (1u32, [*crown, *temples, *top]),
            _ => (0, [0.0; 3]),
        };
        let seed = rng::seed(interpolate.seed, 0x1A7E);
        let mask_seed = rng::seed(interpolate.seed, 0xD35E);
        let offset = rng::signed_vec3(mask_seed, 0x9A7C) * 50.0;

        let candidates = self.scratch.candidates.fit(
            &context,
            (n as u64 * 32).into(),
            "hair root candidates".into(),
        )?;
        let candidate_normals = self.scratch.normals.fit(
            &context,
            (n as u64 * 16).into(),
            "hair root candidate normals".into(),
        )?;
        let keep = self.scratch.keep.fit(
            &context,
            ((n as u64 + 1) * 4).into(),
            "hair roots kept".into(),
        )?;
        batch.clear_buffer(&keep);
        batch.dispatch_folded(
            &self.kernels.roots,
            &params! {
                "tri_cdf" => scalp.tri_cdf.clone(),
                "tri_index" => scalp.tri_index.clone(),
                "scalp_points" => scalp.points.clone(),
                "scalp_normals" => scalp.normals.clone(),
                "scalp_extra" => extra,
                "cand_position" => candidates.clone(),
                "cand_normal" => candidate_normals.clone(),
                "keep" => keep.clone(),
                "count" => n,
                "triangles" => scalp.triangles,
                "seed" => seed,
                "balding" => balding,
                "total_area" => scalp.area.max(1e-9),
                "crown" => crown,
                "temples" => temples,
                "top" => top,
                "patchiness" => interpolate.patchiness,
                "edge" => interpolate.edge,
                "offset_x" => offset.x,
                "offset_y" => offset.y,
                "offset_z" => offset.z,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            n,
        )?;
        self.ensure_scan(n + 1)?;
        self.scan.record(
            batch,
            &keep,
            &self.scan_scratch,
            ScanElementCount::from_elements(n + 1),
        )?;
        Ok((n, candidates, candidate_normals, keep))
    }

}

/// A parting as the kernels' parameters.
pub(super) fn parting_parameters(
    parting: &Parting,
) -> Vec<(
    &'static str,
    fabelgeist_gpu::data::gpu::parameters::PassParameter,
)> {
    let (kind, lanes, plane, until, front, half_width) = match parting {
        Parting::None => (0u32, 0u32, [0.0f32; 4], 0.0f32, 0.0f32, 0.0f32),
        Parting::Plane {
            normal,
            offset,
            until,
        } => (
            1,
            0,
            [normal[0], normal[1], normal[2], *offset],
            *until,
            0.0,
            0.0,
        ),
        Parting::Lanes { count } => (2, *count, [0.0; 4], 0.0, 0.0, 0.0),
        Parting::Fringe { front, half_width } => (3, 0, [0.0; 4], 0.0, *front, *half_width),
    };
    vec![
        ("parting", kind.into()),
        ("lanes", lanes.into()),
        ("plane_x", plane[0].into()),
        ("plane_y", plane[1].into()),
        ("plane_z", plane[2].into()),
        ("plane_offset", plane[3].into()),
        ("until", until.into()),
        ("fringe_front", front.into()),
        ("fringe_half_width", half_width.into()),
    ]
}

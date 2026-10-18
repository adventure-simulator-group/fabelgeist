//! The guides a groom grows from, on the GPU: grown there from a hairdo
//! ([`crate::hairdo::Hairdo::guides`]'s kernels), or brought from the host.

use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_math::Vec3;

use super::grow::{params, parting_parameters};
use super::{GpuGroomer, GpuScalp, StrandGpu, read_strands, storage, upload};
use crate::follicles::Growth;
use crate::hairdo::{Hairdo, Tied, edge_spacing, guide_key, guide_spacing};
use crate::head::Head;
use crate::ops::Parting;
use crate::rng;
use crate::strands::Strands;

/// What a groom grows from.
pub enum Guides<'a> {
    /// Guides made elsewhere -- a hairdo's on the host, a sample's, drawn by
    /// hand -- resampled to the groom's points.
    Host(&'a Strands),
    /// A hairdo's guides, grown on the GPU as [`Hairdo::guides`] grows them
    /// on the host, from the same roots and the same random numbers.
    Hairdo {
        hairdo: &'a Hairdo,
        variation: f32,
        seed: u64,
    },
}

/// The guides a groom was grown from, as the operations before
/// interpolation left them, still on the GPU.
pub struct GpuGuides {
    pub count: u32,
    pub points: u32,
    pub positions: Buffer,
    /// [`StrandGpu`] a guide.
    pub attributes: Buffer,
    /// Each guide's root.
    pub(super) roots: Buffer,
}

impl GpuGuides {
    /// The guides back on the host.
    pub fn read(&self, context: &WgpuContext) -> Result<Strands> {
        read_strands(
            context,
            &self.positions,
            &self.attributes,
            self.count,
            self.points,
        )
    }
}

/// The hairdo as the comb kernel reads it: the WGSL `Hairdo` of
/// [`super::wgsl::guides`], field for field.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct HairdoGpu {
    guides: u32,
    points: u32,
    max_path: u32,
    key: u32,
    variation: f32,
    angle: f32,
    angle_spread: f32,
    direction_spread: f32,
    kink: f32,
    growth: u32,
    whorls: u32,
    fringe_falls: u32,
    center: [f32; 3],
    lanes: u32,
    w0: [f32; 3],
    w0_spiral: f32,
    w0_hand: f32,
    w0_reach: f32,
    w1: [f32; 3],
    w1_spiral: f32,
    w1_hand: f32,
    w1_reach: f32,
    top: f32,
    front: f32,
    sides: f32,
    back: f32,
    has_hemline: u32,
    hemline: f32,
    slope: f32,
    clip: f32,
    clipper: f32,
    fade: f32,
    strip: f32,
    has_fringe: u32,
    fringe_width: f32,
    fringe_depth: f32,
    fringe_drop: f32,
    dome: f32,
    flow: u32,
    flow_value: f32,
    part: f32,
    hold: f32,
    lift: f32,
    volume: f32,
    cling: f32,
    tuck: f32,
    hairline: [f32; 3],
    recession: f32,
    peak: f32,
    tied: u32,
    tie: [f32; 3],
    out: [f32; 3],
    axis_len: u32,
}

/// A hairdo's guide roots, placed; how many is read back with the groom's
/// one wait, and the comb recorded after it.
pub(super) struct GuideRoots {
    position: Buffer,
    normal: Buffer,
    uv: Buffer,
    /// How many roots, then how many with the hairline's row.
    pub(super) counts: Buffer,
}

/// The longest a guide of `hairdo` can grow, metres: its longest hair (or
/// its fringe, or out to its dome) as long as the variation makes it.
fn longest_guide(hairdo: &Hairdo, variation: f32) -> f32 {
    let cut = &hairdo.haircut;
    let mut longest = cut.longest();
    if let Some(fringe) = cut.fringe {
        longest = longest.max(0.1 + fringe.drop);
    }
    longest *= 1.0 + variation * 0.24;
    if cut.dome > 0.0 {
        // Out to the dome from anywhere on the scalp, drawn a fifth longer
        // and two spreads more.
        longest = longest.max((cut.dome + 0.16) * 1.4);
    }
    longest
}

impl GpuGroomer {
    /// Place `hairdo`'s guide roots on `scalp`, as the host places them.
    pub(super) fn record_guide_roots(
        &mut self,
        batch: &mut KernelBatch,
        hairdo: &Hairdo,
        seed: u64,
        scalp: &GpuScalp,
    ) -> Result<GuideRoots> {
        let context = self.context.clone();
        let wanted = hairdo.guide_count() as u32;
        let spacing = guide_spacing(scalp.area, wanted as usize);
        let key = guide_key(seed);
        let candidates = wanted * 40;
        let candidate_buffer = self.scratch.guide_candidates.fit(
            &context,
            candidates as u64 * 48,
            "hair guide candidates",
        )?;
        batch.dispatch_folded(
            &self.kernels.guide_candidates,
            &params! {
                "tri_cdf" => scalp.tri_cdf.clone(),
                "tri_index" => scalp.tri_index.clone(),
                "scalp_points" => scalp.points.clone(),
                "scalp_normals" => scalp.normals.clone(),
                "scalp_uvs" => scalp.uvs.clone(),
                "candidates" => candidate_buffer.clone(),
                "count" => candidates,
                "triangles" => scalp.triangles,
                "seed_share" => rng::hash(key, 1),
                "seed_u" => rng::hash(key, 2),
                "seed_v" => rng::hash(key, 3),
                "area" => scalp.area,
                "pad0" => 0u32,
                "pad1" => 0u32,
            },
            candidates,
        )?;
        let boundary = scalp.boundary_count;
        let closest = self.scratch.guide_edges.fit(
            &context,
            boundary.max(1) as u64 * 48,
            "hair guide hairline",
        )?;
        batch.dispatch_folded(
            &self.kernels.edge_closest,
            &params! {
                "boundary" => scalp.boundary.clone(),
                "tri_index" => scalp.tri_index.clone(),
                "scalp_points" => scalp.points.clone(),
                "scalp_normals" => scalp.normals.clone(),
                "scalp_uvs" => scalp.uvs.clone(),
                "closest" => closest.clone(),
                "count" => boundary,
                "triangles" => scalp.triangles,
                "max_distance" => 0.01f32,
                "pad0" => 0u32,
            },
            boundary,
        )?;
        let capacity = (wanted + boundary) as u64;
        let roots = GuideRoots {
            position: storage(&context, capacity * 16, "hair guide roots")?,
            normal: storage(&context, capacity * 16, "hair guide root normals")?,
            uv: storage(&context, capacity * 16, "hair guide root uvs")?,
            counts: storage(&context, 16, "hair guide counts")?,
        };
        batch.dispatch(
            &self.kernels.guide_select,
            &params! {
                "candidates" => candidate_buffer,
                "boundary" => scalp.boundary.clone(),
                "closest" => closest,
                "root_position" => roots.position.clone(),
                "root_normal" => roots.normal.clone(),
                "root_uv" => roots.uv.clone(),
                "counts" => roots.counts.clone(),
                "candidate_count" => candidates,
                "wanted" => wanted,
                "boundary_count" => boundary,
                "pad0" => 0u32,
                "spacing" => spacing,
                "edge_spacing" => edge_spacing(spacing),
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            [1, 1, 1],
        )?;
        Ok(roots)
    }

    /// Comb `count` guides from `roots`, `points` points each.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_hairdo_guides(
        &mut self,
        batch: &mut KernelBatch,
        hairdo: &Hairdo,
        variation: f32,
        seed: u64,
        head: &Head,
        roots: &GuideRoots,
        count: u32,
        points: u32,
        head_shape: &Buffer,
    ) -> Result<GpuGuides> {
        let context = self.context.clone();
        let longest = longest_guide(hairdo, variation);
        let max_path = ((longest / 0.002).ceil() as u32).max(301) + 2;
        let field = hairdo.follicles.field(head);
        let whorl = |i: usize| field.whorls.get(i).copied();
        let (w0, w1) = (whorl(0), whorl(1));
        let (flow, flow_value) = hairdo.flow_code();
        let tied = Tied::new(head, hairdo);
        let axis: Vec<[f32; 4]> = tied.as_ref().map_or(vec![[0.0; 4]; 2], |t| {
            t.axis.iter().map(|p| [p.x, p.y, p.z, 0.0]).collect()
        });
        let cut = &hairdo.haircut;
        let style = &hairdo.hairstyle;
        let follicles = &hairdo.follicles;
        let fringe = cut.fringe.unwrap_or_default();
        let to3 = |v: Vec3| v.to_array();
        let gpu = HairdoGpu {
            guides: count,
            points,
            max_path,
            key: guide_key(seed),
            variation,
            angle: follicles.angle,
            angle_spread: follicles.angle_spread,
            direction_spread: follicles.direction_spread,
            kink: follicles.kink,
            growth: Growth::ALL
                .iter()
                .position(|g| *g == field.growth)
                .unwrap_or(0) as u32,
            whorls: field.whorls.len() as u32,
            fringe_falls: u32::from(hairdo.fringe_falls()),
            center: to3(field.center),
            lanes: match hairdo.parting() {
                Parting::Lanes { count } => count,
                _ => 0,
            },
            w0: w0.map_or([0.0; 3], |w| to3(w.point)),
            w0_spiral: w0.map_or(0.0, |w| w.spiral),
            w0_hand: w0.map_or(1.0, |w| w.hand),
            w0_reach: w0.map_or(0.07, |w| w.reach),
            w1: w1.map_or([0.0; 3], |w| to3(w.point)),
            w1_spiral: w1.map_or(0.0, |w| w.spiral),
            w1_hand: w1.map_or(1.0, |w| w.hand),
            w1_reach: w1.map_or(0.07, |w| w.reach),
            top: cut.top,
            front: cut.front,
            sides: cut.sides,
            back: cut.back,
            has_hemline: u32::from(cut.hemline.is_some()),
            hemline: cut.hemline.unwrap_or(0.0),
            slope: cut.slope,
            clip: cut.clip,
            clipper: cut.clipper,
            fade: cut.fade,
            strip: cut.strip,
            has_fringe: u32::from(cut.fringe.is_some()),
            fringe_width: fringe.width,
            fringe_depth: fringe.depth,
            fringe_drop: fringe.drop,
            dome: cut.dome,
            flow,
            flow_value,
            part: style.part,
            hold: style.hold,
            lift: style.lift,
            volume: style.volume,
            cling: style.cling,
            tuck: style.tuck,
            hairline: head.hairline,
            recession: head.recession,
            peak: head.peak,
            tied: u32::from(tied.is_some()),
            tie: tied.as_ref().map_or([0.0; 3], |t| to3(t.point)),
            out: tied.as_ref().map_or([0.0; 3], |t| to3(t.out)),
            axis_len: axis.len() as u32,
        };
        let path = self.scratch.guide_paths.fit(
            &context,
            count as u64 * max_path as u64 * 16,
            "hair guide paths",
        )?;
        let guides = GpuGuides {
            count,
            points,
            positions: storage(&context, count as u64 * points as u64 * 16, "hair guides")?,
            attributes: storage(
                &context,
                count as u64 * std::mem::size_of::<StrandGpu>() as u64,
                "hair guide strands",
            )?,
            roots: storage(&context, count as u64 * 16, "hair guide roots")?,
        };
        // A workgroup a guide.
        batch.dispatch(
            &self.kernels.guide_comb,
            &params! {
                "root_position" => roots.position.clone(),
                "root_normal" => roots.normal.clone(),
                "hairdo" => upload(&context, &[gpu], "hair hairdo")?,
                "head_shape" => head_shape.clone(),
                "tie_axis" => upload(&context, &axis, "hair tie")?,
                "path" => path,
                "positions" => guides.positions.clone(),
            },
            [count, 1, 1],
        )?;
        let mut p = params! {
            "root_position" => roots.position.clone(),
            "root_normal" => roots.normal.clone(),
            "root_uv" => roots.uv.clone(),
            "strands" => guides.attributes.clone(),
            "roots" => guides.roots.clone(),
            "count" => count,
            "pad0" => 0u32,
            "radius" => follicles.radius,
        };
        for (key, value) in parting_parameters(&hairdo.parting()) {
            p.insert(key, value);
        }
        batch.dispatch_folded(&self.kernels.guide_attributes, &p, count)?;
        Ok(guides)
    }

    /// Guides from the host, resampled to `points`.
    pub(super) fn upload_guides(&self, strands: &Strands, points: u32) -> Result<GpuGuides> {
        let guides = strands.resampled(points as usize);
        let positions: Vec<[f32; 4]> = guides
            .positions
            .iter()
            .map(|p| [p.x, p.y, p.z, 0.0])
            .collect();
        let attributes: Vec<StrandGpu> = (0..guides.len())
            .map(|s| {
                let a = guides.attachment(s);
                StrandGpu {
                    normal: a.normal.to_array(),
                    id: a.id,
                    uv: a.uv,
                    group: a.group,
                    radius: a.radius,
                }
            })
            .collect();
        let roots: Vec<[f32; 4]> = (0..guides.len())
            .map(|s| {
                let r = guides.root(s);
                [r.x, r.y, r.z, 0.0]
            })
            .collect();
        let context = &self.context;
        Ok(GpuGuides {
            count: guides.len() as u32,
            points,
            positions: upload(context, &positions, "hair guides")?,
            attributes: upload(context, &attributes, "hair guide strands")?,
            roots: upload(context, &roots, "hair guide roots")?,
        })
    }
}

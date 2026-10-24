//! Grooming on the GPU.
//!
//! The groom's stack is embarrassingly parallel -- every operation after
//! interpolation works on each strand alone, and a head is a hundred
//! thousand strands of up to two hundred and fifty-six points -- so it runs
//! as compute kernels, and what it grows stays on the GPU: the renderer
//! draws the levels of detail straight from the buffers the kernels wrote.
//!
//! [`GpuGroomer`] holds the kernels, on the compute layer: one
//! [`KernelBatch`] a groom (every dispatch into one encoder, one submit),
//! [`PrefixSum`] to compact the roots and the cards, its
//! [`RadixSort`] for the shuffled strand order the levels of detail share,
//! and its spatial hash grid for the roots about each vertex of the head.
//! The kernels themselves ([`wgsl`]) mirror the host operations in
//! [`crate::ops`] line for line; the host operations remain, the reference
//! the kernels are checked against and the path for a machine without a
//! GPU.
//!
//! * [`GpuGroomer::grow_look`] grows a [`crate::Look`] whole, and
//!   [`GpuGroomer::grow`] any [`Groom`] from [`Guides`]: a hairdo's, grown
//!   there, or the host's. A hairdo's guide roots are placed as the host
//!   places them -- candidates hashed by index, kept greedily in order by
//!   one workgroup -- and each guide is combed by a workgroup of its own,
//!   its lanes sampling the head's field together and drawing the same
//!   numbers as the host ([`crate::rng::Stream`]), so the guides come out
//!   as the host's to a micrometre or so. The operations before
//!   interpolation (the drape, a solve per strand, with a strand's joints
//!   in workgroup memory; the shrinkwrap) run on the guides, the rest on
//!   the strands.
//! * [`GpuGroomer::cut_scalp`] cuts the scalp they grow on ([`GpuScalp`]):
//!   surface nets over the head's field and the part above the hairline,
//!   to the host's own triangles, in the host's order -- a count, a scan
//!   and an emit for the triangles, vertices numbered by where they are
//!   first used, the running area added up in the host's order, the open
//!   edges found by sorting. [`GpuGroomer::upload_scalp`] takes any other
//!   (DiffLocks' `scalp.ply`, say). Roots are placed by area over the scalp
//!   in one pass -- each triangle its share, spread by a low-discrepancy
//!   sequence -- thinned by the density masks and compacted by a scan.
//!   Clump guides are chosen on a grid, the lowest-hashed strand of each
//!   cell leading it. A strand sampling its guide walks the guide from the
//!   root, transporting its frame as it goes, so no frames are stored.
//! * [`GpuGroomer::lods`] prepares a grown groom for drawing: sorted into
//!   the shuffled order, occlusion baked into a volume and every point,
//!   each strand level resampled (and kept off the head), the cards cut
//!   from the locks, and how much hair covers each vertex of the head.
//! * [`GpuGroomer::simulate`] makes real-time physics for it ([`GpuSim`]):
//!   [`crate::sim::HairSim`]'s rods, bound and stepped where they are
//!   drawn, a substep a dispatch (an invocation a strand solving its own
//!   implicit steps), and meeting each other -- the volume and
//!   friction on a density grid in the head's frame, collisions found by
//!   samples sorted into a grid of cells and resolved segment against
//!   segment. [`GpuGroomer::resimulate`] moves it to another level of
//!   detail, its motion carried across.
//! * [`GpuGroomer::upload`] brings strands made elsewhere -- a DiffLocks
//!   sample, a host groom -- to the GPU, to prepare and draw the same way.
//!
//! A sweep along a strand keeps its points in registers or in the kernel's
//! own output, never in a large local array: dynamically indexed
//! `array<vec3<f32>, 32>`s came out differently from run to run on the same
//! input, and the tests grow every groom twice to hold that down.
//!
//! ```no_run
//! # fn main() -> anyhow::Result<()> {
//! use fabelgeist_hair::gpu::{GpuGroomer, headless_context};
//! use fabelgeist_hair::{Head, LodSettings, Look, Preset};
//!
//! let mut groomer = GpuGroomer::new(headless_context()?)?;
//! let look = Look::new(Preset::Afro);
//! let head = look.head(&Head::mannequin());
//! // The scalp, cut once for as long as the hairline stays.
//! let scalp = groomer.cut_scalp(&head, 0.004)?;
//! let grown = groomer.grow_look(&look, &head, &scalp)?;
//! let lods = groomer.lods(&grown, head.cranium.center, &LodSettings::default())?;
//! // `HairRenderer::set_gpu_groom(&lods)` draws it; `grown.read(..)` has
//! // the strands back on the host, to write out.
//! # Ok(())
//! # }
//! ```

use crate::cards::CardAtlas;
use crate::gpu::error::{HairGpuError, Result};
use crate::head::Head;
use buffers::{Scratch, storage, upload};
pub use collide::FieldHeader;
use fabelgeist_compute::{
    Kernel, KernelBatch, KernelCache, PrefixSum, PrefixSumScratch, RadixSort, ScanElementCount,
    SortItemCount, SortScratch,
};
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_math::Vec3;
use fabelgeist_spatial_hash::gpu::{GpuHashGrid, HashKernels};
use fabelgeist_spatial_hash::{HashLayout, HashPointCount};
use groom::read_strands;
pub use groom::{GpuGroom, GpuGroomTarget};
pub use guides::{GpuGuides, Guides};
pub use head::GpuHeadCover;
use head::HeadMesh;
pub use layout::{StrandLayout, StrandLayoutError, StrandLayoutResult};
pub use lods::{GpuCards, GpuLevel, GpuLods};
pub use scalp::GpuScalp;
pub use sim::GpuSim;
use std::sync::Arc;

mod bounds;
mod buffers;
mod collide;
pub mod error;
mod groom;
mod grow;
mod guides;
mod head;
mod layout;
mod lods;
mod readback;
mod scalp;
mod sim;
pub mod wgsl;

/// Initial element capacity; each scratch grows independently when needed.
const INITIAL_SCRATCH_CAPACITY: u32 = 1024;

/// A strand's per-strand data on the GPU.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StrandGpu {
    pub normal: [f32; 3],
    pub id: u32,
    pub uv: [f32; 2],
    pub group: i32,
    pub radius: f32,
}

struct Kernels {
    roots: Arc<Kernel>,
    roots_compact: Arc<Kernel>,
    interp_select: Arc<Kernel>,
    interp_points: Arc<Kernel>,
    orient: Arc<Kernel>,
    trim: Arc<Kernel>,
    map_candidates: Arc<Kernel>,
    map_assign: Arc<Kernel>,
    clump: Arc<Kernel>,
    curl: Arc<Kernel>,
    wave: Arc<Kernel>,
    noise: Arc<Kernel>,
    frizz: Arc<Kernel>,
    smooth: Arc<Kernel>,
    shrinkwrap: Arc<Kernel>,
    profile: Arc<Kernel>,
    order_keys: Arc<Kernel>,
    splat: Arc<Kernel>,
    densify: Arc<Kernel>,
    occlusion: Arc<Kernel>,
    level: Arc<Kernel>,
    card_accumulate: Arc<Kernel>,
    card_flags: Arc<Kernel>,
    card_spread: Arc<Kernel>,
    card_build: Arc<Kernel>,
    cover_count: Arc<Kernel>,
    cover_finish: Arc<Kernel>,
    cover_smooth: Arc<Kernel>,
    bounds: Arc<Kernel>,
    settle_solve: Arc<Kernel>,
    settle_solve_few: Arc<Kernel>,
    settle_apply: Arc<Kernel>,
    guide_candidates: Arc<Kernel>,
    edge_closest: Arc<Kernel>,
    guide_select: Arc<Kernel>,
    guide_comb: Arc<Kernel>,
    guide_attributes: Arc<Kernel>,
    guide_pack: Arc<Kernel>,
    edge_distances: Arc<Kernel>,
    nets_values: Arc<Kernel>,
    nets_cells: Arc<Kernel>,
    nets_quads_count: Arc<Kernel>,
    nets_quads_emit: Arc<Kernel>,
    nets_first: Arc<Kernel>,
    nets_order: Arc<Kernel>,
    nets_vertices: Arc<Kernel>,
    nets_triangles: Arc<Kernel>,
    nets_cdf: Arc<Kernel>,
    nets_open: Arc<Kernel>,
    nets_boundary: Arc<Kernel>,
    sim_rest: Arc<Kernel>,
    sim_transfer: Arc<Kernel>,
    sim_rest_density: Arc<Kernel>,
    sim_rest_stats: Arc<Kernel>,
    sim_splat: Arc<Kernel>,
    sim_volume: Arc<Kernel>,
    sim_samples: Arc<Kernel>,
    sim_sort_samples: Arc<Kernel>,
    sim_collide: Arc<Kernel>,
    sim_collide_apply: Arc<Kernel>,
    sim_bind: Arc<Kernel>,
    sim_reset: Arc<Kernel>,
    sim_shape: Arc<Kernel>,
    sim_orient: Arc<Kernel>,
    sim_step: Arc<Kernel>,
}

#[derive(Default)]
struct Scratches {
    positions: Scratch,
    moves: Scratch,
    candidates: Scratch,
    normals: Scratch,
    keep: Scratch,
    neighbors: Scratch,
    weights: Scratch,
    guide_of: Scratch,
    leaders: Scratch,
    card_guides: Scratch,
    locks: Scratch,
    flags: Scratch,
    keys: Scratch,
    deposits: Scratch,
    density: Scratch,
    root_counts: Scratch,
    totals: Scratch,
    cover: Scratch,
    word: Scratch,
    box_words: Scratch,
    scalp_extra: Scratch,
    guide_candidates: Scratch,
    guide_edges: Scratch,
    guide_paths: Scratch,
    guide_positions: Scratch,
    guide_map: Scratch,
    guide_points: Scratch,
    guide_info: Scratch,
    settle_moments: Scratch,
    settle_dirs: Scratch,
    settle_under: Scratch,
    settle_standard: Scratch,
    nets_values: Scratch,
    nets_cells: Scratch,
    nets_counts: Scratch,
    nets_triangles: Scratch,
    nets_first: Scratch,
    nets_flags: Scratch,
    nets_cell_at: Scratch,
    nets_vertex_of: Scratch,
    nets_areas: Scratch,
    nets_keys: Scratch,
    nets_ids: Scratch,
    nets_steps: Scratch,
    nets_fine: Scratch,
}

pub struct GpuGroomer {
    context: WgpuContext,
    kernels: Kernels,
    scan: PrefixSum,
    scan_scratch: PrefixSumScratch,
    sort: RadixSort,
    sort_scratch: SortScratch,
    grid: GpuHashGrid,
    scratch: Scratches,
    head_mesh: Option<HeadMesh>,
    atlas: Option<Arc<CardAtlas>>,
}

impl Kernels {
    fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        use wgsl::{groom, guides, lod, scalp, sim};
        let k = |source: String| cache.get(context, &source);
        Ok(Self {
            roots: k(groom::roots())?,
            roots_compact: k(groom::roots_compact())?,
            interp_select: k(groom::interp_select())?,
            interp_points: k(groom::interp_points())?,
            orient: k(groom::orient())?,
            trim: k(groom::trim())?,
            map_candidates: k(groom::map_candidates())?,
            map_assign: k(groom::map_assign())?,
            clump: k(groom::clump())?,
            curl: k(groom::curl())?,
            wave: k(groom::wave())?,
            noise: k(groom::noise())?,
            frizz: k(groom::frizz())?,
            smooth: k(groom::smooth())?,
            shrinkwrap: k(groom::shrinkwrap())?,
            profile: k(groom::profile())?,
            order_keys: k(lod::order_keys())?,
            splat: k(lod::splat())?,
            densify: k(lod::densify())?,
            occlusion: k(lod::occlusion())?,
            level: k(lod::level())?,
            card_accumulate: k(lod::card_accumulate())?,
            card_flags: k(lod::card_flags())?,
            card_spread: k(lod::card_spread())?,
            card_build: k(lod::card_build())?,
            cover_count: k(lod::cover_count())?,
            cover_finish: k(lod::cover_finish())?,
            cover_smooth: k(lod::cover_smooth())?,
            bounds: k(lod::bounds())?,
            settle_solve: k(groom::settle_solve())?,
            settle_solve_few: k(groom::settle_solve_few())?,
            settle_apply: k(groom::settle_apply())?,
            guide_candidates: k(guides::guide_candidates())?,
            edge_closest: k(guides::edge_closest())?,
            guide_select: k(guides::guide_select())?,
            guide_comb: k(guides::guide_comb())?,
            guide_attributes: k(guides::guide_attributes())?,
            guide_pack: k(guides::guide_pack())?,
            edge_distances: k(guides::edge_distances())?,
            nets_values: k(scalp::nets_values())?,
            nets_cells: k(scalp::nets_cells())?,
            nets_quads_count: k(scalp::nets_quads(false))?,
            nets_quads_emit: k(scalp::nets_quads(true))?,
            nets_first: k(scalp::nets_first())?,
            nets_order: k(scalp::nets_order())?,
            nets_vertices: k(scalp::nets_vertices())?,
            nets_triangles: k(scalp::nets_triangles())?,
            nets_cdf: k(scalp::nets_cdf())?,
            nets_open: k(scalp::nets_open())?,
            nets_boundary: k(scalp::nets_boundary())?,
            sim_rest: k(sim::sim_rest())?,
            sim_transfer: k(sim::sim_transfer())?,
            sim_rest_density: k(sim::sim_rest_density())?,
            sim_rest_stats: k(sim::sim_rest_stats())?,
            sim_splat: k(sim::sim_splat())?,
            sim_volume: k(sim::sim_volume())?,
            sim_samples: k(sim::sim_samples())?,
            sim_sort_samples: k(sim::sim_sort_samples())?,
            sim_collide: k(sim::sim_collide())?,
            sim_collide_apply: k(sim::sim_collide_apply())?,
            sim_bind: k(sim::sim_bind())?,
            sim_reset: k(sim::sim_reset())?,
            sim_shape: k(sim::sim_shape())?,
            sim_orient: k(sim::sim_orient())?,
            sim_step: k(sim::sim_step())?,
        })
    }
}

impl GpuGroomer {
    pub fn new(context: WgpuContext) -> Result<Self> {
        let cache = KernelCache::new();
        let kernels = Kernels::new(&context, &cache)?;
        let scan = PrefixSum::with_cache(&context, &cache)?;
        let scan_scratch = PrefixSumScratch::new(
            &context,
            ScanElementCount::from_elements(INITIAL_SCRATCH_CAPACITY),
        )?;
        let sort = RadixSort::with_cache(&context, &cache)?;
        let sort_scratch = SortScratch::new(
            &context,
            SortItemCount::from(INITIAL_SCRATCH_CAPACITY),
        )?;
        let grid = GpuHashGrid::new(
            &context,
            HashKernels::new(&context, &cache)?,
            HashLayout::try_from(HashPointCount::from(INITIAL_SCRATCH_CAPACITY))
                .map_err(anyhow::Error::from)?,
        )?;
        Ok(Self {
            context,
            kernels,
            scan,
            scan_scratch,
            sort,
            sort_scratch,
            grid,
            scratch: Scratches::default(),
            head_mesh: None,
            atlas: None,
        })
    }

    pub fn context(&self) -> &WgpuContext {
        &self.context
    }

    fn ensure_scan(&mut self, count: u32) -> Result<()> {
        self.scan_scratch
            .ensure(&self.context, ScanElementCount::from_elements(count))?;
        Ok(())
    }
}

/// The mannequin's shapes, as the kernels' `head_shape` reads them.
pub fn head_shape(head: &Head) -> Vec<[f32; 4]> {
    let e = |c: Vec3| [c.x, c.y, c.z, 0.0];
    vec![
        e(head.cranium.center),
        e(head.cranium.radii),
        e(head.jaw.center),
        e(head.jaw.radii),
        e(head.nose.center),
        e(head.nose.radii),
        e(head.ears[0].center),
        e(head.ears[0].radii),
        e(head.ears[1].center),
        e(head.ears[1].radii),
        [
            head.neck.a.x,
            head.neck.a.y,
            head.neck.a.z,
            head.neck.radius,
        ],
        e(head.neck.b),
        e(head.shoulders.center),
        e(head.shoulders.radii),
        e(head.chest.center),
        e(head.chest.radii),
        [head.base, 0.0, 0.0, 0.0],
    ]
}

/// A device of its own, for grooming and drawing without a window.
pub fn headless_context() -> Result<WgpuContext> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(HairGpuError::Adapter)?;
        let limits = required_limits(&adapter);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("hair"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: Default::default(),
            })
            .await?;
        Ok(WgpuContext::from_parts(
            Arc::new(instance),
            Arc::new(adapter),
            Arc::new(device),
            Arc::new(queue),
        ))
    })
}

/// The limits a device grooming on the GPU needs beyond wgpu's defaults:
/// the largest buffers the adapter allows -- a full groom's positions are
/// one buffer -- and as many of them to a kernel as it allows.
pub fn required_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
    let adapter_limits = adapter.limits();
    wgpu::Limits {
        max_buffer_size: adapter_limits.max_buffer_size,
        max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
        max_storage_buffers_per_shader_stage: adapter_limits.max_storage_buffers_per_shader_stage,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::{PointsPerStrand, StrandCount};
    use crate::{Look, Preset, Strands};
    use crate::identity::StrandId;
    use fabelgeist_math::units::AccelerationVector;

    fn validate(name: &str, source: &str) {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|e| panic!("{name}: {}", e.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    }

    #[test]
    fn the_kernels_validate() {
        use wgsl::{groom, guides, lod, scalp, sim};
        for (name, source) in [
            ("roots", groom::roots()),
            ("roots_compact", groom::roots_compact()),
            ("interp_select", groom::interp_select()),
            ("interp_points", groom::interp_points()),
            ("orient", groom::orient()),
            ("trim", groom::trim()),
            ("map_candidates", groom::map_candidates()),
            ("map_assign", groom::map_assign()),
            ("clump", groom::clump()),
            ("curl", groom::curl()),
            ("wave", groom::wave()),
            ("noise", groom::noise()),
            ("frizz", groom::frizz()),
            ("smooth", groom::smooth()),
            ("shrinkwrap", groom::shrinkwrap()),
            ("profile", groom::profile()),
            ("order_keys", lod::order_keys()),
            ("splat", lod::splat()),
            ("densify", lod::densify()),
            ("occlusion", lod::occlusion()),
            ("level", lod::level()),
            ("card_accumulate", lod::card_accumulate()),
            ("card_flags", lod::card_flags()),
            ("card_spread", lod::card_spread()),
            ("card_build", lod::card_build()),
            ("cover_count", lod::cover_count()),
            ("cover_finish", lod::cover_finish()),
            ("cover_smooth", lod::cover_smooth()),
            ("bounds", lod::bounds()),
            ("settle_solve", groom::settle_solve()),
            ("settle_solve_few", groom::settle_solve_few()),
            ("settle_apply", groom::settle_apply()),
            ("guide_candidates", guides::guide_candidates()),
            ("edge_closest", guides::edge_closest()),
            ("guide_select", guides::guide_select()),
            ("guide_comb", guides::guide_comb()),
            ("guide_attributes", guides::guide_attributes()),
            ("guide_pack", guides::guide_pack()),
            ("edge_distances", guides::edge_distances()),
            ("nets_values", scalp::nets_values()),
            ("nets_cells", scalp::nets_cells()),
            ("nets_quads_count", scalp::nets_quads(false)),
            ("nets_quads_emit", scalp::nets_quads(true)),
            ("nets_first", scalp::nets_first()),
            ("nets_order", scalp::nets_order()),
            ("nets_vertices", scalp::nets_vertices()),
            ("nets_triangles", scalp::nets_triangles()),
            ("nets_cdf", scalp::nets_cdf()),
            ("nets_open", scalp::nets_open()),
            ("nets_boundary", scalp::nets_boundary()),
            ("sim_rest", sim::sim_rest()),
            ("sim_transfer", sim::sim_transfer()),
            ("sim_rest_density", sim::sim_rest_density()),
            ("sim_rest_stats", sim::sim_rest_stats()),
            ("sim_splat", sim::sim_splat()),
            ("sim_volume", sim::sim_volume()),
            ("sim_samples", sim::sim_samples()),
            ("sim_sort_samples", sim::sim_sort_samples()),
            ("sim_collide", sim::sim_collide()),
            ("sim_collide_apply", sim::sim_collide_apply()),
            ("sim_bind", sim::sim_bind()),
            ("sim_reset", sim::sim_reset()),
            ("sim_shape", sim::sim_shape()),
            ("sim_orient", sim::sim_orient()),
            ("sim_step", sim::sim_step()),
        ] {
            validate(name, &source);
        }
    }

    /// Grown on the GPU, a look comes out as it does on the host: as many
    /// strands, as long, rooted on the scalp, out of the head.
    #[test]
    fn a_groom_grows_on_the_gpu_as_on_the_host() {
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        for (preset, on_gpu) in [
            (Preset::Long, false),
            (Preset::Afro, false),
            (Preset::Bob, false),
            (Preset::Long, true),
            (Preset::Afro, true),
            (Preset::Ponytail, true),
        ] {
            let mut look = Look::new(preset);
            look.strands = StrandCount::from_count(20_000);
            let head = look.head(&Head::mannequin());
            let scalp = head.scalp(0.006);
            let uploaded = groomer.upload_scalp(&scalp).expect("scalp");
            let guides = look.guides(&head, &scalp);
            let host = look.grow(&head, &scalp);
            let grown = if on_gpu {
                groomer.grow_look(&look, &head, &uploaded)
            } else {
                groomer.grow(&look.groom(&head), Guides::Host(&guides), &head, &uploaded)
            }
            .expect("grown");
            let gpu = grown.read(groomer.context()).expect("read back");
            let name = preset.name();
            assert!(!gpu.has_non_finite(), "{name}");
            let ratio = gpu.len() as f32 / host.len() as f32;
            assert!(
                (0.85..1.15).contains(&ratio),
                "{name}: {} vs {}",
                gpu.len(),
                host.len()
            );
            let mean = |s: &Strands| s.lengths().iter().sum::<f32>() / s.len() as f32;
            let (a, b) = (mean(&gpu), mean(&host));
            assert!((a / b - 1.0).abs() < 0.15, "{name}: mean length {a} vs {b}");
            for s in (0..gpu.len()).step_by(97) {
                assert!(
                    head.distance(gpu.root(s)).abs() < 0.003,
                    "{name}: root off the scalp"
                );
            }
            let inside = gpu
                .positions
                .iter()
                .filter(|p| head.distance(**p) < -0.002)
                .count();
            assert!(
                inside * 200 < gpu.positions.len(),
                "{name}: {inside} points inside"
            );

            groomer
                .set_head_mesh(Arc::new(Head::mannequin().mesh(0.008)))
                .expect("head");
            let lods = groomer
                .lods(&grown, head.cranium.center, &crate::LodSettings::default())
                .expect("levels");
            assert_eq!(lods.strand_count().count(), gpu.len());
            assert!(lods.cards.iter().all(|c| c.strips().cards().count() > 0), "{name}: no cards");
            let cover: Vec<[f32; 2]> =
                pollster::block_on(lods.cover.as_ref().expect("cover").buffer().read(groomer.context()))
                    .expect("cover read back");
            let covered = cover.iter().filter(|c| c[0] > 0.5).count();
            assert!(covered > 0, "{name}: nothing covered");
        }
    }

    /// Scratch buffers carry the last groom's values, and a sweep over a
    /// polyline is where a kernel is likeliest to go astray: growing again
    /// gives the same groom, and the coarse levels are kept off the head as
    /// the host keeps them.
    #[test]
    fn growing_again_gives_the_same_groom() {
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        groomer
            .set_head_mesh(Arc::new(Head::mannequin().mesh(0.008)))
            .expect("head");
        for preset in [Preset::Long, Preset::Afro] {
            let mut look = Look::new(preset);
            look.strands = StrandCount::from_count(20_000);
            let head = look.head(&Head::mannequin());
            let scalp = groomer.upload_scalp(&head.scalp(0.006)).expect("scalp");
            let settings = crate::LodSettings::default();
            let mut run = || {
                let grown = groomer.grow_look(&look, &head, &scalp).expect("grown");
                let lods = groomer
                    .lods(&grown, head.cranium.center, &settings)
                    .expect("levels");
                let context = groomer.context();
                let levels: Vec<Vec<[f32; 4]>> = lods
                    .levels
                    .iter()
                    .map(|l| pollster::block_on(l.positions.read(context)).expect("read"))
                    .collect();
                let cards: Vec<Vec<[f32; 4]>> = lods
                    .cards
                    .iter()
                    .map(|c| pollster::block_on(c.vertices.read(context)).expect("read"))
                    .collect();
                (levels, cards)
            };
            let (levels, cards) = run();
            let again = run();
            let name = preset.name();
            assert!(levels == again.0, "{name}: the levels differ");
            assert!(cards == again.1, "{name}: the cards differ");

            // The finest level is the groom; a coarser one is it resampled
            // and kept clear, as on the host.
            let full = levels[0].len() / (levels[1].len() / settings.strand_points[0].count());
            let n = settings.strand_points[0].count();
            for slot in (0..levels[1].len() / n).step_by(211) {
                let strand: Vec<Vec3> = levels[0][slot * full..(slot + 1) * full]
                    .iter()
                    .map(|p| Vec3::new(p[0], p[1], p[2]))
                    .collect();
                let mut host = vec![Vec3::default(); n];
                crate::strands::resample_into(&strand, &mut Vec::new(), &mut host);
                crate::collider::keep_clear(&head, &mut host, 0.0012, 4);
                for (j, h) in host.iter().enumerate() {
                    let p = levels[1][slot * n + j];
                    let off = (Vec3::new(p[0], p[1], p[2]) - *h).length();
                    assert!(
                        off < 2e-4,
                        "{name}: strand {slot} point {j} is {off} off the host's"
                    );
                }
            }
        }
    }

    /// Simulated on the GPU, strands move as the host's simulation moves
    /// them, and every drawn strand is bound to the same simulated ones.
    #[test]
    fn hair_moves_on_the_gpu_as_on_the_host() {
        use crate::sim::{HairSim, Rigid, SimSettings};
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        groomer
            .set_head_mesh(Arc::new(Head::mannequin().mesh(0.008)))
            .expect("head");
        let mut look = Look::new(Preset::Long);
        look.strands = StrandCount::from_count(10_000);
        let head = look.head(&Head::mannequin());
        let scalp = groomer.upload_scalp(&head.scalp(0.006)).expect("scalp");
        let grown = groomer.grow_look(&look, &head, &scalp).expect("grown");
        let lods = groomer
            .lods(&grown, head.cranium.center, &crate::LodSettings::default())
            .expect("levels");
        // The solver alone: the host has no strands meeting each other.
        let settings = SimSettings {
            guides: StrandCount::from_count(512),
            wind: [0.4, 0.0, 0.8],
            ..Default::default()
        }
        .alone();
        let mut gpu = groomer.simulate(&lods, Some(&head), settings).expect("sim");
        // The host's simulation of the GPU's rest; its binding of every
        // drawn strand, by their roots, from the finest level.
        let positions = gpu.read_rest(&groomer).expect("read");
        let source = crate::lod::StrandLevel::new(
            gpu.layout().points(), positions,
            vec![crate::lod::StrandInfo::default(); gpu.layout().strands().count()],
        ).expect("host source");
        let mut host = HairSim::from_level(Arc::new(source), settings).expect("host simulation");
        assert_eq!(host.layout().strands().count(), gpu.layout().strands().count());
        let first = &lods.levels[0];
        let finest: Vec<[f32; 4]> =
            pollster::block_on(first.positions.read(groomer.context())).expect("read");
        let source = crate::lod::StrandLevel::new(
            first.layout().points(), finest,
            vec![crate::lod::StrandInfo::default(); first.layout().strands().count()],
        ).expect("binding source");
        let bound = HairSim::from_level(Arc::new(source), settings).expect("host binding");
        let binding = bound.frame().binding();

        let context = groomer.context();
        let ids: Vec<[u32; 4]> = pollster::block_on(gpu.binding_ids().read(context)).expect("read");
        let same = ids
            .iter()
            .zip(binding.guides())
            .filter(|(a, b)| a[0] == b[0])
            .count();
        assert!(
            same * 100 >= ids.len() * 99,
            "bound to other strands: {same} of {}",
            ids.len()
        );

        let pose = |t: f32| {
            Rigid::from_angles(
                0.4 * (1.1 * t).sin(),
                0.1 * t.sin(),
                0.0,
                Vec3::new(0.0, -0.12, -0.02),
                Vec3::default(),
            )
        };
        host.reset(pose(0.0));
        gpu.reset(&groomer, pose(0.0)).expect("reset");
        let dt = 1.0 / 60.0;
        for frame in 1..=30 {
            let t = frame as f32 * dt;
            host.step(dt, pose(t), Some(&head));
            gpu.step(&groomer, dt, pose(t)).expect("step");
        }
        let moved = gpu.read_positions(&groomer).expect("read");
        let expected = host.positions();
        let off: f32 = moved
            .iter()
            .zip(&expected)
            .map(|(a, b)| (Vec3::new(a[0], a[1], a[2]) - Vec3::new(b[0], b[1], b[2])).length())
            .sum::<f32>()
            / moved.len() as f32;
        let travelled: f32 = expected
            .iter()
            .zip(&positions)
            .map(|(a, b)| (Vec3::new(a[0], a[1], a[2]) - Vec3::new(b[0], b[1], b[2])).length())
            .sum::<f32>()
            / expected.len() as f32;
        assert!(travelled > 0.005, "the strands barely moved: {travelled}");
        assert!(
            off < 0.02 * travelled,
            "{off} m off the host, of {travelled} m moved"
        );
    }

    /// Grown on the GPU, a hairdo's guides are the host's: the same roots,
    /// combed the same way, draped and wrapped the same way.
    #[test]
    fn guides_grow_on_the_gpu_as_on_the_host() {
        use crate::ops::Context;
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        for preset in [
            Preset::Long,
            Preset::Bob,
            Preset::Afro,
            Preset::Ringlets,
            Preset::Ponytail,
            Preset::Bun,
            Preset::Braid,
            Preset::Mohawk,
            Preset::Buzz,
            Preset::Bangs,
            Preset::Balding,
            Preset::SideSwept,
            Preset::Dreadlocks,
        ] {
            let mut look = Look::new(preset);
            look.strands = StrandCount::from_count(5_000);
            look.wind = AccelerationVector::from_components([1.5, 0.0, 0.5]);
            let head = look.head(&Head::mannequin());
            let scalp = head.scalp(0.006);
            let groom = look.groom(&head);
            let points = groom.points.max(2);
            // The host's guides, straight to the groom's points, and the
            // operations before interpolation run over them.
            let mut host = look.hairdo.guides(
                &head,
                &scalp,
                points,
                look.variation.ratio(),
                look.seed.word() as u64,
            );
            let at = groom
                .ops
                .iter()
                .position(|op| matches!(op, crate::ops::Op::Interpolate(_)))
                .expect("interpolates");
            let host_context = Context {
                scalp: &scalp,
                collider: &head,
            };
            for op in &groom.ops[..at] {
                op.apply(&mut host, &host_context);
            }
            let uploaded = groomer.upload_scalp(&scalp).expect("scalp");
            let grown = groomer.grow_look(&look, &head, &uploaded).expect("grown");
            let gpu = grown
                .guides
                .as_ref()
                .expect("guides")
                .read(groomer.context())
                .expect("read back");
            let name = preset.name();
            assert!(!gpu.has_non_finite(), "{name}");
            assert!(
                gpu.len().abs_diff(host.len()) <= 2,
                "{name}: {} guides against {}",
                gpu.len(),
                host.len()
            );
            // Each guide against the host's rooted at the same place.
            let mut off = Vec::new();
            for g in 0..gpu.len() {
                let root = gpu.root(g);
                let Some(h) = (0..host.len()).find(|&h| (host.root(h) - root).length() < 1e-3)
                else {
                    continue;
                };
                let mean = gpu
                    .strand(g)
                    .iter()
                    .zip(host.strand(h))
                    .map(|(a, b)| (*a - *b).length())
                    .sum::<f32>()
                    / points as f32;
                off.push(mean);
            }
            assert!(
                off.len() * 100 >= gpu.len() * 97,
                "{name}: {} of {} guides rooted as on the host",
                off.len(),
                gpu.len()
            );
            off.sort_by(f32::total_cmp);
            let median = off[off.len() / 2];
            let high = off[off.len() * 9 / 10];
            // The same numbers drawn and the same steps taken: only the
            // floating point differs. Along a metre of braid that shifts a
            // strand a millimetre or two, and now and then a step lands on
            // the other side of a threshold (lying on the head or not) and
            // the strand goes its own way from there -- a guide in a hundred.
            let worst = *off.last().expect("matched above");
            let astray = off.iter().filter(|&&o| o > 2e-3).count();
            assert!(median < 1e-4, "{name}: guides {median} m off the host's");
            assert!(
                high < 1e-3,
                "{name}: a tenth of the guides {high} m off the host's"
            );
            assert!(astray * 50 <= off.len(), "{name}: {astray} guides astray");
            assert!(worst < 5e-2, "{name}: a guide {worst} m off the host's");
        }
    }

    /// Cut on the GPU, a head's scalp is the host's: the same triangles in
    /// the same order, the same vertices, the same hairline.
    #[test]
    fn the_scalp_is_cut_on_the_gpu_as_on_the_host() {
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        for (preset, cell) in [
            (Preset::Long, 0.004),
            (Preset::Balding, 0.004),
            (Preset::Undercut, 0.006),
            (Preset::Afro, 0.003),
        ] {
            let look = Look::new(preset);
            let head = look.head(&Head::mannequin());
            let host = head.scalp(cell);
            let gpu = groomer.cut_scalp(&head, cell).expect("cut");
            let cut = gpu.read(&groomer).expect("read back");
            let name = preset.name();
            assert_eq!(
                cut.triangles.len(),
                host.triangles.len(),
                "{name}: triangles"
            );
            assert_eq!(
                cut.positions.len(),
                host.positions.len(),
                "{name}: vertices"
            );
            let same = cut
                .triangles
                .iter()
                .zip(&host.triangles)
                .filter(|(a, b)| a == b)
                .count();
            assert_eq!(
                same,
                host.triangles.len(),
                "{name}: triangles in another order"
            );
            let off = |a: &[Vec3], b: &[Vec3]| {
                a.iter()
                    .zip(b)
                    .map(|(p, q)| (*p - *q).length())
                    .fold(0.0f32, f32::max)
            };
            assert!(
                off(&cut.positions, &host.positions) < 1e-6,
                "{name}: positions"
            );
            assert!(off(&cut.normals, &host.normals) < 1e-3, "{name}: normals");
            let uv_off = cut
                .uvs
                .iter()
                .zip(&host.uvs)
                .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
                .fold(0.0f32, f32::max);
            assert!(uv_off < 1e-5, "{name}: uvs {uv_off}");
            assert!(
                (gpu.area() / host.area() - 1.0).abs() < 1e-5,
                "{name}: area"
            );
            let (edge, fine) = gpu.boundary_counts();
            assert_eq!(edge, host.boundary(0.002).len(), "{name}: hairline points");
            assert_eq!(
                fine,
                host.boundary(0.001).len(),
                "{name}: fine hairline points"
            );
            let [lo, hi] = gpu.bounds;
            let hlo = host
                .positions
                .iter()
                .fold(Vec3::splat(f32::INFINITY), |a, p| a.min(*p));
            let hhi = host
                .positions
                .iter()
                .fold(Vec3::splat(f32::NEG_INFINITY), |a, p| a.max(*p));
            assert!(
                (lo - hlo).length() < 1e-6 && (hi - hhi).length() < 1e-6,
                "{name}: box"
            );
        }
    }

    /// Strands made by hand, a simulation of them at their own points with
    /// nothing but what the test asks for, and their states.
    fn bare_sim(
        groomer: &mut GpuGroomer,
        strands: &crate::Strands,
        settings: crate::sim::SimSettings,
    ) -> GpuSim {
        let grown = groomer.upload(strands, None).expect("upload");
        let lods = groomer
            .lods(&grown, Vec3::default(), &crate::LodSettings::default())
            .expect("levels");
        groomer.simulate(&lods, None, settings).expect("sim")
    }

    fn quiet() -> crate::sim::SimSettings {
        crate::sim::SimSettings {
            clump: 1.0,
            internal_damping: 0.0,
            air_drag: 0.0,
            gravity: 0.0,
            turbulence: 0.0,
            ..Default::default()
        }
        .alone()
    }

    /// At rest, hair lying on hair stays as it lies: strands meeting each
    /// other answer motion, not where the groom put them.
    #[test]
    fn strands_meeting_leave_a_groom_at_rest() {
        use crate::sim::{Rigid, SimSettings};
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        groomer
            .set_head_mesh(Arc::new(Head::mannequin().mesh(0.008)))
            .expect("head");
        let mut look = Look::new(Preset::Long);
        look.strands = StrandCount::from_count(20_000);
        let head = look.head(&Head::mannequin());
        let scalp = groomer.cut_scalp(&head, 0.006).expect("scalp");
        let grown = groomer.grow_look(&look, &head, &scalp).expect("grown");
        let lods = groomer
            .lods(&grown, head.cranium.center, &crate::LodSettings::default())
            .expect("levels");
        let run = |groomer: &mut GpuGroomer, settings: SimSettings| {
            let mut sim = groomer.simulate(&lods, Some(&head), settings).expect("sim");
            for _ in 0..60 {
                sim.step(groomer, 1.0 / 60.0, Rigid::default())
                    .expect("step");
            }
            sim.read_positions(groomer).expect("read")
        };
        let settings = SimSettings::default();
        let meeting = run(&mut groomer, settings);
        let alone = run(&mut groomer, settings.alone());
        // How far each is from the groom: the solver alone settles a
        // little under gravity; meeting must not puff the hair out.
        let rest = groomer
            .simulate(&lods, Some(&head), settings)
            .expect("sim")
            .read_rest(&groomer)
            .expect("read");
        let from_rest = |now: &[[f32; 4]]| {
            now.iter()
                .zip(&rest)
                .map(|(a, b)| (Vec3::new(a[0], a[1], a[2]) - Vec3::new(b[0], b[1], b[2])).length())
                .sum::<f32>()
                / now.len() as f32
        };
        let (meeting, alone) = (from_rest(&meeting), from_rest(&alone));
        assert!(
            meeting < alone + 1e-3,
            "meeting moved a groom at rest {meeting} m from its groom, alone {alone} m"
        );
    }

    /// Two strands swung into each other across their paths stop where
    /// they meet, the one above still above; without the strands'
    /// collisions they pass through.
    #[test]
    fn a_strand_does_not_pass_through_another() {
        use crate::strands::{Attachment, Strands};
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        let points = 16;
        let mut strands = Strands::new(points);
        let line = |from: Vec3, to: Vec3| -> Vec<Vec3> {
            (0..points)
                .map(|j| from.lerp(to, j as f32 / (points - 1) as f32))
                .collect()
        };
        // One along x, swinging down; one along z beneath it, swinging up:
        // their bending brings them back, after passing through if nothing
        // stops them.
        strands.push(
            &line(Vec3::new(-0.1, 0.0, 0.0), Vec3::new(0.16, 0.0, 0.0)),
            Attachment {
                id: StrandId::from_word(1),
                ..Default::default()
            },
        );
        strands.push(
            &line(Vec3::new(0.05, -0.03, -0.1), Vec3::new(0.05, -0.03, 0.16)),
            Attachment {
                id: StrandId::from_word(2),
                ..Default::default()
            },
        );
        let mut crossing = |collide: bool| {
            // Hair's own bending and damping, nothing pulling it back.
            let settings = crate::sim::SimSettings {
                guides: StrandCount::from_count(2),
                points: PointsPerStrand::from_count(points),
                strand_collision: collide,
                collision_radius: 0.006,
                clump: crate::sim::SimSettings::default().clump,
                internal_damping: crate::sim::SimSettings::default().internal_damping,
                ..quiet()
            };
            let mut sim = bare_sim(&mut groomer, &strands, settings);
            let rest = sim.read_rest(&groomer).expect("read");
            // Which simulated strand is the upper: rooted at -x.
            let upper = if rest[0][0] < -0.05 { 0 } else { 1 };
            // Each moving towards the other, faster towards its tip.
            let previous: Vec<[f32; 4]> = rest
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let along = (i % points) as f32 / (points - 1) as f32;
                    let towards = if i / points == upper { 1.0 } else { -1.0 };
                    [p[0], p[1] + 0.004 * along * towards, p[2], 1.0]
                })
                .collect();
            sim.set_state(&groomer, &rest, &previous).expect("state");
            // Wherever the two cross, seen from above, every frame: how far
            // the upper is above the lower there.
            let mut heights = Vec::new();
            for _ in 0..20 {
                sim.step(&groomer, 1.0 / 60.0, crate::sim::Rigid::default())
                    .expect("step");
                let now = sim.read_positions(&groomer).expect("read");
                let strand = |g: usize| &now[g * points..(g + 1) * points];
                let (top, bottom) = (strand(upper), strand(1 - upper));
                for a in top.windows(2) {
                    for b in bottom.windows(2) {
                        let (p, r) = ([a[0][0], a[0][2]], [a[1][0] - a[0][0], a[1][2] - a[0][2]]);
                        let (q, t) = ([b[0][0], b[0][2]], [b[1][0] - b[0][0], b[1][2] - b[0][2]]);
                        let cross = r[0] * t[1] - r[1] * t[0];
                        if cross.abs() < 1e-12 {
                            continue;
                        }
                        let d = [q[0] - p[0], q[1] - p[1]];
                        let u = (d[0] * t[1] - d[1] * t[0]) / cross;
                        let v = (d[0] * r[1] - d[1] * r[0]) / cross;
                        if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
                            let ya = a[0][1] + (a[1][1] - a[0][1]) * u;
                            let yb = b[0][1] + (b[1][1] - b[0][1]) * v;
                            heights.push(ya - yb);
                        }
                    }
                }
            }
            heights
        };
        let with = crossing(true);
        let without = crossing(false);
        assert!(
            without.iter().any(|&h| h < -0.01),
            "without collisions they should pass: {without:?}"
        );
        // Through by no more than a sliver of the radius.
        assert!(
            with.iter().all(|&h| h > -0.002),
            "they passed through: {with:?} above"
        );
    }

    /// A bundle pressed together spreads again: the volume pushes hair
    /// denser than at rest apart.
    #[test]
    fn hair_pressed_together_keeps_its_volume() {
        use crate::strands::{Attachment, Strands};
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        let points = 12;
        let mut strands = Strands::new(points);
        let mut id = 0;
        for a in 0..8 {
            for b in 0..8 {
                let root = Vec3::new((a as f32 - 3.5) * 0.01, 0.0, (b as f32 - 3.5) * 0.01);
                let strand: Vec<Vec3> = (0..points)
                    .map(|j| root + Vec3::new(0.0, -0.15 * j as f32 / (points - 1) as f32, 0.0))
                    .collect();
                id += 1;
                strands.push(
                    &strand,
                    Attachment {
                        id: StrandId::from_word(id),
                        ..Default::default()
                    },
                );
            }
        }
        let mut spread = |volume: f32| {
            let settings = crate::sim::SimSettings {
                guides: StrandCount::from_count(64),
                points: PointsPerStrand::from_count(points),
                volume,
                grid_cell: 0.008,
                ..quiet()
            };
            let mut sim = bare_sim(&mut groomer, &strands, settings);
            // Every point below the root pulled onto the bundle's axis.
            let rest = sim.read_rest(&groomer).expect("read");
            let pressed: Vec<[f32; 4]> = rest
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if i % points == 0 {
                        *p
                    } else {
                        [p[0] * 0.1, p[1], p[2] * 0.1, 1.0]
                    }
                })
                .collect();
            sim.set_state(&groomer, &pressed, &pressed).expect("state");
            for _ in 0..30 {
                sim.step(&groomer, 1.0 / 60.0, crate::sim::Rigid::default())
                    .expect("step");
            }
            let now = sim.read_positions(&groomer).expect("read");
            let middle: Vec<f32> = now
                .chunks(points)
                .map(|s| (s[points / 2][0].powi(2) + s[points / 2][2].powi(2)).sqrt())
                .collect();
            middle.iter().sum::<f32>() / middle.len() as f32
        };
        let kept = spread(0.5);
        let pressed = spread(0.0);
        assert!(
            kept > pressed * 1.5,
            "the bundle stayed pressed: {kept} m out against {pressed} m"
        );
    }

    /// Every hairstyle, curls and all, rests as groomed under gravity with
    /// its strands meeting each other, and swings with a shaking head
    /// without blowing up.
    #[test]
    fn every_hairstyle_rests_and_swings() {
        use crate::sim::{Rigid, SimLevel, SimSettings};
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        groomer
            .set_head_mesh(Arc::new(Head::mannequin().mesh(0.008)))
            .expect("head");
        let shake = |t: f32| {
            Rigid::from_angles(
                0.6 * (7.0 * t).sin(),
                0.2 * (5.0 * t).sin(),
                0.0,
                Vec3::new(0.0, -0.12, -0.02),
                Vec3::default(),
            )
        };
        for preset in Preset::ALL {
            let mut look = Look::new(preset);
            look.strands = StrandCount::from_count(20_000);
            let head = look.head(&Head::mannequin());
            let scalp = groomer.cut_scalp(&head, 0.006).expect("scalp");
            let grown = groomer.grow_look(&look, &head, &scalp).expect("grown");
            let lods = groomer
                .lods(&grown, head.cranium.center, &crate::LodSettings::default())
                .expect("levels");
            let mut sim = groomer
                .simulate(
                    &lods,
                    Some(&head),
                    SimSettings::default().at(SimLevel::MEDIUM),
                )
                .expect("sim");
            for _ in 0..30 {
                sim.step(&groomer, 1.0 / 60.0, Rigid::default())
                    .expect("step");
            }
            let rest = sim.read_rest(&groomer).expect("read");
            let still = sim.read_positions(&groomer).expect("read");
            let drift = still
                .iter()
                .zip(&rest)
                .map(|(a, b)| (Vec3::new(a[0], a[1], a[2]) - Vec3::new(b[0], b[1], b[2])).length())
                .fold(0.0f32, f32::max);
            for frame in 1..=90 {
                sim.step(&groomer, 1.0 / 60.0, shake(frame as f32 / 60.0))
                    .expect("step");
            }
            let shaken = sim.read_positions(&groomer).expect("read");
            let centre = head.cranium.center;
            let farthest = shaken
                .iter()
                .map(|p| (Vec3::new(p[0], p[1], p[2]) - centre).length())
                .fold(
                    0.0f32,
                    |a, b| if b.is_nan() { f32::INFINITY } else { a.max(b) },
                );
            eprintln!("{preset:?}: drifts {drift} m at rest, reaches {farthest} m shaken");
            // Dense curls settle a few millimetres as their strands find
            // each other; nothing else moves.
            assert!(drift < 0.01, "{preset:?} drifts {drift} m at rest");
            assert!(
                farthest < 0.8,
                "{preset:?} reaches {farthest} m from the head"
            );
        }
    }

    /// A simulation moved to a finer level carries on as it was: the
    /// strands both simulate are where they were.
    #[test]
    fn a_simulation_changes_level_without_a_jump() {
        use crate::sim::{Rigid, SimLevel, SimSettings};
        let Ok(context) = headless_context() else {
            eprintln!("no GPU: skipped");
            return;
        };
        let mut groomer = GpuGroomer::new(context).expect("kernels");
        groomer
            .set_head_mesh(Arc::new(Head::mannequin().mesh(0.008)))
            .expect("head");
        let mut look = Look::new(Preset::Long);
        look.strands = StrandCount::from_count(20_000);
        let head = look.head(&Head::mannequin());
        let scalp = groomer.cut_scalp(&head, 0.006).expect("scalp");
        let grown = groomer.grow_look(&look, &head, &scalp).expect("grown");
        let lods = groomer
            .lods(&grown, head.cranium.center, &crate::LodSettings::default())
            .expect("levels");
        let settings = SimSettings::default();
        let mut low = groomer
            .simulate(&lods, Some(&head), settings.at(SimLevel::LOW))
            .expect("sim");
        let sway = Rigid::from_angles(0.4, 0.1, 0.0, Vec3::new(0.0, -0.12, -0.02), Vec3::default());
        for _ in 0..20 {
            low.step(&groomer, 1.0 / 60.0, sway).expect("step");
        }
        let high = groomer
            .resimulate(&low, &lods, Some(&head), settings.at(SimLevel::HIGH))
            .expect("resimulated");
        assert_eq!(high.level(), SimLevel::HIGH);
        let before = low.read_positions(&groomer).expect("read");
        let after = high.read_positions(&groomer).expect("read");
        let (lp, hp) = (low.layout().points().count(), high.layout().points().count());
        // The low level's strands, at their tips, in both.
        for g in 0..low.layout().strands().count() {
            let a = before[g * lp + lp - 1];
            let b = after[g * hp + hp - 1];
            let off = (Vec3::new(a[0], a[1], a[2]) - Vec3::new(b[0], b[1], b[2])).length();
            assert!(off < 1e-4, "strand {g} jumped {off} m");
        }
        // And none of the new strands left at rest while the head is turned.
        let rest = high.read_rest(&groomer).expect("read");
        let moved = (low.layout().strands().count()..high.layout().strands().count())
            .filter(|g| {
                let i = g * hp + hp - 1;
                let p = Vec3::new(after[i][0], after[i][1], after[i][2]);
                let r = Vec3::new(rest[i][0], rest[i][1], rest[i][2]);
                (p - r).length() > 1e-3
            })
            .count();
        assert!(
            moved * 10 > (high.layout().strands().count() - low.layout().strands().count()) * 9,
            "{moved} new strands moved"
        );
    }
}

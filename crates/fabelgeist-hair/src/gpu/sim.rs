//! Real-time physics on the GPU: [`crate::sim::HairSim`]'s strands, stepped
//! where the groom already is and read where it is drawn, and meeting each
//! other -- volume, friction and collisions (see [`crate::sim`]).
//!
//! A groom made on the GPU never comes back to the host, and neither does
//! its simulation: the simulated strands are the first of the groom's
//! shuffled order (an even sample of the head, as on the host), resampled
//! to the simulation's points; each drawn strand is bound to the four
//! nearest its root by a kernel, and a substep is a dispatch with an
//! invocation a simulated strand, which takes its rod through the
//! substep's implicit steps ([`crate::rod`]) before the strands meet. The renderer deforms the levels it draws
//! straight from the positions buffer
//! ([`crate::render::HairRenderer::set_gpu_dynamics`]).
//!
//! How many strands, of how many points, is the simulation's level of
//! detail ([`crate::sim::SimLevel`]); [`GpuGroomer::resimulate`] moves a running
//! simulation to another, carrying its motion.

use crate::gpu::error::Result;
use crate::gpu::{GpuLevel, StrandLayout};
use crate::parameters::{PointsPerStrand, StrandCount};
use std::sync::Arc;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_math::Vec3;

use super::grow::params;
use super::{GpuGroomer, GpuLods, head_shape, storage, upload};
use crate::head::Head;
use crate::rod::Rod;
use crate::sim::{Rigid, SimSettings};

use binding::{BindingBuffers, BindingRequest};
use collision::CollisionBuffers;
use rods::RodBuffers;

mod binding;
mod collision;
mod collision_progress;
mod density;
mod execution;
mod inspect;
mod rods;

/// The density grid, in the head's frame.
struct SimGrid {
    lo: Vec3,
    cell: f32,
    dims: [u32; 3],
    /// Density and momentum, four words a cell, cleared every substep.
    words: Buffer,
    /// The groom's density at rest.
    rest_density: Buffer,
    /// The rest density's sum and how many cells hold any.
    stats: Buffer,
}

/// Simulated strands on the GPU, and each drawn strand's binding to them.
pub struct GpuSim {
    pub settings: SimSettings,
    layout: StrandLayout,
    source: Arc<GpuLevel>,
    /// Groomed points, in the head's frame, `w` one.
    rest: Buffer,
    /// Each strand's rod's rest ([`crate::rod::rest_shape`]), two words a
    /// point, and the material and choice it was solved for.
    shape: Buffer,
    shaped: Option<(Rod, bool)>,
    /// The implicit steps' factors, fourteen words a point.
    factors: Buffer,
    /// World space, `w` one, as the renderer reads them; `previous` is a
    /// substep's velocity behind, the substep `h_last` seconds. Each
    /// segment's frame and angular velocity, two words a point.
    position: Buffer,
    previous: Buffer,
    frames: Buffer,
    h_last: f32,
    /// Per drawn strand, in the groom's shuffled order: four simulated
    /// strands and their weights.
    binding: BindingBuffers,
    head_shape: Buffer,
    body: u32,
    head: Rigid,
    time: f32,
    grid: SimGrid,
    /// Samples, grid and native addresses for the colliding prefix.
    collisions: CollisionBuffers,
}

impl SimGrid {
    fn cells(&self) -> u32 {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    fn insert(&self, p: &mut PassParameters) {
        p.insert("lo_x", self.lo.x);
        p.insert("lo_y", self.lo.y);
        p.insert("lo_z", self.lo.z);
        p.insert("cell", self.cell);
        p.insert("nx", self.dims[0]);
        p.insert("ny", self.dims[1]);
        p.insert("nz", self.dims[2]);
    }
}

impl GpuGroomer {
    /// Simulate `lods`' groom at `settings`' level (its strands and
    /// points), keeping it out of `head` when there is one: the strands
    /// bound and at rest, the head where [`Rigid::default`] has it until
    /// [`GpuSim::reset`].
    pub fn simulate(
        &mut self,
        lods: &GpuLods,
        head: Option<&Head>,
        settings: SimSettings,
    ) -> Result<GpuSim> {
        let mut sim = self.make_sim(lods, head, settings)?;
        sim.reset(self, Rigid::default())?;
        Ok(sim)
    }

    /// `old`'s simulation moved to `settings`' level: the strands both
    /// simulate carry on as they were, resampled, and the others start as
    /// the old strands around them have moved.
    pub fn resimulate(
        &mut self,
        old: &GpuSim,
        lods: &GpuLods,
        head: Option<&Head>,
        settings: SimSettings,
    ) -> Result<GpuSim> {
        old.source.ensure_device(&self.context)?;
        let mut sim = self.make_sim(lods, head, settings)?;
        let context = self.context.clone();
        let mut batch = KernelBatch::labelled(&context, "hair sim transfer");
        let binding = BindingBuffers::record(
            &context,
            &mut batch,
            &self.kernels.sim_bind,
            BindingRequest {
                drawn: sim.source(),
                guides: old.source(),
                drawn_count: sim.layout.strands(),
                guide_count: old.layout.strands(),
            },
        )?;
        let mut p = params! {
            "rest" => sim.rest.clone(),
            "position" => sim.position.clone(),
            "previous" => sim.previous.clone(),
            "old_rest" => old.rest.clone(),
            "old_position" => old.position.clone(),
            "old_previous" => old.previous.clone(),
            "ids" => binding.ids,
            "weights" => binding.weights,
            "count" => sim.layout.native_strands(),
            "points" => sim.layout.native_points(),
            "old_points" => old.layout.native_points(),
            "pad0" => 0u32,
        };
        insert_rigid(&mut p, &old.head);
        batch.dispatch_folded(&self.kernels.sim_transfer, &p, sim.layout.sample_invocations())?;
        let rod = sim.record_shape(self, &mut batch)?;
        sim.record_orient(self, &mut batch, &old.head)?;
        batch.submit()?;
        sim.shaped = Some((rod, sim.settings.sag_free));
        sim.head = old.head;
        sim.time = old.time;
        sim.h_last = old.h_last;
        Ok(sim)
    }

    fn make_sim(
        &mut self,
        lods: &GpuLods,
        head: Option<&Head>,
        settings: SimSettings,
    ) -> Result<GpuSim> {
        let context = self.context.clone();
        let first = &lods.levels[0];
        first.ensure_device(&context)?;
        let source_layout = first.layout();
        let available = source_layout.strands().count();
        let count = settings.guides.min(available).max(1.min(available));
        let layout = StrandLayout::new(
            StrandCount::from_count(count),
            PointsPerStrand::from_count(settings.points.max(2)),
        )?;
        let RodBuffers {
            rest,
            shape,
            frames,
            factors,
            position,
            previous,
        } = RodBuffers::new(&context, layout)?;
        let count = layout.native_strands();
        let points = layout.native_points();
        let body = upload(
            &context,
            &head.map(head_shape).unwrap_or_else(|| vec![[0.0; 4]; 17]),
            "hair sim head".into(),
        )?;
        let longest = storage(&context, 16_u64.into(), "hair sim longest segment".into())?;
        let mut batch = KernelBatch::labelled(&context, "hair sim");
        batch.clear_buffer(&longest);
        batch.dispatch_folded(
            &self.kernels.sim_rest,
            &params! {
                "level" => first.positions.clone(),
                "rest" => rest.clone(),
                "longest" => longest.clone(),
                "count" => count,
                "points" => points,
                "level_points" => first.layout.native_points(),
                "pad0" => 0u32,
            },
            layout.strand_invocations(),
        )?;
        let binding = BindingBuffers::record(
            &context,
            &mut batch,
            &self.kernels.sim_bind,
            BindingRequest {
                drawn: first,
                guides: first,
                drawn_count: source_layout.strands(),
                guide_count: layout.strands(),
            },
        )?;

        // The density grid, round the groom with room for it to swing, in
        // the head's frame (the groom's).
        let [lo, hi] = lods.bounds;
        let pad = Vec3::new(0.08, 0.15, 0.08);
        let (lo, hi) = (lo - pad, hi + pad);
        let grid = self.record_rest_density(
            &mut batch,
            &rest,
            layout,
            lo,
            hi,
            settings.grid_cell,
        )?;

        let longest = f32::from_bits(self.read_words(batch, &[(longest, 0)])?[0]);
        let collisions = CollisionBuffers::new(&context, layout, settings, lo, hi, longest)?;
        Ok(GpuSim {
            settings,
            layout,
            source: first.clone(),
            rest,
            shape,
            shaped: None,
            factors,
            position,
            previous,
            frames,
            h_last: 1.0 / 240.0,
            binding,
            head_shape: body,
            body: u32::from(head.is_some()),
            head: Rigid::default(),
            time: 0.0,
            grid,
            collisions,
        })
    }
}

fn insert_rigid(p: &mut PassParameters, head: &Rigid) {
    let [x, y, z] = head.axes;
    let t = head.translation;
    for (name, value) in [
        ("ax_x", x.x),
        ("ax_y", x.y),
        ("ax_z", x.z),
        ("ay_x", y.x),
        ("ay_y", y.y),
        ("ay_z", y.z),
        ("az_x", z.x),
        ("az_y", z.y),
        ("az_z", z.z),
        ("tx", t.x),
        ("ty", t.y),
        ("tz", t.z),
    ] {
        p.insert(name, value);
    }
}

/// The pose a substep starts from, as [`crate::gpu::wgsl::sim`]'s
/// `FROM_PARAMS` names it.
fn insert_from(p: &mut PassParameters, head: &Rigid) {
    let [x, y, z] = head.axes;
    let t = head.translation;
    for (name, value) in [
        ("from_xx", x.x),
        ("from_xy", x.y),
        ("from_xz", x.z),
        ("from_yx", y.x),
        ("from_yy", y.y),
        ("from_yz", y.z),
        ("from_zx", z.x),
        ("from_zy", z.y),
        ("from_zz", z.z),
        ("from_tx", t.x),
        ("from_ty", t.y),
        ("from_tz", t.z),
    ] {
        p.insert(name, value);
    }
}

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
//! detail ([`SimLevel`]); [`GpuGroomer::resimulate`] moves a running
//! simulation to another, carrying its motion.

use crate::gpu::error::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::PrefixSumScratch;
use fabelgeist_compute::ScanElementCount;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_math::Vec3;

use super::grow::params;
use super::{GpuGroomer, GpuLods, head_shape, storage, upload};
use crate::head::Head;
use crate::rod::Rod;
use crate::sim::{Rigid, SimSettings};

use collision_progress::CollisionProgress;

mod collision_progress;
mod density;
mod execution;

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

/// The grid the strands' samples are sorted into to find their neighbours:
/// cells as wide as a sample's reach, round the groom in the head's frame
/// (samples outside are kept in the nearest cell), and how many samples
/// each holds, scanned into where each cell's begin.
struct CollisionCells {
    lo: Vec3,
    cell: f32,
    dims: [u32; 3],
    counts: Buffer,
    scan: PrefixSumScratch,
}

impl CollisionCells {
    fn cells(&self) -> u32 {
        self.dims[0] * self.dims[1] * self.dims[2]
    }
}

/// Simulated strands on the GPU, and each drawn strand's binding to them.
pub struct GpuSim {
    pub settings: SimSettings,
    count: u32,
    points: u32,
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
    ids: Buffer,
    weights: Buffer,
    head_shape: Buffer,
    body: u32,
    head: Rigid,
    time: f32,
    grid: SimGrid,
    /// The strands' collisions: how many strands collide, samples along
    /// their segments -- this pass's and the last's, in turn -- with their
    /// segments and velocities, sorted into a grid of cells in the head's
    /// frame.
    colliders: u32,
    cells: CollisionCells,
    samples: [Buffer; 2],
    collision_progress: CollisionProgress,
    /// Each sample's segment, now and at rest, and its velocity.
    sample_data: Buffer,
    per_segment: u32,
    radius: f32,
    /// Each sample's cell and place in it.
    sample_slots: Buffer,
    /// The samples in the cells' order, eight words a sample, and which
    /// sample each is.
    sorted: Buffer,
    sorted_index: Buffer,
    /// What each sample asks, two words a sample.
    asks: Buffer,
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
        let mut sim = self.make_sim(lods, head, settings)?;
        let context = self.context.clone();
        let first = &lods.levels[0];
        // Each new strand's nearest among the old ones.
        let ids = storage(
            &context,
            (sim.count as u64 * 16).into(),
            "hair sim transfer ids".into(),
        )?;
        let weights = storage(
            &context,
            (sim.count as u64 * 16).into(),
            "hair sim transfer weights".into(),
        )?;
        let mut batch = KernelBatch::labelled(&context, "hair sim transfer");
        batch.dispatch_folded(
            &self.kernels.sim_bind,
            &params! {
                "level" => first.positions.clone(),
                "ids" => ids.clone(),
                "weights" => weights.clone(),
                "strands" => sim.count,
                "points" => first.points,
                "count" => old.count,
                "pad0" => 0u32,
            },
            sim.count,
        )?;
        let mut p = params! {
            "rest" => sim.rest.clone(),
            "position" => sim.position.clone(),
            "previous" => sim.previous.clone(),
            "old_rest" => old.rest.clone(),
            "old_position" => old.position.clone(),
            "old_previous" => old.previous.clone(),
            "ids" => ids,
            "weights" => weights,
            "count" => sim.count,
            "points" => sim.points,
            "old_points" => old.points,
            "pad0" => 0u32,
        };
        insert_rigid(&mut p, &old.head);
        batch.dispatch_folded(&self.kernels.sim_transfer, &p, sim.count * sim.points)?;
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
        let strands = first.count;
        let count = (settings.guides as u32).min(strands).max(1.min(strands));
        let points = (settings.points as u32).max(2);
        let words = (count * points) as u64 * 16;
        let rest = storage(&context, (words).into(), "hair sim rest".into())?;
        let shape = storage(&context, (words * 2).into(), "hair sim rod rest".into())?;
        let frames = storage(&context, (words * 2).into(), "hair sim frames".into())?;
        let factors = storage(&context, (words * 14).into(), "hair sim factors".into())?;
        let position = storage(&context, (words).into(), "hair sim positions".into())?;
        let previous = storage(&context, (words).into(), "hair sim previous".into())?;
        let ids = storage(
            &context,
            (strands as u64 * 16).into(),
            "hair sim bind ids".into(),
        )?;
        let weights = storage(
            &context,
            (strands as u64 * 16).into(),
            "hair sim bind weights".into(),
        )?;
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
                "level_points" => first.points,
                "pad0" => 0u32,
            },
            count,
        )?;
        batch.dispatch_folded(
            &self.kernels.sim_bind,
            &params! {
                "level" => first.positions.clone(),
                "ids" => ids.clone(),
                "weights" => weights.clone(),
                "strands" => strands,
                "points" => first.points,
                "count" => count,
                "pad0" => 0u32,
            },
            strands,
        )?;

        // The density grid, round the groom with room for it to swing, in
        // the head's frame (the groom's).
        let [lo, hi] = lods.bounds;
        let pad = Vec3::new(0.08, 0.15, 0.08);
        let (lo, hi) = (lo - pad, hi + pad);
        let grid = self.record_rest_density(
            &mut batch,
            &rest,
            count * points,
            lo,
            hi,
            settings.grid_cell,
        )?;

        // The colliding strands are the first: a wisp each, as wide as
        // their spacing over a scalp of about six hundred square
        // centimetres, with samples along a segment no further apart.
        let colliders = (settings.collision_strands as u32).clamp(1, count);
        let radius = if settings.collision_radius > 0.0 {
            settings.collision_radius
        } else {
            (0.6 * (0.06 / colliders as f32).sqrt()).clamp(0.002, 0.008)
        };
        let longest = f32::from_bits(self.read_words(batch, &[(longest, 0)])?[0]);
        // Samples two radii apart along their segments lie no more than
        // this apart when the segments are a radius apart: a radius each way
        // along them, and the radius between.
        let reach = radius * 1.75;
        let mut cell = reach;
        let cell_dims = |cell: f32| {
            let e = hi - lo;
            [e.x, e.y, e.z].map(|e| ((e / cell).ceil() as u32).max(1))
        };
        while cell_dims(cell).iter().map(|&d| d as u64).product::<u64>() > 1 << 23 {
            cell *= 1.25;
        }
        let dims = cell_dims(cell);
        let cell_count = dims.iter().product::<u32>();
        let cells = CollisionCells {
            lo,
            cell,
            dims,
            counts: storage(
                &context,
                ((cell_count as u64 + 1) * 4).into(),
                "hair sim collision cells".into(),
            )?,
            scan: PrefixSumScratch::new(&context, ScanElementCount::from_elements(cell_count + 1))?,
        };
        // Samples two radii apart: the narrow phase takes the segments as
        // they are, and samples need only find which are near.
        let per_segment = ((longest / (2.0 * radius)).ceil() as u32).clamp(1, 16);
        let sample_count = (colliders * (points - 1) * per_segment).max(1);
        let sample_buffer =
            |label: &str| storage(&context, (sample_count as u64 * 16).into(), (label).into());
        Ok(GpuSim {
            settings,
            count,
            points,
            rest,
            shape,
            shaped: None,
            factors,
            position,
            previous,
            frames,
            h_last: 1.0 / 240.0,
            ids,
            weights,
            head_shape: body,
            body: u32::from(head.is_some()),
            head: Rigid::default(),
            time: 0.0,
            grid,
            colliders,
            cells,
            samples: [
                sample_buffer("hair sim samples")?,
                sample_buffer("hair sim samples before")?,
            ],
            collision_progress: CollisionProgress::default(),
            sample_data: storage(
                &context,
                (sample_count as u64 * 80).into(),
                "hair sim sample segments".into(),
            )?,
            per_segment,
            radius,
            sample_slots: storage(
                &context,
                (sample_count as u64 * 8).into(),
                "hair sim sample cells".into(),
            )?,
            sorted: storage(
                &context,
                (sample_count as u64 * 128).into(),
                "hair sim sorted samples".into(),
            )?,
            sorted_index: storage(
                &context,
                (sample_count as u64 * 4).into(),
                "hair sim sorted samples' order".into(),
            )?,
            asks: storage(
                &context,
                (sample_count as u64 * 32).into(),
                "hair sim collision asks".into(),
            )?,
        })
    }
}


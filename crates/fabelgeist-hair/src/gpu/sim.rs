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

use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::PrefixSumScratch;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_math::Vec3;

use super::grow::params;
use super::{GpuGroomer, GpuLods, head_shape, storage, upload};
use crate::head::Head;
use crate::rod::Rod;
use crate::sim::{Rigid, SimLevel, SimSettings, blend};

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
    current: usize,
    have_before: bool,
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
        let ids = storage(&context, sim.count as u64 * 16, "hair sim transfer ids")?;
        let weights = storage(&context, sim.count as u64 * 16, "hair sim transfer weights")?;
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
        sim.record_shape(self, &mut batch)?;
        sim.record_orient(self, &mut batch, &old.head)?;
        batch.submit();
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
        let rest = storage(&context, words, "hair sim rest")?;
        let shape = storage(&context, words * 2, "hair sim rod rest")?;
        let frames = storage(&context, words * 2, "hair sim frames")?;
        let factors = storage(&context, words * 14, "hair sim factors")?;
        let position = storage(&context, words, "hair sim positions")?;
        let previous = storage(&context, words, "hair sim previous")?;
        let ids = storage(&context, strands as u64 * 16, "hair sim bind ids")?;
        let weights = storage(&context, strands as u64 * 16, "hair sim bind weights")?;
        let body = upload(
            &context,
            &head.map(head_shape).unwrap_or_else(|| vec![[0.0; 4]; 17]),
            "hair sim head",
        )?;
        let longest = storage(&context, 16, "hair sim longest segment")?;
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
        let mut cell = settings.grid_cell.max(0.002);
        let dims = |cell: f32| {
            let e = hi - lo;
            [e.x, e.y, e.z].map(|e| ((e / cell).ceil() as u32).max(1))
        };
        while dims(cell).iter().map(|&d| d as u64).product::<u64>() > 1 << 21 {
            cell *= 1.25;
        }
        let grid = {
            let dims = dims(cell);
            let cells = dims.iter().product::<u32>() as u64;
            SimGrid {
                lo,
                cell,
                dims,
                words: storage(&context, cells * 16, "hair sim grid")?,
                rest_density: storage(&context, cells * 4, "hair sim rest density")?,
                stats: storage(&context, 16, "hair sim rest density stats")?,
            }
        };
        batch.clear_buffer(&grid.rest_density);
        batch.clear_buffer(&grid.stats);
        let mut p = params! {
            "rest" => rest.clone(),
            "rest_density" => grid.rest_density.clone(),
            "count" => count * points,
        };
        grid.insert(&mut p);
        batch.dispatch_folded(&self.kernels.sim_rest_density, &p, count * points)?;
        batch.dispatch_folded(
            &self.kernels.sim_rest_stats,
            &params! {
                "rest_density" => grid.rest_density.clone(),
                "stats" => grid.stats.clone(),
                "count" => grid.cells(),
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            grid.cells(),
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
                (cell_count as u64 + 1) * 4,
                "hair sim collision cells",
            )?,
            scan: PrefixSumScratch::new(&context, cell_count + 1)?,
        };
        // Samples two radii apart: the narrow phase takes the segments as
        // they are, and samples need only find which are near.
        let per_segment = ((longest / (2.0 * radius)).ceil() as u32).clamp(1, 16);
        let sample_count = (colliders * (points - 1) * per_segment).max(1);
        let sample_buffer = |label: &str| storage(&context, sample_count as u64 * 16, label);
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
            current: 0,
            have_before: false,
            sample_data: storage(
                &context,
                sample_count as u64 * 80,
                "hair sim sample segments",
            )?,
            per_segment,
            radius,
            sample_slots: storage(&context, sample_count as u64 * 8, "hair sim sample cells")?,
            sorted: storage(
                &context,
                sample_count as u64 * 128,
                "hair sim sorted samples",
            )?,
            sorted_index: storage(
                &context,
                sample_count as u64 * 4,
                "hair sim sorted samples' order",
            )?,
            asks: storage(
                &context,
                sample_count as u64 * 32,
                "hair sim collision asks",
            )?,
        })
    }
}

impl GpuSim {
    /// Strands simulated.
    pub fn strands(&self) -> usize {
        self.count as usize
    }

    /// Points per simulated strand.
    pub fn points(&self) -> usize {
        self.points as usize
    }

    /// The level simulated.
    pub fn level(&self) -> SimLevel {
        SimLevel {
            strands: self.count as usize,
            points: self.points as usize,
        }
    }

    /// How far apart strands keep, metres.
    pub fn collision_radius(&self) -> f32 {
        self.radius
    }

    /// How many of the strands collide.
    pub fn colliding(&self) -> usize {
        self.colliders as usize
    }

    pub fn head(&self) -> Rigid {
        self.head
    }

    /// The simulated strands' groomed points, in the head's frame.
    pub fn rest(&self) -> &Buffer {
        &self.rest
    }

    /// The simulated strands' points now, world space.
    pub fn positions(&self) -> &Buffer {
        &self.position
    }

    /// The body the strands are kept off ([`super::head_shape`]), if any.
    pub fn body(&self) -> Option<&Buffer> {
        (self.body != 0).then_some(&self.head_shape)
    }

    /// Each drawn strand's four simulated strands, and their weights.
    pub fn binding(&self) -> (&Buffer, &Buffer) {
        (&self.ids, &self.weights)
    }

    /// Back to the groom, the head where it is.
    pub fn reset(&mut self, groomer: &GpuGroomer, head: Rigid) -> Result<()> {
        self.head = head;
        let context = groomer.context();
        let count = self.count * self.points;
        let mut p = params! {
            "rest" => self.rest.clone(),
            "position" => self.position.clone(),
            "previous" => self.previous.clone(),
            "count" => count,
            "pad0" => 0u32,
            "pad1" => 0u32,
            "pad2" => 0u32,
        };
        insert_rigid(&mut p, &head);
        let mut batch = KernelBatch::labelled(context, "hair sim reset");
        batch.dispatch_folded(&groomer.kernels.sim_reset, &p, count)?;
        self.record_shape(groomer, &mut batch)?;
        self.record_orient(groomer, &mut batch, &head)?;
        batch.submit();
        Ok(())
    }

    /// The rods' rest solved for the settings, if they changed.
    fn record_shape(&mut self, groomer: &GpuGroomer, batch: &mut KernelBatch) -> Result<()> {
        let s = self.settings;
        let rod = Rod::new(&s);
        if self.shaped == Some((rod, s.sag_free)) {
            return Ok(());
        }
        batch.dispatch_folded(
            &groomer.kernels.sim_shape,
            &params! {
                "rest" => self.rest.clone(),
                "shape" => self.shape.clone(),
                "count" => self.count,
                "points" => self.points,
                "sag_free" => u32::from(s.sag_free),
                "pad0" => 0u32,
                "stretch" => rod.stretch,
                "bend" => rod.bend,
                "twist" => rod.twist,
                "gravity" => s.gravity,
            },
            self.count,
        )?;
        self.shaped = Some((rod, s.sag_free));
        Ok(())
    }

    /// Each segment's frame onto where its strand now is, the head at
    /// `head`, not spinning.
    fn record_orient(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        head: &Rigid,
    ) -> Result<()> {
        let count = self.count * self.points;
        let mut p = params! {
            "rest" => self.rest.clone(),
            "shape" => self.shape.clone(),
            "position" => self.position.clone(),
            "frames" => self.frames.clone(),
            "count" => count,
            "points" => self.points,
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        insert_rigid(&mut p, head);
        batch.dispatch_folded(&groomer.kernels.sim_orient, &p, count)?;
        Ok(())
    }

    /// Step `dt` seconds with the head moved to `head`: the substeps are
    /// recorded, not waited for -- the frame drawn next reads them. Each
    /// substep takes the rods through their implicit steps, then keeps the
    /// strands' volume and stops them closing through each other.
    pub fn step(&mut self, groomer: &GpuGroomer, dt: f32, head: Rigid) -> Result<()> {
        let dt = dt.min(1.0 / 20.0);
        if dt.is_nan() || dt <= 0.0 {
            return Ok(());
        }
        let s = self.settings;
        let rod = Rod::new(&s);
        let mut batch = KernelBatch::labelled(groomer.context(), "hair sim step");
        self.record_shape(groomer, &mut batch)?;
        let substeps = s.substeps.max(1);
        let inner = s.solver_substeps.max(1) as u32;
        let h = dt / (substeps as u32 * inner) as f32;
        let from = self.head;
        let particles = self.count * self.points;
        let interacting = s.volume > 0.0 || s.friction > 0.0;
        for k in 0..substeps {
            // The head moves through the frame's substeps.
            let start = blend(&from, &head, k as f32 / substeps as f32);
            let now = blend(&from, &head, (k + 1) as f32 / substeps as f32);
            let wind = Vec3::from_array(s.wind);
            let mut p = params! {
                "rest" => self.rest.clone(),
                "shape" => self.shape.clone(),
                "position" => self.position.clone(),
                "previous" => self.previous.clone(),
                "frames" => self.frames.clone(),
                "head_shape" => self.head_shape.clone(),
                "factors" => self.factors.clone(),
                "count" => self.count,
                "points" => self.points,
                "body" => self.body,
                "inner" => inner,
                "h" => h,
                "h_last" => self.h_last,
                "time" => self.time,
                "gravity" => s.gravity,
                "wind_x" => wind.x,
                "wind_y" => wind.y,
                "wind_z" => wind.z,
                "turbulence" => s.turbulence,
                "collision_offset" => s.collision_offset,
                "head_friction" => s.head_friction,
                "stretch" => rod.stretch,
                "bend" => rod.bend,
                "twist" => rod.twist,
                "gyration" => rod.gyration,
                "damping" => rod.damping,
                "drag" => rod.drag,
                "drag_viscous" => rod.drag_viscous,
                "pad0" => 0.0f32,
                "pad1" => 0.0f32,
                "pad2" => 0.0f32,
            };
            insert_rigid(&mut p, &now);
            insert_from(&mut p, &start);
            batch.dispatch_folded(&groomer.kernels.sim_step, &p, self.count)?;
            self.h_last = h;

            if interacting {
                batch.clear_buffer(&self.grid.words);
                let mut p = params! {
                    "position" => self.position.clone(),
                    "previous" => self.previous.clone(),
                    "grid_words" => self.grid.words.clone(),
                    "count" => particles,
                };
                insert_rigid(&mut p, &now);
                self.grid.insert(&mut p);
                batch.dispatch_folded(&groomer.kernels.sim_splat, &p, particles)?;
                let mut p = params! {
                    "position" => self.position.clone(),
                    "previous" => self.previous.clone(),
                    "grid_words" => self.grid.words.clone(),
                    "rest_density" => self.grid.rest_density.clone(),
                    "stats" => self.grid.stats.clone(),
                    "count" => particles,
                    "points" => self.points,
                    "volume" => s.volume,
                    "tolerance" => s.volume_tolerance,
                    "friction" => s.friction,
                };
                insert_rigid(&mut p, &now);
                self.grid.insert(&mut p);
                batch.dispatch_folded(&groomer.kernels.sim_volume, &p, particles)?;
            }
            if s.strand_collision && (k + 1) % s.collision_interval.clamp(1, substeps) == 0 {
                self.record_collisions(groomer, &mut batch, &now)?;
            }
            self.time += h * inner as f32;
        }
        if !s.strand_collision {
            self.have_before = false;
        }
        batch.submit();
        self.head = head;
        Ok(())
    }

    /// The colliding strands' collisions, the head at `head`: their samples
    /// counted into the grid's cells, the counts scanned, the samples
    /// scattered into the cells' order, each sample's asks, and the asks
    /// gathered by the points.
    fn record_collisions(
        &mut self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        head: &Rigid,
    ) -> Result<()> {
        let samples = self.colliders * (self.points - 1) * self.per_segment;
        // Last pass's samples become the ones before.
        self.current = 1 - self.current;
        let (now, before) = (
            self.samples[self.current].clone(),
            self.samples[1 - self.current].clone(),
        );
        let cells = &self.cells;
        let cell_count = cells.cells();
        batch.clear_buffer(&cells.counts);
        let mut p = params! {
            "position" => self.position.clone(),
            "previous" => self.previous.clone(),
            "rest" => self.rest.clone(),
            "samples" => now.clone(),
            "sample_data" => self.sample_data.clone(),
            "cell_counts" => cells.counts.clone(),
            "sample_slots" => self.sample_slots.clone(),
            "cell_lo_x" => cells.lo.x,
            "cell_lo_y" => cells.lo.y,
            "cell_lo_z" => cells.lo.z,
            "cell_size" => cells.cell,
            "cell_nx" => cells.dims[0],
            "cell_ny" => cells.dims[1],
            "cell_nz" => cells.dims[2],
            "count" => samples,
            "points" => self.points,
            "per_segment" => self.per_segment,
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        insert_rigid(&mut p, head);
        batch.dispatch_folded(&groomer.kernels.sim_samples, &p, samples)?;
        groomer
            .scan
            .record(batch, &cells.counts, &cells.scan, cell_count + 1)?;
        batch.dispatch_folded(
            &groomer.kernels.sim_sort_samples,
            &params! {
                "samples" => now,
                "samples_before" => before,
                "sample_data" => self.sample_data.clone(),
                "sample_slots" => self.sample_slots.clone(),
                "cell_offsets" => cells.counts.clone(),
                "sorted" => self.sorted.clone(),
                "sorted_index" => self.sorted_index.clone(),
                "count" => samples,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            samples,
        )?;
        batch.dispatch_folded(
            &groomer.kernels.sim_collide,
            &params! {
                "sorted" => self.sorted.clone(),
                "sorted_index" => self.sorted_index.clone(),
                "cell_offsets" => cells.counts.clone(),
                "asks" => self.asks.clone(),
                "count" => samples,
                "cell_nx" => cells.dims[0],
                "cell_ny" => cells.dims[1],
                "cell_nz" => cells.dims[2],
                "radius" => self.radius,
                "reach" => self.radius * 1.75,
                "friction" => self.settings.collision_friction,
                "have_before" => u32::from(self.have_before),
            },
            samples,
        )?;
        // Only the colliding strands' points gather asks.
        let colliding = self.colliders * self.points;
        batch.dispatch_folded(
            &groomer.kernels.sim_collide_apply,
            &params! {
                "position" => self.position.clone(),
                "previous" => self.previous.clone(),
                "asks" => self.asks.clone(),
                "count" => colliding,
                "points" => self.points,
                "per_segment" => self.per_segment,
                "pad0" => 0u32,
            },
            colliding,
        )?;
        self.have_before = true;
        Ok(())
    }

    /// The simulated strands' points, read back: for tests and tools.
    pub fn read_positions(&self, groomer: &GpuGroomer) -> Result<Vec<[f32; 4]>> {
        pollster::block_on(self.position.read(groomer.context()))
    }

    /// The simulated strands' groomed points, read back.
    pub fn read_rest(&self, groomer: &GpuGroomer) -> Result<Vec<[f32; 4]>> {
        pollster::block_on(self.rest.read(groomer.context()))
    }

    /// Where the strands are, and were a substep ago, their frames along
    /// them: for tests to set a scene up.
    #[cfg(test)]
    pub(crate) fn set_state(
        &self,
        groomer: &GpuGroomer,
        positions: &[[f32; 4]],
        previous: &[[f32; 4]],
    ) -> Result<()> {
        self.position.write(
            groomer.context(),
            fabelgeist_gpu::prelude::BufferUpload::from_elements(positions),
        )?;
        self.previous.write(
            groomer.context(),
            fabelgeist_gpu::prelude::BufferUpload::from_elements(previous),
        )?;
        let mut batch = KernelBatch::labelled(groomer.context(), "hair sim state");
        self.record_orient(groomer, &mut batch, &self.head)?;
        batch.submit();
        Ok(())
    }
}

//! Simulation recording and publication after an accepted submission.

use super::collision_progress::CollisionProgress;
use super::{GpuGroomer, GpuSim, insert_from, insert_rigid, params};
use crate::gpu::error::Result;
use crate::rod::Rod;
use crate::sim::{Rigid, blend};
use fabelgeist_compute::{KernelBatch, ScanElementCount};
use fabelgeist_math::Vec3;

impl GpuSim {
    /// Back to the groom, the head where it is.
    pub fn reset(&mut self, groomer: &GpuGroomer, head: Rigid) -> Result<()> {
        let context = groomer.context();
        self.source.ensure_device(context)?;
        let count = self.layout.native_samples();
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
        batch.dispatch_folded(
            &groomer.kernels.sim_reset,
            &p,
            self.layout.sample_invocations(),
        )?;
        let rod = self.record_shape(groomer, &mut batch)?;
        self.record_orient(groomer, &mut batch, &head)?;
        batch.submit()?;
        // The reset pose has no collision samples from a preceding pass.
        self.collisions.progress.forget_previous();
        self.shaped = Some((rod, self.settings.sag_free));
        self.head = head;
        Ok(())
    }

    /// The rods' rest solved for the settings, if they changed.
    pub(super) fn record_shape(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
    ) -> Result<Rod> {
        let s = self.settings;
        let rod = Rod::new(&s);
        if self.shaped == Some((rod, s.sag_free)) {
            return Ok(rod);
        }
        batch.dispatch_folded(
            &groomer.kernels.sim_shape,
            &params! {
                "rest" => self.rest.clone(),
                "shape" => self.shape.clone(),
                "count" => self.layout.native_strands(),
                "points" => self.layout.native_points(),
                "sag_free" => u32::from(s.sag_free),
                "pad0" => 0u32,
                "stretch" => rod.stretch,
                "bend" => rod.bend,
                "twist" => rod.twist,
                "gravity" => s.gravity,
            },
            self.layout.strand_invocations(),
        )?;
        Ok(rod)
    }

    /// Each segment's frame onto where its strand now is, the head at
    /// `head`, not spinning.
    pub(super) fn record_orient(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        head: &Rigid,
    ) -> Result<()> {
        let count = self.layout.native_samples();
        let mut p = params! {
            "rest" => self.rest.clone(),
            "shape" => self.shape.clone(),
            "position" => self.position.clone(),
            "frames" => self.frames.clone(),
            "count" => count,
            "points" => self.layout.native_points(),
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        insert_rigid(&mut p, head);
        batch.dispatch_folded(
            &groomer.kernels.sim_orient,
            &p,
            self.layout.sample_invocations(),
        )?;
        Ok(())
    }

    /// Step `dt` seconds with the head moved to `head`: the substeps are
    /// recorded, not waited for -- the frame drawn next reads them. Each
    /// substep takes the rods through their implicit steps, then keeps the
    /// strands' volume and stops them closing through each other.
    pub fn step(&mut self, groomer: &GpuGroomer, dt: f32, head: Rigid) -> Result<()> {
        self.source.ensure_device(groomer.context())?;
        let dt = dt.min(1.0 / 20.0);
        if dt.is_nan() || dt <= 0.0 {
            return Ok(());
        }
        let s = self.settings;
        let mut batch = KernelBatch::labelled(groomer.context(), "hair sim step");
        let rod = self.record_shape(groomer, &mut batch)?;
        let substeps = s.substeps.max(1);
        let inner = s.solver_substeps.max(1) as u32;
        let h = dt / (substeps as u32 * inner) as f32;
        let from = self.head;
        let mut last_step = self.h_last;
        let mut time = self.time;
        let mut collision_progress = self.collisions.progress;
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
                "count" => self.layout.native_strands(),
                "points" => self.layout.native_points(),
                "body" => self.body,
                "inner" => inner,
                "h" => h,
                "h_last" => last_step,
                "time" => time,
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
            batch.dispatch_folded(
                &groomer.kernels.sim_step,
                &p,
                self.layout.strand_invocations(),
            )?;
            last_step = h;

            if interacting {
                self.record_volume(groomer, &mut batch, &now)?;
            }
            if s.strand_collision && (k + 1) % s.collision_interval.clamp(1, substeps) == 0 {
                self.record_collisions(groomer, &mut batch, &now, &mut collision_progress)?;
            }
            time += h * inner as f32;
        }
        if !s.strand_collision {
            collision_progress.forget_previous();
        }
        batch.submit()?;
        self.shaped = Some((rod, s.sag_free));
        self.h_last = last_step;
        self.time = time;
        self.collisions.progress = collision_progress;
        self.head = head;
        Ok(())
    }

    /// Record the density-grid volume and friction corrections for this pose.
    fn record_volume(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        head: &Rigid,
    ) -> Result<()> {
        let s = self.settings;
        let particles = self.layout.native_samples();
        batch.clear_buffer(&self.grid.words);
        let mut p = params! {
            "position" => self.position.clone(),
            "previous" => self.previous.clone(),
            "grid_words" => self.grid.words.clone(),
            "count" => particles,
        };
        insert_rigid(&mut p, head);
        self.grid.insert(&mut p);
        batch.dispatch_folded(
            &groomer.kernels.sim_splat,
            &p,
            self.layout.sample_invocations(),
        )?;
        let mut p = params! {
            "position" => self.position.clone(),
            "previous" => self.previous.clone(),
            "grid_words" => self.grid.words.clone(),
            "rest_density" => self.grid.rest_density.clone(),
            "stats" => self.grid.stats.clone(),
            "count" => particles,
            "points" => self.layout.native_points(),
            "volume" => s.volume,
            "tolerance" => s.volume_tolerance,
            "friction" => s.friction,
        };
        insert_rigid(&mut p, head);
        self.grid.insert(&mut p);
        batch.dispatch_folded(
            &groomer.kernels.sim_volume,
            &p,
            self.layout.sample_invocations(),
        )?;
        Ok(())
    }

    /// The colliding strands' collisions, the head at `head`: their samples
    /// counted into the grid's cells, the counts scanned, the samples
    /// scattered into the cells' order, each sample's asks, and the asks
    /// gathered by the points.
    fn record_collisions(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        head: &Rigid,
        progress: &mut CollisionProgress,
    ) -> Result<()> {
        let samples = self.collisions.layout.samples();
        // Last pass's samples become the ones before.
        progress.advance();
        let now = self.collisions.samples[progress.current_index()].clone();
        let before = self.collisions.samples[progress.previous_index()].clone();
        let cells = &self.collisions.cells;
        let cell_count = cells.cells();
        batch.clear_buffer(&cells.counts);
        let mut p = params! {
            "position" => self.position.clone(),
            "previous" => self.previous.clone(),
            "rest" => self.rest.clone(),
            "samples" => now.clone(),
            "sample_data" => self.collisions.sample_data.clone(),
            "cell_counts" => cells.counts.clone(),
            "sample_slots" => self.collisions.sample_slots.clone(),
            "cell_lo_x" => cells.lo.x,
            "cell_lo_y" => cells.lo.y,
            "cell_lo_z" => cells.lo.z,
            "cell_size" => cells.cell,
            "cell_nx" => cells.dims[0],
            "cell_ny" => cells.dims[1],
            "cell_nz" => cells.dims[2],
            "count" => samples,
            "points" => self.layout.native_points(),
            "per_segment" => self.collisions.layout.per_segment(),
            "pad0" => 0u32,
            "pad1" => 0u32,
        };
        insert_rigid(&mut p, head);
        batch.dispatch_folded(
            &groomer.kernels.sim_samples,
            &p,
            self.collisions.layout.sample_invocations(),
        )?;
        groomer.scan.record(
            batch,
            &cells.counts,
            &cells.scan,
            ScanElementCount::from_elements(cell_count + 1),
        )?;
        batch.dispatch_folded(
            &groomer.kernels.sim_sort_samples,
            &params! {
                "samples" => now,
                "samples_before" => before,
                "sample_data" => self.collisions.sample_data.clone(),
                "sample_slots" => self.collisions.sample_slots.clone(),
                "cell_offsets" => cells.counts.clone(),
                "sorted" => self.collisions.sorted.clone(),
                "sorted_index" => self.collisions.sorted_index.clone(),
                "count" => samples,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            self.collisions.layout.sample_invocations(),
        )?;
        batch.dispatch_folded(
            &groomer.kernels.sim_collide,
            &params! {
                "sorted" => self.collisions.sorted.clone(),
                "sorted_index" => self.collisions.sorted_index.clone(),
                "cell_offsets" => cells.counts.clone(),
                "asks" => self.collisions.asks.clone(),
                "count" => samples,
                "cell_nx" => cells.dims[0],
                "cell_ny" => cells.dims[1],
                "cell_nz" => cells.dims[2],
                "radius" => self.collisions.radius,
                "reach" => self.collisions.radius * 1.75,
                "friction" => self.settings.collision_friction,
                "have_before" => u32::from(progress.has_previous()),
            },
            self.collisions.layout.sample_invocations(),
        )?;
        // Only the colliding strands' points gather asks.
        let colliding = self.collisions.layout.strands().native_samples();
        batch.dispatch_folded(
            &groomer.kernels.sim_collide_apply,
            &params! {
                "position" => self.position.clone(),
                "previous" => self.previous.clone(),
                "asks" => self.collisions.asks.clone(),
                "count" => colliding,
                "points" => self.layout.native_points(),
                "per_segment" => self.collisions.layout.per_segment(),
                "pad0" => 0u32,
            },
            self.collisions.layout.strands().sample_invocations(),
        )?;
        progress.mark_available();
        Ok(())
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
        batch.submit()?;
        Ok(())
    }
}

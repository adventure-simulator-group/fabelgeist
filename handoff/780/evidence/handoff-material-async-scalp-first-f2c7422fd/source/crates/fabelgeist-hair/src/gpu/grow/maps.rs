//! Native guide-map allocation and candidate assignment.

use super::*;

impl GpuGroomer {
    /// A guide map of `set` into its `guide_of`.
    pub(in crate::gpu) fn record_set_map(
        &mut self,
        batch: &mut KernelBatch,
        set: &mut StrandSet,
        distance: f32,
        mask: f32,
        seed: u32,
    ) -> Result<()> {
        self.record_map(
            batch,
            &set.roots,
            &set.attributes,
            &set.guide_of,
            set.layout,
            set.root_bounds,
            distance,
            mask,
            seed,
        )?;
        set.mapped = true;
        Ok(())
    }

    /// A guide map into `out`: on a grid of `distance` over the roots' box,
    /// each cell's lowest-hashed candidate of each parting group leads it,
    /// and every strand takes the nearest leader around it.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::gpu) fn record_map(
        &mut self,
        batch: &mut KernelBatch,
        roots: &Buffer,
        attributes: &Buffer,
        out: &Buffer,
        layout: StrandLayout,
        bounds: [Vec3; 2],
        distance: f32,
        mask: f32,
        seed: u32,
    ) -> Result<()> {
        let count = layout.native_strands();
        let mut cell = distance.max(0.002);
        let pad = Vec3::splat(0.01);
        let (lo, hi) = (bounds[0] - pad, bounds[1] + pad);
        let extent = hi - lo;
        let dims =
            |cell: f32| [extent.x, extent.y, extent.z].map(|e| ((e / cell).ceil() as u32).max(1));
        while dims(cell).iter().map(|&d| d as u64).product::<u64>() * 5 > 8 << 20 {
            cell *= 1.25;
        }
        let [nx, ny, nz] = dims(cell);
        let leaders = self.scratch.leaders.fit(
            &self.context,
            (nx as u64 * ny as u64 * nz as u64 * 5 * 4).into(),
            "hair guide leaders".into(),
        )?;
        batch.clear_buffer(&leaders);
        let grid = |mut p: PassParameters| {
            p.insert("roots", roots.clone());
            p.insert("strands", attributes.clone());
            p.insert("leaders", leaders.clone());
            p.insert("count", count);
            p.insert("nx", nx);
            p.insert("ny", ny);
            p.insert("nz", nz);
            p.insert("lo_x", lo.x);
            p.insert("lo_y", lo.y);
            p.insert("lo_z", lo.z);
            p.insert("inverse_cell", 1.0 / cell);
            p.insert("mask", mask);
            p.insert("seed", seed);
            p.insert("pad0", 0u32);
            p.insert("pad1", 0u32);
            p
        };
        batch.dispatch_folded(
            &self.kernels.map_candidates,
            &grid(PassParameters::new()),
            layout.strand_invocations(),
        )?;
        batch.dispatch_folded(
            &self.kernels.map_assign,
            &grid(params! { "guide_of" => out.clone() }),
            layout.strand_invocations(),
        )?;
        Ok(())
    }
}

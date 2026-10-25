//! Rest-density grid allocation and recording in the head's metre frame.
use super::*;
impl GpuGroomer {
    pub(super) fn record_rest_density(
        &mut self,
        batch: &mut KernelBatch,
        rest: &Buffer,
        samples: u32,
        lo: Vec3,
        hi: Vec3,
        cell_metres: f32,
    ) -> Result<SimGrid> {
        let context = self.context.clone();
        let mut cell = cell_metres.max(0.002);
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
                words: storage(&context, (cells * 16).into(), "hair sim grid".into())?,
                rest_density: storage(
                    &context,
                    (cells * 4).into(),
                    "hair sim rest density".into(),
                )?,
                stats: storage(
                    &context,
                    16_u64.into(),
                    "hair sim rest density stats".into(),
                )?,
            }
        };
        batch.clear_buffer(&grid.rest_density);
        batch.clear_buffer(&grid.stats);
        let mut p = params! {
            "rest" => rest.clone(),
            "rest_density" => grid.rest_density.clone(),
            "count" => samples,
        };
        grid.insert(&mut p);
        batch.dispatch_folded(&self.kernels.sim_rest_density, &p, samples)?;
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
        Ok(grid)
    }
}

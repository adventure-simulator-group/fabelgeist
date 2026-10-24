//! Native GPU layout preparation for level-of-detail passes.
use super::*;
use fabelgeist_gpu::prelude::{BufferDefinition, BufferUse};
use fabelgeist_compute::SortKeyWidth;
use crate::cards::CardStripLayout;
pub(super) struct CoverDensity {
    pub(super) grid: Grid,
    pub(super) density: Buffer,
}
pub(super) struct CardBuffers {
    pub(super) vertices: Buffer,
    pub(super) indices: Buffer,
}
impl CardBuffers {
    pub(super) fn new(
        context: &fabelgeist_gpu::globals::WgpuContext,
        strips: CardStripLayout,
    ) -> Result<Self> {
        let layout = strips.layout();
        let vertices = Buffer::new(
            context,
            layout.vertices_binding_bytes(),
            BufferDefinition::storage()
                .with_usage(BufferUse::Vertex)
                .with_label("hair card vertices".into()),
        )?;
        let indices = Buffer::new(
            context,
            layout.indices_binding_bytes(),
            BufferDefinition::storage()
                .with_usage(BufferUse::Index)
                .with_label("hair card indices".into()),
        )?;
        Ok(Self { vertices, indices })
    }
}
impl GpuGroomer {
    pub(super) fn record_order(
        &mut self,
        context: &fabelgeist_gpu::globals::WgpuContext,
        batch: &mut KernelBatch,
        groom: &GpuGroom,
    ) -> Result<Buffer> {
        let layout = groom.layout;
        let count = layout.native_strands();
        let items = layout.sort_items();
        // The shuffled order.
        let keys =
            self.scratch
                .keys
                .fit(context, items.word_bytes(), "hair order keys".into())?;
        let order = storage(context, items.word_bytes(), "hair order".into())?;
        batch.dispatch_folded(
            &self.kernels.order_keys,
            &params! {
                "strands" => groom.attributes.clone(),
                "keys" => keys.clone(),
                "order" => order.clone(),
                "count" => count,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            layout.strand_invocations(),
        )?;
        self.sort_scratch.ensure(context, items)?;
        self.sort
            .record(batch, &keys, &order, &mut self.sort_scratch, items, SortKeyWidth::from(u32::BITS))?;

        Ok(order)
    }
    pub(super) fn record_cover_density(
        &mut self,
        batch: &mut KernelBatch,
        groom: &GpuGroom,
        bounds: [Vec3; 2],
        reach: f32,
    ) -> Result<CoverDensity> {
        let context = self.context.clone();
        // Hair over the skin: its density on a grid of four millimetres.
        let pad = Vec3::splat(reach + 0.004);
        let grid = Grid::over(bounds[0] - pad, bounds[1] + pad, 0.004);
        let grid = if grid.cells() > 32 << 20 {
            let cell = ((bounds[1] - bounds[0] + pad * 2.0).max_component()) / 300.0;
            Grid::over(bounds[0] - pad, bounds[1] + pad, cell)
        } else {
            grid
        };
        let head = upload(&context, &[[0.0f32; 4]; 17], "hair no body".into())?;
        self.record_density(batch, groom, grid, 1.0, 0, &head)?;
        let density = self.scratch.density.fit(
            &context,
            (grid.cells() as u64 * 4).into(),
            "hair density".into(),
        )?;
        Ok(CoverDensity { grid, density })
    }
}

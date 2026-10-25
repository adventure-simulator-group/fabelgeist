//! Physical scalp sampling and its strand/texel recording passes.

use super::super::upload;
use super::{
    AT_REACH, AT_SKIN, BARE_REACH, HairGpu, LOCK_SPREAD, LOCK_WIDTH, REACH, SHADOW_BLOCK,
    SHADOW_REACH, STEP, STRAND_SPREAD, ScalpTexelSpan, ScalpTextureLayout, params,
};
use crate::hair::scalp_map::ScalpMap;
use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferElementCount, InvocationCount, PassParameter};
use fabelgeist_hair::Head;
use fabelgeist_hair::gpu::{GpuGroom, head_shape};

/// Physical sampling in texels and the number of shadow blocks per row.
pub(super) struct Grid<'a> {
    layout: &'a ScalpTextureLayout,
    block_side: ScalpTexelSpan,
    blocks: ShadowBlockAxisCount,
    bare_reach: ScalpTexelSpan,
}

/// Positive shadow blocks along one texture axis, bounded by its texel side.
#[derive(Clone, Copy)]
struct ShadowBlockAxisCount(u32);

impl ShadowBlockAxisCount {
    fn covering(side: ScalpTexelSpan, block_side: ScalpTexelSpan) -> Self {
        // Both spans are positive, so this ceiling is at least one block.
        Self(side.native_texels().div_ceil(block_side.native_texels()))
    }

    fn elements(self) -> BufferElementCount {
        // The fixed base has at most 2048 texels, hence at most 2048 blocks.
        BufferElementCount::from(self.0 * self.0)
    }
}

impl From<ShadowBlockAxisCount> for PassParameter {
    fn from(blocks: ShadowBlockAxisCount) -> Self {
        Self::Unsigned(blocks.0)
    }
}

impl<'a> Grid<'a> {
    pub(super) fn new(layout: &'a ScalpTextureLayout, map: &ScalpMap) -> Self {
        let texel = map.texel(layout.base().side());
        let block_side = ScalpTexelSpan::at_least_one(
            (SHADOW_BLOCK.metres() / texel.metres()).round() as u32,
        );
        let blocks = ShadowBlockAxisCount::covering(layout.base().side(), block_side);
        let bare_reach = ScalpTexelSpan::at_least_one(
            (BARE_REACH.metres() / texel.metres()).round() as u32,
        );
        Self {
            layout,
            block_side,
            blocks,
            bare_reach,
        }
    }

    pub(super) fn shadow_elements(&self) -> BufferElementCount {
        self.blocks.elements()
    }

    /// Record every strand of `groom`, grown on `bust`, drawn into
    /// `topmost`, or marking the blocks it shadows.
    pub(super) fn draw(
        &self,
        gpu: &HairGpu,
        batch: &mut KernelBatch<'_>,
        groom: &GpuGroom,
        bust: &Head,
        map: &ScalpMap,
        [topmost, shadowed]: [&Buffer; 2],
    ) -> Result<()> {
        let context = gpu.context();
        let kernels = &gpu.bake;
        let [center, axis, forward, side] = [map.center, map.axis, map.forward, map.side];
        let (block_side, blocks) = (self.block_side, self.blocks);
        batch.dispatch_folded(
            &kernels.draw,
            &params! {
                "positions" => groom.positions().clone(),
                "strands" => groom.attributes().clone(),
                "head_shape" => upload(context, &head_shape(bust), "hair bust shape")?,
                "topmost" => topmost.clone(),
                "shadowed" => shadowed.clone(),
                "center_x" => center.x,
                "center_y" => center.y,
                "center_z" => center.z,
                "rim" => map.rim,
                "axis_x" => axis.x,
                "axis_y" => axis.y,
                "axis_z" => axis.z,
                "size" => self.layout.base().side(),
                "forward_x" => forward.x,
                "forward_y" => forward.y,
                "forward_z" => forward.z,
                "count" => groom.layout().native_strands(),
                "side_x" => side.x,
                "side_y" => side.y,
                "side_z" => side.z,
                "points" => groom.layout().native_points(),
                "step" => STEP.metres(),
                "reach" => REACH.metres(),
                "shadow_reach" => SHADOW_REACH.metres(),
                "at_skin" => AT_SKIN.ratio(),
                "at_reach" => AT_REACH.ratio(),
                "lock_spread" => LOCK_SPREAD.ratio(),
                "strand_spread" => STRAND_SPREAD.ratio(),
                "lock_width" => LOCK_WIDTH.metres(),
                "block_side" => block_side,
                "blocks" => blocks,
                "pad0" => 0u32,
                "pad1" => 0u32,
            },
            groom.layout().strand_invocations(),
        )?;
        Ok(())
    }

    /// Record which texels are bare, into `bare[0]`, and their share round
    /// each texel, blurred twice along rows and columns, back into it.
    pub(super) fn bare(
        &self,
        gpu: &HairGpu,
        batch: &mut KernelBatch<'_>,
        [topmost, shadowed]: [&Buffer; 2],
        bare: &[Buffer; 2],
    ) -> Result<()> {
        let kernels = &gpu.bake;
        let (block_side, blocks, bare_reach) = (self.block_side, self.blocks, self.bare_reach);
        let texel_count = self.layout.base().elements();
        batch.dispatch_folded(
            &kernels.bare,
            &params! {
                "topmost" => topmost.clone(),
                "shadowed" => shadowed.clone(),
                "bare" => bare[0].clone(),
                "size" => self.layout.base().side(),
                "block_side" => block_side,
                "blocks" => blocks,
                "pad0" => 0u32,
            },
            InvocationCount::from(texel_count),
        )?;
        // Twice over, along rows then columns each time, so the nearest
        // scalp counts most and the share falls smoothly.
        for pass in 0..4u32 {
            let (from, to) = (&bare[(pass % 2) as usize], &bare[((pass + 1) % 2) as usize]);
            batch.dispatch_folded(
                &kernels.blur,
                &params! {
                    "blur_in" => from.clone(),
                    "blur_out" => to.clone(),
                    "size" => self.layout.base().side(),
                    "stride" => if pass % 2 == 0 { 1 } else { self.layout.base().side_texels() },
                    "reach" => bare_reach,
                    "pad0" => 0u32,
                },
                InvocationCount::from(texel_count),
            )?;
        }
        Ok(())
    }
}

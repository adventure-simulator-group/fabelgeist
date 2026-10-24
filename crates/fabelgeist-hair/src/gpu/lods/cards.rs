//! Compact locks into checked native strip geometry.

use super::prepare::CardBuffers;
use super::{GpuCards, GpuGroom, GpuGroomer, Grid, params};
use crate::cards::{AtlasVariantCount, CardCount, CardError, CardSegments, CardSettings,
    CardStripLayout};
use crate::gpu::StrandLayout;
use crate::gpu::error::Result;
use crate::parameters::StrandCount;
use fabelgeist_compute::{KernelBatch, ScanElementCount};
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferElementCount, PassParameters,
    WgpuContext};
use fabelgeist_math::Vec3;

/// Source resources and native uniforms from one level-preparation batch.
pub(super) struct CardPass<'a> {
    pub groom: &'a GpuGroom,
    pub settings: &'a CardSettings,
    pub center: Vec3,
    pub grid: Grid,
    pub visibility: &'a Buffer,
    pub head: &'a Buffer,
    pub body: u32,
    pub variants: AtlasVariantCount,
}

struct CardWork {
    source: StrandLayout,
    segments: CardSegments,
    min_strands: StrandCount,
}

struct CardCompaction {
    locks: Buffer,
    flags: Buffer,
    cards: CardCount,
}

impl CardWork {
    fn new(source: StrandLayout, settings: &CardSettings) -> Result<Self> {
        let segments = settings.segments.clamp(CardSegments::from_count(1), CardSegments::from_count(31));
        let work = Self { source, segments, min_strands: settings.min_strands };
        BufferElementCount::from(source.native_strands())
            .checked_mul(work.lock_stride())
            .ok_or(CardError::LockAddresses { strands: source.strands(), segments })?;
        source.native_strands().checked_add(1)
            .ok_or(CardError::LockAddresses { strands: source.strands(), segments })?;
        Ok(work)
    }

    fn lock_stride(&self) -> u32 {
        // LOCKS has four header words and four words per native row.
        4 + (self.segments.count() as u32 + 1) * 4
    }

    fn locks_bytes(&self) -> BufferByteLength {
        (u64::from(self.source.native_strands()) * u64::from(self.lock_stride()) * size_of::<u32>() as u64).into()
    }

    fn scan_elements(&self) -> ScanElementCount {
        ScanElementCount::from_elements(self.source.native_strands() + 1)
    }

    fn parameters(&self, mut parameters: PassParameters) -> PassParameters {
        parameters.insert("count", self.source.native_strands());
        parameters.insert("points", self.source.native_points());
        parameters.insert("segments", self.segments.count() as u32);
        // LOCKS uses signed counts. Its admitted address extent is below
        // i32::MAX, so a larger threshold still leaves every lock out.
        parameters.insert("min_strands", self.min_strands.count().min(i32::MAX as usize) as u32);
        parameters
    }

    fn strips(&self, cards: CardCount) -> Result<CardStripLayout> {
        if cards.count() > self.source.strands().count() {
            return Err(CardError::Compaction { cards, strands: self.source.strands() }.into());
        }
        Ok(CardStripLayout::new(cards, self.segments)?.admit_native_writer()?)
    }
}

impl GpuGroomer {
    pub(super) fn record_cards<'a>(
        &mut self,
        context: &'a WgpuContext,
        mut batch: KernelBatch<'a>,
        pass: CardPass<'_>,
    ) -> Result<(GpuCards, KernelBatch<'a>)> {
        let work = CardWork::new(pass.groom.layout, pass.settings)?;
        let guide_of = self.scratch.card_guides.fit(context, work.source.strand_word_bytes(), "hair card locks".into())?;
        self.record_map(&mut batch, &pass.groom.roots, &pass.groom.attributes, &guide_of,
            work.source, pass.groom.root_bounds, pass.settings.size, 1.0, 0xCA4D)?;
        let compacted = self.compact_cards(context, batch, &pass, &work, &guide_of)?;
        let mut batch = KernelBatch::labelled(context, "hair cards");
        batch.dispatch_folded(
            &self.kernels.card_spread,
            &work.parameters(params! {
                "positions" => pass.groom.positions.clone(),
                "guide_of" => guide_of,
                "locks" => compacted.locks.clone(),
                "strands" => pass.groom.attributes.clone(),
                "center_x" => pass.center.x,
                "center_y" => pass.center.y,
                "center_z" => pass.center.z,
                "pad0" => 0u32,
            }),
            work.source.strand_invocations(),
        )?;
        let cards = self.emit_cards(context, &mut batch, pass, &work, compacted)?;
        Ok((cards, batch))
    }

    fn compact_cards(
        &mut self,
        context: &WgpuContext,
        mut batch: KernelBatch,
        pass: &CardPass<'_>,
        work: &CardWork,
        guide_of: &Buffer,
    ) -> Result<CardCompaction> {
        let locks = self.scratch.locks.fit(context, work.locks_bytes(), "hair card lock stats".into())?;
        batch.clear_buffer(&locks);
        batch.dispatch_folded(
            &self.kernels.card_accumulate,
            &work.parameters(params! {
                "positions" => pass.groom.positions.clone(),
                "guide_of" => guide_of.clone(),
                "locks" => locks.clone(),
            }),
            work.source.strand_invocations(),
        )?;
        let flags = self.scratch.flags.fit(context, work.scan_elements().byte_length(), "hair card flags".into())?;
        batch.clear_buffer(&flags);
        batch.dispatch_folded(
            &self.kernels.card_flags,
            &work.parameters(params! { "locks" => locks.clone(), "flags" => flags.clone() }),
            work.source.strand_invocations(),
        )?;
        self.scan_scratch.ensure(context, work.scan_elements())?;
        self.scan.record(&mut batch, &flags, &self.scan_scratch, work.scan_elements())?;
        let cards = CardCount::from_count(self.read_word(batch, &flags, work.source.native_strands())? as usize);
        Ok(CardCompaction { locks, flags, cards })
    }

    fn emit_cards(
        &self,
        context: &WgpuContext,
        batch: &mut KernelBatch,
        pass: CardPass<'_>,
        work: &CardWork,
        compacted: CardCompaction,
    ) -> Result<GpuCards> {
        let strips = work.strips(compacted.cards)?;
        let CardBuffers { vertices, indices } = CardBuffers::new(context, strips)?;
        let mut parameters = work.parameters(params! {
            "locks" => compacted.locks,
            "slots" => compacted.flags,
            "strands" => pass.groom.attributes.clone(),
            "grid_values" => pass.visibility.clone(),
            "head_shape" => pass.head.clone(),
            "card_vertices" => vertices.clone(),
            "card_indices" => indices.clone(),
            "body" => pass.body,
            "center_x" => pass.center.x,
            "center_y" => pass.center.y,
            "center_z" => pass.center.z,
            "size" => pass.settings.size,
            "width_scale" => pass.settings.width_scale,
            "variants" => pass.variants.count(),
            "pad0" => 0u32,
            "pad1" => 0u32,
        });
        pass.grid.insert(&mut parameters, work.source.native_strands());
        batch.dispatch_folded(&self.kernels.card_build, &parameters, work.source.strand_invocations())?;
        Ok(GpuCards { strips, device: context.device.as_ref().clone(), vertices, indices })
    }
}

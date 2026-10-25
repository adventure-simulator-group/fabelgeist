//! Native level records retain their checked shape and allocating device.

use super::super::error::{HairGpuError, Result};
use super::super::head::GpuHeadCover;
use super::super::StrandLayout;
use crate::cards::{CardAtlas, CardLayout, CardStripLayout};
use crate::lod::{CardLevelCount, LodCounts, StrandLevelCount};
use crate::parameters::StrandCount;
use fabelgeist_gpu::prelude::{Buffer, WgpuContext};
use fabelgeist_math::Vec3;
use std::sync::Arc;

/// One immutable native strand level in the groom's shuffled order.
/// Positions include sky visibility in `w`; infos have one
/// [`crate::lod::StrandInfo`] per strand.
pub struct GpuLevel {
    pub(in crate::gpu) layout: StrandLayout,
    pub(in crate::gpu) device: wgpu::Device,
    pub(in crate::gpu) positions: Buffer,
    pub(in crate::gpu) infos: Buffer,
}

/// One cut of cards: [`crate::cards::CardVertex`] vertices and a triangle list.
pub struct GpuCards {
    pub(in crate::gpu) strips: CardStripLayout,
    pub(in crate::gpu) device: wgpu::Device,
    pub(in crate::gpu) vertices: Buffer,
    pub(in crate::gpu) indices: Buffer,
}

/// A groom prepared on one device, with at least its original strand level.
pub struct GpuLods {
    pub(in crate::gpu) levels: Vec<Arc<GpuLevel>>,
    pub(in crate::gpu) cards: Vec<GpuCards>,
    pub(in crate::gpu) atlas: Arc<CardAtlas>,
    pub(in crate::gpu) bounds: [Vec3; 2],
    pub(in crate::gpu) center: Vec3,
    pub(in crate::gpu) cover: Option<GpuHeadCover>,
}

impl GpuLevel {
    pub fn layout(&self) -> StrandLayout {
        self.layout
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn positions(&self) -> &Buffer {
        &self.positions
    }

    pub fn infos(&self) -> &Buffer {
        &self.infos
    }

    pub(in crate::gpu) fn ensure_device(&self, context: &WgpuContext) -> Result<()> {
        if self.device() != context.device.as_ref() {
            return Err(HairGpuError::DeviceMismatch);
        }
        Ok(())
    }
}

impl GpuCards {
    pub fn layout(&self) -> CardLayout {
        self.strips.layout()
    }

    pub fn strips(&self) -> CardStripLayout {
        self.strips
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn vertices(&self) -> &Buffer {
        &self.vertices
    }

    pub fn indices(&self) -> &Buffer {
        &self.indices
    }
}

impl GpuLods {
    pub fn device(&self) -> &wgpu::Device {
        self.levels[0].device()
    }

    pub fn levels(&self) -> &[Arc<GpuLevel>] {
        &self.levels
    }

    pub fn cards(&self) -> &[GpuCards] {
        &self.cards
    }

    pub fn atlas(&self) -> &Arc<CardAtlas> {
        &self.atlas
    }

    pub fn bounds(&self) -> [Vec3; 2] {
        self.bounds
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    /// Per-head-vertex hair cover prepared with this groom.
    pub fn cover(&self) -> Option<&GpuHeadCover> {
        self.cover.as_ref()
    }

    pub fn strand_count(&self) -> StrandCount {
        self.levels[0].layout.strands()
    }

    /// Populations available to the level-of-detail policy.
    pub fn counts(&self) -> LodCounts {
        LodCounts {
            strands: self.strand_count(),
            levels: StrandLevelCount::from_count(self.levels.len()),
            cards: CardLevelCount::from_count(self.cards.len()),
        }
    }
}

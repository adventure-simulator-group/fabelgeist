//! Hair's levels of detail as triangles, built on the GPU.
//!
//! Cards and strand ribbons are both strips: rows of two points, one either
//! side of the strip's middle, joined by straight segments. Cards come from
//! the hair crate's cuts, already strips; strands become ribbons, each point
//! widened across the strand, square to the direction out from the head's
//! centre, as the hair crate's cards lie, and tapering to the tip. A level
//! drawn with a share of its strands draws them wider by the inverse of
//! that share, so the head's coverage holds. A minimum width before taper keeps
//! base ribbons visible without the hair renderer's sub-pixel coverage.
//!
//! The hair crate keeps a strand's points and a card's spine outside the
//! body, but over a rounded shoulder or skull a straight segment between two
//! rows just outside the skin passes under it in the middle -- the longer
//! the segment, the deeper: the coarse levels most. So segments are subdivided
//! towards [`LONGEST_SEGMENT`] using the longest strand as an estimate,
//! within each detail level's piece budget, and every row near the
//! skin is moved off it by [`CLEARANCE`] and by as much as the strip sags
//! across its width over the tightest curve hair lies on. A row's lift
//! spreads to its neighbours, tapering over [`LIFT_SPREAD`] rows, so a strip
//! bends off the skin rather than in a sawtooth. The hair crate turns a card
//! to face away from the head's centre, right for a card hanging beside the
//! head; a card lying on a shoulder would stand on its edge there, a fin, so
//! near the skin each row turns to lie flat on it instead, fully within
//! [`LIE_FLAT_REACH`] of touching.
//!
//! One invocation builds one strip, into one row of the output buffers per
//! row of the strip, both edges of a row side by side.

use std::num::NonZeroU32;
use std::sync::Arc;

use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_hair::cards::AtlasVariantCount;
use fabelgeist_hair::gpu::error::HairGpuError;
use fabelgeist_hair::gpu::wgsl::common::{HEAD, MATH};
use fabelgeist_hair::gpu::{GpuCards, GpuLevel};
use fabelgeist_hair::material::Taper;
use fabelgeist_hair::parameters::StrandCount;
use fabelgeist_math::Vec3;
use fabelgeist_math::units::{Metres, Ratio};
use fabelgeist_mesh::{MeshStripCount, MeshStripLayout, MeshStripRows};

use geometry::StripBuffers;
pub use geometry::GpuGeometry;
use wgsl::{BINDINGS, CARD_ROWS, RIBBON_ROWS, WRAP};

use super::{HairGpu, KernelSource, params, upload};

mod geometry;
mod wgsl;

/// How far hair is kept off the skin, just above its scalp cap.
const CLEARANCE: Metres = Metres::from_metres(0.0006);
/// Target straight-segment length for the bounded subdivision policy.
const LONGEST_SEGMENT: Metres = Metres::from_metres(0.01);
/// Radius of the tightest curve hair lies over: the neck.
const TIGHTEST_CURVE: Metres = Metres::from_metres(0.05);
/// Rows either side a row's lift spreads to, tapering.
const LIFT_SPREAD: u32 = 4;
/// Reach over which a row turns to lie flat against the skin.
const LIE_FLAT_REACH: Metres = Metres::from_metres(0.03);
/// Minimum strand-ribbon width before its root-to-tip taper.
const MIN_RIBBON_WIDTH: Metres = Metres::from_metres(0.0004);
/// Maximum subdivision of a card segment.
const MOST_CARD_PIECES: u32 = 16;
/// Maximum subdivision at the coarse ribbon levels drawn far from the head.
const MOST_RIBBON_PIECES: u32 = 8;
/// Samples of the strands' taper from root to tip, blended between.
const TAPER_SAMPLES: usize = 33;

pub(super) struct StripKernels {
    cards: Arc<Kernel>,
    ribbons: Arc<Kernel>,
}

/// Source skin and the immutable groom's allocating device and length.
pub(crate) struct StripSource<'a> {
    pub(crate) body: &'a Buffer,
    pub(crate) device: &'a wgpu::Device,
    pub(crate) longest: Metres,
}

impl StripKernels {
    pub(super) fn new(source: &KernelSource) -> Result<Self> {
        let kernel = |rows: &str| format!("{BINDINGS}{rows}{WRAP}{MATH}{HEAD}");
        Ok(Self {
            cards: source.compile(&kernel(CARD_ROWS))?,
            ribbons: source.compile(&kernel(RIBBON_ROWS))?,
        })
    }
}

impl StripSource<'_> {
    /// The numerical subdivision policy, bounded before native row admission.
    fn pieces(&self, rows: MeshStripRows, most: u32) -> Result<NonZeroU32> {
        let segment = self.longest.metres() / (rows.native_rows() - 1) as f32;
        let pieces = ((segment / LONGEST_SEGMENT.metres()).ceil() as u32).clamp(1, most);
        Ok(NonZeroU32::try_from(pieces)?)
    }

    /// Rows a ribbon is drawn with, including the preview's subdivision.
    pub(crate) fn ribbon_rows(&self, level: &GpuLevel) -> Result<MeshStripRows> {
        let rows = MeshStripRows::try_from(level.layout().native_points().max(2) as usize)?;
        Ok(rows.subdivided(self.pieces(rows, MOST_RIBBON_PIECES)?)?)
    }

    /// The cut `cards` as strips, their atlas `variants` wide.
    pub(crate) fn cards(
        &self,
        gpu: &HairGpu,
        cards: &GpuCards,
        variants: AtlasVariantCount,
    ) -> Result<GpuGeometry> {
        if self.device != gpu.context().device.as_ref()
            || cards.device() != gpu.context().device.as_ref()
        {
            return Err(HairGpuError::DeviceMismatch.into());
        }
        let source = cards.strips().topology();
        let pieces = self.pieces(source.rows(), MOST_CARD_PIECES)?;
        let layout = MeshStripLayout::new(source.strips(), source.rows().subdivided(pieces)?)?;
        let p = params! {
            "card_vertices" => cards.vertices().clone(),
            "source_rows" => source.rows(),
            "variants" => variants.count() as f32,
            "pad2" => 0u32,
        };
        self.build(gpu, &gpu.strips.cards, p, layout, pieces)
    }

    /// The first `drawn` strands as ribbons, widened to preserve coverage.
    pub(crate) fn ribbons(
        &self,
        gpu: &HairGpu,
        level: &GpuLevel,
        drawn: StrandCount,
        widen: Ratio,
        center: Vec3,
    ) -> Result<GpuGeometry> {
        if self.device != gpu.context().device.as_ref()
            || level.device() != gpu.context().device.as_ref()
        {
            return Err(HairGpuError::DeviceMismatch.into());
        }
        let prefix = level.layout().prefix(drawn)?;
        let source_rows = MeshStripRows::try_from(prefix.native_points().max(2) as usize)?;
        let pieces = self.pieces(source_rows, MOST_RIBBON_PIECES)?;
        let layout = MeshStripLayout::new(
            MeshStripCount::from(prefix.native_strands()),
            source_rows.subdivided(pieces)?,
        )?;
        let taper = Taper::default();
        let samples: Vec<f32> = (0..TAPER_SAMPLES)
            .map(|k| taper.at(k as f32 / (TAPER_SAMPLES - 1) as f32))
            .collect();
        let p = params! {
            "level_positions" => level.positions().clone(),
            "level_infos" => level.infos().clone(),
            "taper" => upload(gpu.context(), &samples, "hair taper")?,
            "source_rows" => prefix.native_points(),
            "widen" => widen.ratio(),
            "min_width" => MIN_RIBBON_WIDTH.metres(),
            "taper_samples" => TAPER_SAMPLES as u32,
            "center_x" => center.x,
            "center_y" => center.y,
            "center_z" => center.z,
        };
        self.build(gpu, &gpu.strips.ribbons, p, layout, pieces)
    }

    fn build(
        &self,
        gpu: &HairGpu,
        kernel: &Kernel,
        mut p: super::Params,
        layout: MeshStripLayout,
        pieces: NonZeroU32,
    ) -> Result<GpuGeometry> {
        let context = gpu.context();
        let StripBuffers { geometry, lifts } = StripBuffers::new(context, layout)?;
        p.insert("positions", geometry.positions().clone());
        p.insert("normals", geometry.normals().clone());
        p.insert("tangents", geometry.tangents().clone());
        p.insert("texcoords", geometry.texcoords().clone());
        p.insert("lifts", lifts);
        p.insert("head_shape", self.body.clone());
        p.insert("strips", layout.strips());
        p.insert("rows", layout.rows());
        p.insert("pieces", pieces.get());
        p.insert("spread", LIFT_SPREAD);
        p.insert("clearance", CLEARANCE.metres());
        p.insert("tightest_curve", TIGHTEST_CURVE.metres());
        p.insert("lie_flat_reach", LIE_FLAT_REACH.metres());
        p.insert("pad0", 0u32);
        p.insert("pad1", 0u32);
        let mut batch = KernelBatch::labelled(context, "hair strips");
        batch.dispatch_folded(kernel, &p, layout.strips().into())?;
        batch.submit()?;
        Ok(geometry)
    }
}

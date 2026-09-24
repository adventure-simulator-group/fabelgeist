//! Hand and foot armor on the device: mittens, sabatons and boots.
//!
//! The mitten's cuff and finger lames are charts in the hand frame; its thumb
//! is a capped chart in a thumb frame of its own. Sabaton lames are foot
//! charts thickened radially about the sole; their toe cap and the leather
//! boot are coordinate shells whose angular samples the host takes from the
//! topology alone.

use super::ArmorGpu;
use super::chart::{ChartBoundary, ChartKernel, PlateChart};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use crate::{GenerateError, LimbArmorDesign};

/// A shape's charts or shells and its design floats.
pub(crate) struct ExtremityShape {
    pub part: PartRecipe,
    pub design: Vec<f32>,
}

pub(super) fn chart_kernel(
    gpu: &ArmorGpu,
    design: &str,
    shape: &str,
) -> Result<ChartKernel, GenerateError> {
    ChartKernel::new(gpu, &format!("{design}{shape}"))
}

/// A thin chart with no shell constants, in the part's first frame.
pub(super) fn chart(
    rows: usize,
    boundary: ChartBoundary,
    thickness: f32,
    extrusion: Extrusion,
    fluting: Option<crate::PlateFluting>,
    span: [f32; 2],
) -> PlateChart {
    PlateChart {
        rows,
        boundary,
        thickness,
        extrusion,
        origin: [0.0; 3],
        axis: [0.0, 1.0, 0.0],
        fluting,
        span,
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
    }
}

/// The shells of a hand or foot design, before any fit.
pub(crate) fn shape(
    gpu: &ArmorGpu,
    design: &LimbArmorDesign,
) -> Result<ExtremityShape, GenerateError> {
    design.validate()?;
    crate::device_support::on_device(design.device_unsupported())?;
    match design {
        LimbArmorDesign::MittenGauntlet(d) => super::mitten::gauntlet(gpu, d),
        LimbArmorDesign::Sabaton(d) => super::footwear::sabaton(gpu, d),
        LimbArmorDesign::LeatherBoot(d) => super::footwear::boot(gpu, d),
        _ => Err(GenerateError::InvalidSurface),
    }
}

/// Record a hand or foot design into a new device part. `frames[0]` places
/// the hand or foot; a mitten's thumb is placed by `frames[1]`.
pub fn record_extremity_armor(
    gpu: &ArmorGpu,
    batch: &mut fabelgeist_compute::KernelBatch,
    design: &LimbArmorDesign,
    frames: &[&fabelgeist_gpu::prelude::Buffer],
) -> Result<super::DevicePart, GenerateError> {
    let ExtremityShape { part, design } = shape(gpu, design)?;
    part.record(gpu, batch, &design, frames)
}

//! Helmets on the device: the skulls, brims, skirts, cheeks and visors of
//! every helmet family except the close helmet and the coif, which are
//! fitted to measured sections of their own.
//!
//! The host lists every carrier vertex by what it is -- a dome ring at a
//! latitude and meridian, a brim ring, a skirt row -- and the triangles
//! joining them, from the design alone. One kernel evaluates every vertex of
//! every helmet in the head frame, including the per-vertex remaps that
//! follow a dome (the barbute's bowl table, the sallet's front arc).

mod burgonet;
mod codes;
mod parts;
mod shape;
mod surface;

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::coord::CoordKernel;
use super::{ArmorGpu, BuiltPart, DevicePart};
use crate::{GenerateError, HelmetDesign, PartFrame};

fn recipe(
    gpu: &ArmorGpu,
    design: &HelmetDesign,
) -> Result<(super::recipe::PartRecipe, Vec<f32>), GenerateError> {
    design.validate()?;
    crate::device_support::on_device(design.device_unsupported())?;
    let kernel = CoordKernel::new(gpu, &shape::source())?;
    parts::parts(design)?.recipe(&kernel)
}

/// Record a helmet, placed by the head frame at the start of `frame`, into
/// a new device part. Close helmets and coifs are recorded by
/// [`super::record_close_helmet`] and [`super::record_coif`] instead.
pub fn record_helmet(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &HelmetDesign,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    let (part, design) = recipe(gpu, design)?;
    part.record(gpu, batch, &design, &[frame])
}

/// Generate a helmet placed by the host frame `fit`.
pub fn generate_helmet_on(
    gpu: &ArmorGpu,
    design: &HelmetDesign,
    fit: &PartFrame,
) -> Result<BuiltPart, GenerateError> {
    let (part, design) = recipe(gpu, design)?;
    part.build(gpu, &design, fit)
}

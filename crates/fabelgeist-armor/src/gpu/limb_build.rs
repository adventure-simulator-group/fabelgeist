//! Entry points for plain limb charts and shared fitted shoulder carriers.
use super::limb::{LimbShape, shape};
use super::{ArmorGpu, BuiltPart};
use crate::{GenerateError, LimbArmorDesign, PartFrame};
use fabelgeist_compute::KernelBatchLabel;
use fabelgeist_gpu::prelude::BufferUpload;

/// Record a limb design's charts, placed by the part frame at the start of
/// `frame`, into a new device part.
pub fn record_limb_armor(
    gpu: &ArmorGpu,
    batch: &mut fabelgeist_compute::KernelBatch,
    design: &LimbArmorDesign,
    frame: &fabelgeist_gpu::prelude::Buffer,
) -> Result<super::DevicePart, GenerateError> {
    if let LimbArmorDesign::Pauldron(d) = design {
        return super::pauldron::DevicePauldronCarrier::record(gpu, batch, d, frame)?
            .record_plates(gpu, batch, d);
    }
    let LimbShape { part, design } = shape(gpu, design)?;
    part.record(gpu, batch, &design, &[frame])
}

/// Generate a limb plate on the device, placed by `fit`.
pub fn generate_limb_armor_on(
    gpu: &ArmorGpu,
    design: &LimbArmorDesign,
    fit: &PartFrame,
) -> Result<BuiltPart, GenerateError> {
    if let LimbArmorDesign::Pauldron(d) = design {
        fit.validate()?;
        let frame = gpu.upload(BufferUpload::from_elements(&super::recipe::frame_words(
            fit,
        )))?;
        let mut batch = gpu.batch(KernelBatchLabel::from("pauldron"));
        let carrier = super::pauldron::DevicePauldronCarrier::record(gpu, &mut batch, d, &frame)?;
        let mut part = carrier.record_plates(gpu, &mut batch, d)?;
        part.record_shells(gpu, &mut batch)?;
        batch.submit();
        if gpu.read::<u32>(&carrier.status)?[0] != 0 {
            return Err(GenerateError::InvalidSurface);
        }
        return part.read(gpu);
    }
    let LimbShape { part, design } = shape(gpu, design)?;
    part.build(gpu, &design, fit)
}

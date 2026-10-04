//! Sample the opening's signed trim domain on the unchanged carrier.
use super::{
    kernels::{Params, dispatch},
    plates::Plate,
};
use crate::{GenerateError, gpu::ArmorGpu};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

impl Plate {
    pub(super) fn record_arm_trim(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        frame: &Buffer,
        rear: bool,
    ) -> Result<(), GenerateError> {
        dispatch(
            gpu,
            batch,
            include_str!("arm_trim.wgsl"),
            &[],
            Params {
                count: self.count(),
                width: self.width(),
                rear,
                ..Params::default()
            },
            &[
                ("plate", frame),
                ("columns", &gpu.upload(&self.topology.columns)?),
                ("distances", &self.arm_distances),
            ],
            self.count(),
        )
    }
}

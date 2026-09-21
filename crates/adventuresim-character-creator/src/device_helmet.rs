//! Helmets fitted on the device: the head frame, then the helmet in it.

use adventuresim_armor_model::{HelmetDesign, record_helmet};
use anyhow::Result;
use fabelgeist_compute::KernelBatch;

use crate::armor_frames::FitRegion;
use crate::device_frames::DeviceWearer;
use crate::device_piece::DeviceRecording;

impl DeviceWearer<'_> {
    /// Record a helmet in the wearer's head frame, short of thickening.
    pub fn record_helmet(
        &self,
        batch: &mut KernelBatch,
        design: &HelmetDesign,
    ) -> Result<DeviceRecording> {
        let frame = self.record_frame(batch, FitRegion::Head)?;
        let part = record_helmet(self.gpu, batch, design, &frame.frame)?;
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, FitRegion::Head)],
            checks: Vec::new(),
        })
    }
}

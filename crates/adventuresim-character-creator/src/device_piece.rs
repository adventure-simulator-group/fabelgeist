//! What recording a fitted piece on the device leaves behind.

use fabelgeist_armor::{ArmorGpu, BuiltPart, DevicePart};

use crate::armor_frames::FitRegion;
use crate::device_frames::DeviceFrame;

/// A piece recorded on the device, short of thickening, with the frames
/// that place it: their validity is only known once the batch has run.
pub struct DeviceRecording {
    pub part: DevicePart,
    pub frames: Vec<(DeviceFrame, FitRegion)>,
    /// Wearer-dependent failures the fit raised on the device, read back and
    /// reported as errors once the batch has run.
    pub checks: Vec<DeviceCheck>,
}

/// Reads a fit's status back and turns a raised failure into its error.
pub enum DeviceCheck {
    Device(DeviceValidation),
    /// Validate the finished shell already staged with the rest of the part.
    Mesh(Box<dyn Fn(&BuiltPart) -> anyhow::Result<()> + Send + Sync>),
}

pub type DeviceValidation = Box<
    dyn for<'a> Fn(
            &'a ArmorGpu,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>,
        > + Send
        + Sync,
>;

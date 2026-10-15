//! Interpret torso and shell failures from their existing staged readbacks.
use crate::gpu::anatomy::{STATUS_DEGENERATE, STATUS_EMPTY_SELECTION};
use crate::{GenerateError, TorsoFitFailure};

/// Set when the rig landmarks used by the torso frame coincide.
pub const STATUS_COINCIDENT_LANDMARKS: u32 = 32;
/// Set when the front torso contains no width samples.
pub const STATUS_NO_WIDTH: u32 = 64;
/// Set when the measured front torso width is degenerate.
pub const STATUS_DEGENERATE_WIDTH: u32 = 128;

/// Status word read from the breastplate's device buffer.
pub(super) struct BreastplateStatus(u32);

impl BreastplateStatus {
    pub(super) fn from_device_word(bits: u32) -> Self {
        Self(bits)
    }

    pub(super) fn validate(self) -> Result<(), GenerateError> {
        let failure = if self.0 & STATUS_COINCIDENT_LANDMARKS != 0 {
            Some(TorsoFitFailure::CoincidentLandmarks)
        } else if self.0 & STATUS_NO_WIDTH != 0 {
            Some(TorsoFitFailure::NoWidthSamples)
        } else if self.0 & STATUS_DEGENERATE_WIDTH != 0 {
            Some(TorsoFitFailure::DegenerateWidth)
        } else if self.0 & STATUS_EMPTY_SELECTION != 0 {
            Some(TorsoFitFailure::EmptySelection)
        } else {
            None
        };
        if let Some(failure) = failure {
            return Err(failure.into());
        }
        if self.0 & STATUS_DEGENERATE != 0 {
            return Err(GenerateError::Degenerate);
        }
        if self.0 != 0 {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(())
    }
}

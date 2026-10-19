//! Terminal capture admission rejection, separate from loading/readiness waits.

use super::*;
use adventuresim_tactical_core::animation::AnimationValueError;

/// One terminal authority: a rejected admission cannot return to driving.
pub(super) enum CaptureAdmission {
    Open,
    Rejected(RejectedCaptureAdmission),
    Terminated,
}

pub(super) enum RejectedCaptureAdmission {
    FrameInterval(AnimationValueError),
    Locomotion(AnimationValueError),
}

impl CaptureSequence {
    /// A failed support calculation is terminal admission, not another readiness wait.
    pub(super) fn admit_locomotion_support(
        &mut self,
        skeleton: &SkeletonState,
        speed: NonnegativeSpeed,
    ) -> Option<(f32, f32)> {
        match locomotion_support_weights(skeleton, speed) {
            Ok(weights) => Some(weights),
            Err(error) => {
                self.admission =
                    CaptureAdmission::Rejected(RejectedCaptureAdmission::Locomotion(error));
                None
            }
        }
    }

    /// Driving is permitted only before admission terminates and between captures.
    pub(super) fn can_drive_frame(&self) -> bool {
        self.admission.is_open()
            && !self.applied
            && !self.capture_in_flight
            && self.index < self.plan.len()
    }
}

impl CaptureAdmission {
    pub(super) fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}

impl RejectedCaptureAdmission {
    pub(super) fn message(&self) -> String {
        match self {
            Self::FrameInterval(error) => format!("capture frame interval rejected: {error}\n"),
            Self::Locomotion(error) => {
                format!("capture locomotion observation rejected: {error}\n")
            }
        }
    }
}

/// Admission rejection must terminate even while equipment is still loading.
pub(in crate::animation_viewer) fn reject_capture(
    mut sequence: ResMut<CaptureSequence>,
    mut exit: MessageWriter<AppExit>,
) {
    if let CaptureAdmission::Rejected(rejection) = &sequence.admission {
        fail_capture(&sequence, &rejection.message(), &mut exit);
        sequence.admission = CaptureAdmission::Terminated;
    }
}

pub(super) fn fail_capture(
    sequence: &CaptureSequence,
    message: &str,
    exit: &mut MessageWriter<AppExit>,
) {
    let path = sequence.output.join("failure.txt");
    if let Err(error) = fs::write(&path, message) {
        error!(%error, ?path, "Failed to write capture failure artifact");
    }
    error!(%message, ?path, "Animation capture failed");
    exit.write(AppExit::Error(std::num::NonZeroU8::MIN));
}

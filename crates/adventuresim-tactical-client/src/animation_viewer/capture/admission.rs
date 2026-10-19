//! Terminal capture admission rejection, separate from loading/readiness waits.

use super::*;
use adventuresim_tactical_core::animation::AnimationValueError;

pub(super) enum RejectedCaptureAdmission {
    FrameInterval(AnimationValueError),
    Locomotion(AnimationValueError),
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

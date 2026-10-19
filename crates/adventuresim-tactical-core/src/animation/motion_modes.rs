//! Carried locomotion policies, independent of native Boolean encodings.

use crate::physics::MovementPace;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum DownedMotion {
    #[default]
    Resting,
    Turning,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum GuardedLocomotionMode {
    #[default]
    Footwork,
    Sprint,
}

impl From<MovementPace> for GuardedLocomotionMode {
    fn from(pace: MovementPace) -> Self {
        match pace {
            MovementPace::Sprint => Self::Sprint,
            MovementPace::Walk | MovementPace::Jog => Self::Footwork,
        }
    }
}
impl From<bool> for DownedMotion {
    fn from(turning: bool) -> Self {
        if turning {
            Self::Turning
        } else {
            Self::Resting
        }
    }
}
impl From<DownedMotion> for bool {
    fn from(motion: DownedMotion) -> Self {
        motion == DownedMotion::Turning
    }
}
impl From<bool> for GuardedLocomotionMode {
    fn from(sprinting: bool) -> Self {
        if sprinting {
            Self::Sprint
        } else {
            Self::Footwork
        }
    }
}
impl From<GuardedLocomotionMode> for bool {
    fn from(mode: GuardedLocomotionMode) -> Self {
        mode == GuardedLocomotionMode::Sprint
    }
}

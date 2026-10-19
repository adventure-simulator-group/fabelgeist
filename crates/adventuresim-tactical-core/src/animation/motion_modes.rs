//! Carried locomotion policies, independent of native Boolean encodings.

use crate::physics::MovementPace;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DownedMotion {
    #[default]
    Resting,
    Turning,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
impl Serialize for DownedMotion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(*self == Self::Turning)
    }
}
impl<'de> Deserialize<'de> for DownedMotion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(if bool::deserialize(deserializer)? {
            Self::Turning
        } else {
            Self::Resting
        })
    }
}
impl Serialize for GuardedLocomotionMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(*self == Self::Sprint)
    }
}
impl<'de> Deserialize<'de> for GuardedLocomotionMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(if bool::deserialize(deserializer)? {
            Self::Sprint
        } else {
            Self::Footwork
        })
    }
}

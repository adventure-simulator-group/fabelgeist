//! Client input coordinates, distinct from the animation sample clock.

use bevy::prelude::Reflect;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

/// A complete client input sample on the wrapping 32-bit transport clock.
/// Every integer is valid, including the first observed zero.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, Serialize, Deserialize, From, Display,
)]
#[serde(transparent)]
pub struct PlayerInputTick(u32);

impl PlayerInputTick {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u32::MAX);

    pub fn wrapping_next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Unreliable-channel ordering within the existing half-range window.
    pub fn is_newer_than(self, previous: Self) -> bool {
        let distance = self.0.wrapping_sub(previous.0);
        distance != 0 && distance <= u32::MAX / 2
    }

    /// Original scalar at a transport, diagnostic or native sequence boundary.
    pub fn encoded(self) -> u32 {
        self.0
    }
}

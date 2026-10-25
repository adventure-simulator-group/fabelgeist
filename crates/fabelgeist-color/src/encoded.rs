//! Authored float sRGB channels, without range or finiteness policy.

use crate::TransferFunction;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Encoded sRGB channels; preserves arbitrary float precision and array serde.
/// Application recipes own their numerical acceptance policy.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Srgb([f32; 3]);

/// Encoded sRGB channels and unencoded alpha, with the same scalar policy.
/// RGB alone passes through a transfer function; alpha is copied directly.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Srgba([f32; 4]);

impl Srgb {
    pub const fn from_encoded_channels(channels: [f32; 3]) -> Self {
        Self(channels)
    }

    /// Native encoded channels for serialization, color math or a framework.
    pub const fn encoded_channels(self) -> [f32; 3] {
        self.0
    }

    /// Native linear channels. Unbounded source values may overflow decoding.
    pub fn linear_channels(self) -> [f32; 3] {
        TransferFunction::Srgb.to_linear_rgb(self.0)
    }

    pub const fn opaque(self) -> Srgba {
        let [red, green, blue] = self.0;
        Srgba([red, green, blue, 1.0])
    }
}

impl Srgba {
    pub const WHITE: Self = Self([1.0; 4]);

    /// RGB is encoded sRGB; the fourth native channel is unencoded alpha.
    pub const fn from_encoded_channels(channels: [f32; 4]) -> Self {
        Self(channels)
    }

    /// Native encoded RGB and unencoded alpha at an actual receiving boundary.
    pub const fn encoded_channels(self) -> [f32; 4] {
        self.0
    }

    pub const fn rgb(self) -> Srgb {
        let [red, green, blue, _] = self.0;
        Srgb([red, green, blue])
    }

    /// Replace the native unencoded opacity channel, without an RGB round trip.
    pub const fn with_alpha(self, alpha: f32) -> Self {
        let [red, green, blue, _] = self.0;
        Self([red, green, blue, alpha])
    }

    /// Native linear RGB and unchanged alpha; callers admit output finiteness.
    pub fn linear_channels(self) -> [f32; 4] {
        let [red, green, blue] = self.rgb().linear_channels();
        [red, green, blue, self.0[3]]
    }
}

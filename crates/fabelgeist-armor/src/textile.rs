//! Shared authored textile appearance.
use crate::SurfaceMaterial;
use fabelgeist_color::Srgb;
use fabelgeist_math::units::Ratio;
use serde::{Deserialize, Serialize};

/// Serialized sRGB textile color.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TextileColor(pub [u8; 3]);

impl TextileColor {
    pub fn material(self) -> SurfaceMaterial {
        SurfaceMaterial {
            // The authored byte palette is decoded to float encoded channels.
            base_color: Srgb::from_encoded_channels(
                self.0.map(|channel| f32::from(channel) / 255.0),
            ).opaque(),
            metallic: Ratio::from_ratio(0.0),
            roughness: Ratio::from_ratio(0.92),
        }
    }
}

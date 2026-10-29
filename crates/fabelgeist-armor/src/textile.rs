//! Shared authored textile appearance.
use crate::ArmorComponentMaterial;
use serde::{Deserialize, Serialize};

/// Serialized sRGB textile color.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TextileColor(pub [u8; 3]);

impl TextileColor {
    pub fn material(self) -> ArmorComponentMaterial {
        ArmorComponentMaterial {
            base_color: [
                f32::from(self.0[0]) / 255.0,
                f32::from(self.0[1]) / 255.0,
                f32::from(self.0[2]) / 255.0,
                1.0,
            ],
            metallic: 0.0,
            roughness: 0.92,
        }
    }
}

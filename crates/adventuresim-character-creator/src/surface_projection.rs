//! Terminal scalar material adapters for encoded previews and linear artifacts.

use bevy::prelude::{Color, StandardMaterial};
use fabelgeist_armor::SurfaceMaterial;
use serde::Serialize;

/// Native linear factors for GLB or review JSON, never retained recipe state.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct LinearSurfaceMaterial {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum SurfaceProjectionError {
    #[error("surface color cannot be represented as finite linear native factors")]
    BaseColor,
    #[error("surface metallic factor must be finite")]
    Metallic,
    #[error("surface roughness factor must be finite")]
    Roughness,
}

impl LinearSurfaceMaterial {
    pub const fn base_color(self) -> [f32; 4] {
        self.base_color
    }

    pub const fn metallic(self) -> f32 {
        self.metallic
    }

    pub const fn roughness(self) -> f32 {
        self.roughness
    }
}

impl TryFrom<SurfaceMaterial> for LinearSurfaceMaterial {
    type Error = SurfaceProjectionError;

    fn try_from(material: SurfaceMaterial) -> Result<Self, Self::Error> {
        let base_color = material.base_color.linear_channels();
        if base_color.iter().any(|value| !value.is_finite()) {
            return Err(SurfaceProjectionError::BaseColor);
        }
        let metallic = material.metallic.ratio();
        if !metallic.is_finite() {
            return Err(SurfaceProjectionError::Metallic);
        }
        let roughness = material.roughness.ratio();
        if !roughness.is_finite() {
            return Err(SurfaceProjectionError::Roughness);
        }
        Ok(Self { base_color, metallic, roughness })
    }
}

/// Bevy consumes encoded RGB and unencoded alpha through its native Color API.
/// This projection preserves Bevy's color policy, independently of GLB decoding.
pub fn standard_material(material: SurfaceMaterial) -> StandardMaterial {
    let [red, green, blue, alpha] = material.base_color.encoded_channels();
    StandardMaterial {
        base_color: Color::srgba(red, green, blue, alpha),
        metallic: material.metallic.ratio(),
        perceptual_roughness: material.roughness.ratio(),
        ..Default::default()
    }
}

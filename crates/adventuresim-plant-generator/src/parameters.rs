use serde::{Deserialize, Serialize};

/// Solid sRGB pigment. Lighting is evaluated by the renderer, never baked here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pigment(pub [u8; 3]);

impl Pigment {
    pub(crate) fn linear(self) -> [f32; 4] {
        let rgb = self.0.map(|channel| {
            let s = f32::from(channel) / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        });
        [rgb[0], rgb[1], rgb[2], 1.0]
    }
}

/// Invalid authoring input is rejected before any mesh allocation.
#[derive(Debug, thiserror::Error)]
#[error("{field} must be finite and in {min}..={max}")]
pub struct GenerationError {
    pub field: &'static str,
    pub min: f32,
    pub max: f32,
}

pub(crate) fn bounded(
    field: &'static str,
    value: f32,
    min: f32,
    max: f32,
) -> Result<(), GenerationError> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(GenerationError { field, min, max })
    }
}

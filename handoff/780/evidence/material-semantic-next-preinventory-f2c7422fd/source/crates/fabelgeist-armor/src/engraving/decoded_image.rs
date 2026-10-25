//! Decoded image shape and admission to the native bilinear sampler.

use super::{Engraving, EngravingError, Relief};
use super::sampling;

pub struct ReliefImage {
    extent: ImageExtent,
    pub(super) pixels: ReliefPixels,
}

#[derive(Clone, Copy, Debug)]
struct ImageExtent {
    width: u32,
    height: u32,
}

pub(crate) enum ReliefPixels {
    /// Surface height 0..=1, 1 the untouched surface.
    Height(Vec<f32>),
    /// Rise per run along the image's columns and rows.
    Slopes(Vec<[f32; 2]>),
}

impl ReliefImage {
    pub(super) fn decode(engraving: &Engraving, bytes: &[u8]) -> Result<Self, EngravingError> {
        let image = image::load_from_memory(bytes).map_err(|source| EngravingError::Decode {
            origin: engraving.source.clone(),
            source,
        })?;
        let extent = ImageExtent::new(image.width(), image.height(), engraving.relief)?;
        let pixels = match engraving.relief {
            Relief::Height { .. } => ReliefPixels::Height(image.to_luma32f().into_raw()),
            Relief::Normal { .. } => ReliefPixels::Slopes(
                image
                    .to_rgb32f()
                    .pixels()
                    .map(|pixel| {
                        let [x, y, z] = pixel.0.map(|channel| channel * 2.0 - 1.0);
                        // Steeper than a normal map can encode reliably.
                        let z = z.max(0.05);
                        [-x / z, y / z]
                    })
                    .collect(),
            ),
        };
        Ok(Self { extent, pixels })
    }

    pub(crate) fn dimensions(&self) -> [u32; 2] {
        [self.extent.width, self.extent.height]
    }

    pub(crate) fn pixels(&self) -> &ReliefPixels {
        &self.pixels
    }


}

impl ImageExtent {
    fn new(width: u32, height: u32, relief: Relief) -> Result<Self, EngravingError> {
        sampling::admit_extent(width, height)?;
        let words_per_pixel = match relief {
            Relief::Height { .. } => 1,
            Relief::Normal { .. } => 2,
        };
        let words = u64::from(width) * u64::from(height) * words_per_pixel;
        u32::try_from(words).map_err(|source| EngravingError::ImageWords {
            width,
            height,
            source,
        })?;
        Ok(Self { width, height })
    }
}

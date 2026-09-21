//! Ornament cut into armor metal from an authored relief image.
//!
//! The image tiles with the metal's scratches: one metal tile spans
//! `1 / Metal::TILES_PER_METRE` of surface, and the engraving repeats
//! [`Engraving::tiles`] times across it. A height map gives the cut a depth,
//! so its floor can roughen and the preview can show it in parallax; a normal
//! map only tilts the shading.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Engraving {
    /// The relief image, a PNG resolved against the working directory.
    pub image: PathBuf,
    pub relief: Relief,
    /// Image repeats across one metal tile.
    pub tiles: f32,
    /// Turn of the image on the surface, radians.
    pub rotation: f32,
    /// Roughness added at the floor of a cut; the untouched surface keeps the
    /// metal's finish.
    pub recess_roughness: f32,
}

/// How the relief image encodes the cut.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relief {
    /// A grayscale height map: white is the untouched surface and black the
    /// floor of a cut `depth` metres deep.
    Height { depth: f32 },
    /// A tangent-space normal map in the glTF convention, +Y up; `strength`
    /// scales its slopes, 1 as authored.
    Normal { strength: f32 },
}

impl Relief {
    /// A shallow etch, the usual depth of acid-etched ornament.
    pub const ETCH_DEPTH: f32 = 0.0004;
    /// The deepest cut supported, a heavy chiselled or embossed relief.
    pub const MAX_DEPTH: f32 = 0.005;
    pub const MAX_STRENGTH: f32 = 4.0;
}

impl Engraving {
    pub const MIN_TILES: f32 = 0.25;
    pub const MAX_TILES: f32 = 16.0;

    /// An etched ornament from `image`, repeating once per metal tile.
    pub fn new(image: impl Into<PathBuf>) -> Self {
        Self {
            image: image.into(),
            relief: Relief::Height {
                depth: Relief::ETCH_DEPTH,
            },
            tiles: 1.0,
            rotation: 0.0,
            recess_roughness: 0.35,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let bounded =
            |value: f32, low: f32, high: f32| value.is_finite() && (low..=high).contains(&value);
        let relief = match self.relief {
            Relief::Height { depth } => bounded(depth, 0.0, Relief::MAX_DEPTH),
            Relief::Normal { strength } => bounded(strength, 0.0, Relief::MAX_STRENGTH),
        };
        if self.image.as_os_str().is_empty()
            || !relief
            || !bounded(self.tiles, Self::MIN_TILES, Self::MAX_TILES)
            || !bounded(self.rotation, -std::f32::consts::PI, std::f32::consts::PI)
            || !bounded(self.recess_roughness, 0.0, 1.0)
        {
            return Err("Invalid engraving parameters".into());
        }
        Ok(())
    }

    /// Read and decode the relief image.
    pub fn load(&self) -> Result<ReliefImage, String> {
        let bytes = std::fs::read(&self.image)
            .map_err(|error| format!("Engraving {}: {error}", self.image.display()))?;
        self.decode(&bytes)
    }

    /// Decode PNG `bytes` as this engraving's kind of relief.
    pub fn decode(&self, bytes: &[u8]) -> Result<ReliefImage, String> {
        let image = image::load_from_memory(bytes)
            .map_err(|error| format!("Engraving {}: {error}", self.image.display()))?;
        let (width, height) = (image.width() as usize, image.height() as usize);
        let pixels = match self.relief {
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
        Ok(ReliefImage {
            width,
            height,
            pixels,
        })
    }
}

/// A decoded relief image.
pub struct ReliefImage {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixels: ReliefPixels,
}

pub(crate) enum ReliefPixels {
    /// Surface height 0..=1, 1 the untouched surface.
    Height(Vec<f32>),
    /// Rise per run along the image's columns and rows.
    Slopes(Vec<[f32; 2]>),
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use image::ImageEncoder;

    pub(crate) fn png(width: u32, height: u32, rgb: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
        let rgb = &rgb;
        let pixels = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| rgb(x, y)))
            .collect::<Vec<_>>();
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(&pixels, width, height, image::ExtendedColorType::Rgb8)
            .unwrap();
        bytes
    }

    /// A groove down the middle of the image.
    pub(crate) fn groove_png() -> Vec<u8> {
        png(8, 8, |x, _| {
            if (3..5).contains(&x) {
                [0; 3]
            } else {
                [255; 3]
            }
        })
    }

    #[test]
    fn a_height_map_decodes_white_as_the_untouched_surface() {
        let engraving = Engraving::new("groove.png");
        let image = engraving.decode(&groove_png()).unwrap();
        assert_eq!((image.width, image.height), (8, 8));
        let ReliefPixels::Height(heights) = image.pixels else {
            panic!("a height map decodes to heights");
        };
        assert!((heights[0] - 1.0).abs() < 1e-6 && heights[3].abs() < 1e-6);
    }

    #[test]
    fn a_normal_map_decodes_to_its_authored_slopes() {
        // A normal tilted left encodes a surface rising to the right.
        let bytes = png(4, 4, |_, _| [64, 128, 255]);
        let mut engraving = Engraving::new("tilt.png");
        engraving.relief = Relief::Normal { strength: 1.0 };
        let ReliefPixels::Slopes(slopes) = engraving.decode(&bytes).unwrap().pixels else {
            panic!("a normal map decodes to slopes");
        };
        let expected = -(64.0 / 255.0 * 2.0 - 1.0);
        assert!(
            slopes
                .iter()
                .all(|s| (s[0] - expected).abs() < 0.02 && s[1].abs() < 0.02)
        );
    }

    #[test]
    fn bounds_are_enforced() {
        let mut engraving = Engraving::new("groove.png");
        engraving.tiles = 0.0;
        assert!(engraving.validate().is_err());
        engraving.tiles = 1.0;
        engraving.relief = Relief::Height { depth: 1.0 };
        assert!(engraving.validate().is_err());
        assert!(Engraving::new("").validate().is_err());
        assert!(Engraving::new("missing.png").load().is_err());
        assert!(Engraving::new("garbage.png").decode(b"not a png").is_err());
    }
}

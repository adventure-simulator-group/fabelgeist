//! Ornament cut into armor metal, from an authored relief image or drawn
//! procedurally.
//!
//! The engraving tiles with the metal's scratches: one metal tile spans
//! `1 / Metal::TILES_PER_METRE` of surface, and the engraving repeats
//! [`Engraving::tiles`] times across it. A height map gives the cut a depth,
//! so its floor can roughen and the preview can show it in parallax; a normal
//! map only tilts the shading. A procedural [`Ornament`] is always cut as a
//! height map.
use crate::material::MetalTextureSize;
use crate::ornament::Ornament;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

pub use error::EngravingError;
pub use decoded_image::ReliefImage;
pub(crate) use decoded_image::ReliefPixels;

mod error;
mod decoded_image;
pub(crate) mod sampling;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Engraving {
    pub source: ReliefSource,
    pub relief: Relief,
    /// Image repeats across one metal tile.
    pub tiles: f32,
    /// Turn of the image on the surface, radians.
    pub rotation: f32,
    /// Roughness added at the floor of a cut; the untouched surface keeps the
    /// metal's finish.
    pub recess_roughness: f32,
}

/// A decoded image held with the immutable recipe that defines its relief.
pub struct EngravingImage<'a> {
    recipe: &'a Engraving,
    image: ReliefImage,
}

pub(crate) struct EngravingOrnament<'a> {
    recipe: &'a Engraving,
    ornament: &'a Ornament,
}

pub(crate) enum Cut<'a> {
    Image(EngravingImage<'a>),
    Ornament(EngravingOrnament<'a>),
}

/// What the engraving cuts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReliefSource {
    /// A relief image, a PNG resolved against the working directory.
    Image(PathBuf),
    /// An ornament drawn on the device.
    Ornament(Ornament),
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
    /// Cells a few millimetres long, for a narrow trim's ornament.
    pub const MAX_TILES: f32 = 64.0;

    /// An etched ornament from `image`, repeating once per metal tile.
    pub fn new(image: impl Into<PathBuf>) -> Self {
        Self::from_source(ReliefSource::Image(image.into()))
    }

    /// An etched procedural `ornament`, repeating once per metal tile.
    pub fn ornament(ornament: Ornament) -> Self {
        Self::from_source(ReliefSource::Ornament(ornament))
    }

    fn from_source(source: ReliefSource) -> Self {
        Self {
            source,
            relief: Relief::Height {
                depth: Relief::ETCH_DEPTH,
            },
            tiles: 1.0,
            rotation: 0.0,
            recess_roughness: 0.35,
        }
    }

    pub fn validate(&self) -> Result<(), EngravingError> {
        let bounded =
            |value: f32, low: f32, high: f32| value.is_finite() && (low..=high).contains(&value);
        let relief = match self.relief {
            Relief::Height { depth } => bounded(depth, 0.0, Relief::MAX_DEPTH),
            Relief::Normal { strength } => bounded(strength, 0.0, Relief::MAX_STRENGTH),
        };
        let source = match &self.source {
            ReliefSource::Image(image) => !image.as_os_str().is_empty(),
            ReliefSource::Ornament(ornament) => {
                ornament.validate()?;
                matches!(self.relief, Relief::Height { .. })
            }
        };
        if !source
            || !relief
            || !bounded(self.tiles, Self::MIN_TILES, Self::MAX_TILES)
            || !bounded(self.rotation, -std::f32::consts::PI, std::f32::consts::PI)
            || !bounded(self.recess_roughness, 0.0, 1.0)
        {
            return Err(EngravingError::Parameters);
        }
        Ok(())
    }
}

impl<'a> Cut<'a> {
    pub(crate) fn new(recipe: &'a Engraving) -> Result<Self, EngravingError> {
        recipe.validate()?;
        match &recipe.source {
            ReliefSource::Image(path) => {
                let bytes = std::fs::read(path).map_err(|source| EngravingError::Read {
                    path: path.clone(),
                    source,
                })?;
                Ok(Cut::Image(EngravingImage {
                    recipe,
                    image: ReliefImage::decode(recipe, &bytes)?,
                }))
            }
            ReliefSource::Ornament(ornament) => Ok(Cut::Ornament(EngravingOrnament {
                recipe,
                ornament,
            })),
        }
    }

    pub(crate) fn recipe(&self) -> &Engraving {
        match self {
            Self::Image(image) => image.recipe,
            Self::Ornament(ornament) => ornament.recipe,
        }
    }
}

impl<'a> EngravingImage<'a> {
    /// Decode supplied PNG bytes using this validated image recipe.
    pub fn decode(recipe: &'a Engraving, bytes: &[u8]) -> Result<Self, EngravingError> {
        recipe.validate()?;
        if !matches!(recipe.source, ReliefSource::Image(_)) {
            return Err(EngravingError::ImageSource);
        }
        Ok(Self {
            recipe,
            image: ReliefImage::decode(recipe, bytes)?,
        })
    }

    pub(crate) fn recipe(&self) -> &Engraving {
        self.recipe
    }

    pub(crate) fn image(&self) -> &ReliefImage {
        &self.image
    }

    pub(crate) fn sampling(&self, size: MetalTextureSize) -> Result<[f32; 2], EngravingError> {
        let (sine, cosine) = self.recipe.rotation.sin_cos();
        let coefficients = [sine, cosine];
        self.image.admit_samples(size, self.recipe, coefficients)?;
        Ok(coefficients)
    }
}

impl EngravingOrnament<'_> {
    pub(crate) fn recipe(&self) -> &Engraving {
        self.recipe
    }

    pub(crate) fn ornament(&self) -> &Ornament {
        self.ornament
    }
}

impl fmt::Display for ReliefSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(path) => fmt::Display::fmt(&path.display(), formatter),
            Self::Ornament(_) => formatter.write_str("ornament"),
        }
    }
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
        let image = EngravingImage::decode(&engraving, &groove_png()).unwrap();
        assert_eq!(image.image.dimensions(), [8, 8]);
        let ReliefPixels::Height(heights) = image.image.pixels else {
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
        let ReliefPixels::Slopes(slopes) = EngravingImage::decode(&engraving, &bytes).unwrap().image.pixels else {
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
        assert!(Cut::new(&Engraving::new("missing.png")).is_err());
        let mut ornament = Engraving::ornament(Ornament::default());
        assert!(ornament.validate().is_ok());
        ornament.relief = Relief::Normal { strength: 1.0 };
        assert!(
            ornament.validate().is_err(),
            "an ornament is cut as heights"
        );
        assert!(EngravingImage::decode(&Engraving::new("garbage.png"), b"not a png").is_err());
    }
}

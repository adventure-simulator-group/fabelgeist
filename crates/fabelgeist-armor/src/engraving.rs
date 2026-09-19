//! Ornament cut into armor metal from an authored relief image.
//!
//! The image tiles with the metal's scratches: one metal tile spans
//! `1 / Metal::TILES_PER_METRE` of surface, and the engraving repeats
//! [`Engraving::tiles`] times across it. A height map gives the cut a depth,
//! so its floor can roughen and the preview can show it in parallax; a normal
//! map only tilts the shading.
use crate::material::Metal;
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

    /// The engraving's contribution to each texel of a `size` square metal tile.
    pub(crate) fn rasterize(&self, image: &ReliefImage, size: u32) -> EngravedTile {
        let n = size as usize;
        let (sin, cos) = self.rotation.sin_cos();
        let source = |x: usize, y: usize| {
            let px = (x as f32 + 0.5) / size as f32 - 0.5;
            let py = (y as f32 + 0.5) / size as f32 - 0.5;
            // Turn the tile back into the image, then repeat the image.
            let (qx, qy) = (cos * px + sin * py, -sin * px + cos * py);
            (
                (qx * self.tiles + 0.5) * image.width as f32,
                (qy * self.tiles + 0.5) * image.height as f32,
            )
        };
        match (&image.pixels, self.relief) {
            (ReliefPixels::Height(heights), Relief::Height { depth }) => {
                let recess = (0..n * n)
                    .map(|index| {
                        let (x, y) = source(index % n, index / n);
                        1.0 - image.bilinear(x, y, |i, weight| heights[i] * weight)
                    })
                    .collect::<Vec<_>>();
                let texel = 1.0 / (Metal::TILES_PER_METRE * size as f32);
                let cut = |x: usize, y: usize| -depth * recess[(y % n) * n + x % n];
                let slopes = (0..n * n)
                    .map(|index| {
                        let (x, y) = (index % n, index / n);
                        [
                            (cut(x + 1, y) - cut(x + n - 1, y)) / (2.0 * texel),
                            (cut(x, y + 1) - cut(x, y + n - 1)) / (2.0 * texel),
                        ]
                    })
                    .collect();
                EngravedTile {
                    slopes,
                    recess,
                    recess_roughness: self.recess_roughness,
                    depth_uv: Some(depth * Metal::TILES_PER_METRE),
                }
            }
            (ReliefPixels::Slopes(source_slopes), Relief::Normal { strength }) => {
                let slopes = (0..n * n)
                    .map(|index| {
                        let (x, y) = source(index % n, index / n);
                        let [sx, sy] = image
                            .bilinear(x, y, |i, weight| {
                                Sum(source_slopes[i].map(|slope| slope * weight))
                            })
                            .0;
                        // The image turned with the tile, so its slopes turn too.
                        [
                            (cos * sx - sin * sy) * strength,
                            (sin * sx + cos * sy) * strength,
                        ]
                    })
                    .collect();
                EngravedTile {
                    slopes,
                    recess: vec![0.0; n * n],
                    recess_roughness: self.recess_roughness,
                    depth_uv: None,
                }
            }
            (ReliefPixels::Height(_), Relief::Normal { .. })
            | (ReliefPixels::Slopes(_), Relief::Height { .. }) => {
                unreachable!("a relief image is decoded as the engraving's own kind")
            }
        }
    }
}

/// A decoded relief image.
pub struct ReliefImage {
    width: usize,
    height: usize,
    pixels: ReliefPixels,
}

enum ReliefPixels {
    /// Surface height 0..=1, 1 the untouched surface.
    Height(Vec<f32>),
    /// Rise per run along the image's columns and rows.
    Slopes(Vec<[f32; 2]>),
}

impl ReliefImage {
    /// Sample the repeating image at pixel coordinates, weighting the four
    /// neighbours through `term`, which returns a summable value.
    fn bilinear<T: std::ops::Add<Output = T>>(
        &self,
        x: f32,
        y: f32,
        term: impl Fn(usize, f32) -> T,
    ) -> T {
        let (x, y) = (x - 0.5, y - 0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let column = |dx: i64| (x0 as i64 + dx).rem_euclid(self.width as i64) as usize;
        let row = |dy: i64| (y0 as i64 + dy).rem_euclid(self.height as i64) as usize;
        let at = |dx, dy, weight| term(row(dy) * self.width + column(dx), weight);
        at(0, 0, (1.0 - fx) * (1.0 - fy))
            + at(1, 0, fx * (1.0 - fy))
            + at(0, 1, (1.0 - fx) * fy)
            + at(1, 1, fx * fy)
    }
}

/// A slope pair that can be summed for bilinear weighting.
struct Sum([f32; 2]);

impl std::ops::Add for Sum {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self([self.0[0] + other.0[0], self.0[1] + other.0[1]])
    }
}

/// The engraving rasterized onto a metal tile.
pub(crate) struct EngravedTile {
    /// Rise per run along the texture's columns and rows.
    pub slopes: Vec<[f32; 2]>,
    /// Fraction of the cut's depth below the surface, 0 on the untouched surface.
    pub recess: Vec<f32>,
    /// Roughness added at a full recess.
    pub recess_roughness: f32,
    /// Depth of a full recess in texture units, for parallax; a normal map has none.
    pub depth_uv: Option<f32>,
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
    fn a_height_map_cuts_below_the_surface_and_slopes_into_the_groove() {
        let engraving = Engraving::new("groove.png");
        let tile = engraving.rasterize(&engraving.decode(&groove_png()).unwrap(), 32);
        assert!(tile.depth_uv.is_some());
        let row = &tile.recess[..32];
        assert!(row[0] < 0.01 && row[16] > 0.99, "{row:?}");
        // The left wall descends the whole cut and the right wall climbs out
        // of it, so each wall's slopes sum to the depth in texels.
        let slopes = &tile.slopes[..32];
        let left = slopes[8..16].iter().map(|s| s[0]).sum::<f32>();
        let right = slopes[16..24].iter().map(|s| s[0]).sum::<f32>();
        let wall = Relief::ETCH_DEPTH * Metal::TILES_PER_METRE * 32.0;
        assert!((left + wall).abs() < wall * 0.02, "{left} {wall}");
        assert!((right - wall).abs() < wall * 0.02, "{right} {wall}");
        assert!(slopes.iter().all(|s| s[1].abs() < 1e-4));
    }

    #[test]
    fn a_normal_map_gives_its_authored_slopes_and_no_depth() {
        // A normal tilted left encodes a surface rising to the right.
        let bytes = png(4, 4, |_, _| [64, 128, 255]);
        let mut engraving = Engraving::new("tilt.png");
        engraving.relief = Relief::Normal { strength: 1.0 };
        let tile = engraving.rasterize(&engraving.decode(&bytes).unwrap(), 8);
        assert!(tile.depth_uv.is_none());
        assert!(tile.recess.iter().all(|r| *r == 0.0));
        let expected = -(64.0 / 255.0 * 2.0 - 1.0);
        assert!(
            tile.slopes
                .iter()
                .all(|s| (s[0] - expected).abs() < 0.02 && s[1].abs() < 0.02)
        );
        engraving.rotation = std::f32::consts::FRAC_PI_2;
        let turned = engraving.rasterize(&engraving.decode(&bytes).unwrap(), 8);
        assert!(
            turned
                .slopes
                .iter()
                .all(|s| s[0].abs() < 0.02 && (s[1] - expected).abs() < 0.02)
        );
    }

    #[test]
    fn tiling_repeats_the_image_and_bounds_are_enforced() {
        let mut engraving = Engraving::new("groove.png");
        engraving.tiles = 2.0;
        let image = engraving.decode(&groove_png()).unwrap();
        let tile = engraving.rasterize(&image, 32);
        let grooves = tile.recess[..32]
            .windows(2)
            .filter(|w| w[0] < 0.5 && w[1] >= 0.5)
            .count();
        assert_eq!(grooves, 2);
        engraving.tiles = 0.0;
        assert!(engraving.validate().is_err());
        engraving.tiles = 1.0;
        engraving.relief = Relief::Height { depth: 1.0 };
        assert!(engraving.validate().is_err());
        assert!(Engraving::new("").validate().is_err());
        assert!(Engraving::new("missing.png").load().is_err());
    }
}

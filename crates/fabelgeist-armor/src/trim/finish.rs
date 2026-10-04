//! A band finished apart from the rest of the plate along its edges.
//!
//! The band has its own [`Metal`], so an edge can be gilded, blued or left
//! bright on a darker plate, and the metal's engraving is the ornament that
//! runs along it. The ornament repeats once per engraving cell along the edge,
//! starting at the edge itself.
use crate::engraving::Engraving;
use crate::material::{Metal, MetalError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trim {
    /// How far the band reaches in from the edge, metres.
    pub width: f32,
    pub metal: Metal,
}

#[derive(Debug, thiserror::Error)]
pub enum TrimFinishError {
    #[error("Invalid trim width")]
    Width,
    #[error("{0}")]
    Metal(#[from] MetalError),
}

impl Default for Trim {
    /// A gilt band two centimetres wide.
    fn default() -> Self {
        Self {
            width: 0.02,
            metal: Metal {
                color: [1.0, 0.77, 0.34],
                roughness: 0.25,
                scratch_density: 0,
                ..Metal::default()
            },
        }
    }
}

impl Trim {
    pub const MIN_WIDTH: f32 = 0.003;
    pub const MAX_WIDTH: f32 = 0.06;

    /// The length along the edge its ornament repeats over, metres: one
    /// engraving cell, or one metal tile without an engraving.
    pub fn period(&self) -> f32 {
        1.0 / (Metal::TILES_PER_METRE * self.tiles())
    }

    /// Where an engraving cell begins, in metal tiles: the engraving repeats
    /// about the tile's centre, so the band's coordinates start here to put
    /// the top of the image on the edge.
    pub fn cell_origin(&self) -> f32 {
        0.5 - 0.5 / self.tiles()
    }

    /// Engraving repeats per metal tile that make one cell as long as the
    /// band is wide, within the engraving's range.
    pub fn cell_tiles(&self) -> f32 {
        (1.0 / (Metal::TILES_PER_METRE * self.width))
            .clamp(Engraving::MIN_TILES, Engraving::MAX_TILES)
    }

    /// The same finish on a band `width` wide, its ornament shrunk in
    /// proportion so that it still spans the band.
    pub fn narrowed(&self, width: f32) -> Self {
        let mut narrowed = self.clone();
        if width < self.width {
            let scale = self.width / width;
            if let Some(engraving) = &mut narrowed.metal.engraving {
                engraving.tiles =
                    (engraving.tiles * scale).clamp(Engraving::MIN_TILES, Engraving::MAX_TILES);
            }
            narrowed.width = width;
        }
        narrowed
    }

    fn tiles(&self) -> f32 {
        self.metal.engraving.as_ref().map_or(1.0, |e| e.tiles)
    }

    pub fn validate(&self) -> Result<(), TrimFinishError> {
        if !self.width.is_finite() || !(Self::MIN_WIDTH..=Self::MAX_WIDTH).contains(&self.width) {
            return Err(TrimFinishError::Width);
        }
        Ok(self.metal.validate()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_narrowed_band_shrinks_its_ornament_with_it() {
        let mut trim = Trim::default();
        trim.metal.engraving = Some(Engraving {
            tiles: 4.0,
            ..Engraving::new("border.png")
        });
        let narrow = trim.narrowed(trim.width * 0.25);
        assert_eq!(narrow.width, trim.width * 0.25);
        assert_eq!(narrow.metal.engraving.unwrap().tiles, 16.0);
        assert_eq!(trim.narrowed(trim.width * 2.0), trim);
    }

    #[test]
    fn the_ornament_repeats_once_per_engraving_cell() {
        let mut trim = Trim::default();
        assert_eq!(trim.period(), 1.0 / Metal::TILES_PER_METRE);
        trim.metal.engraving = Some(Engraving {
            tiles: 5.0,
            ..Engraving::new("border.png")
        });
        assert!((trim.period() - 0.05).abs() < 1e-7);
    }

    #[test]
    fn a_cell_begins_at_the_origin() {
        for tiles in [1.0, 5.0, 12.5] {
            let trim = Trim {
                metal: Metal {
                    engraving: Some(Engraving {
                        tiles,
                        ..Engraving::new("border.png")
                    }),
                    ..Metal::default()
                },
                ..Trim::default()
            };
            // The engraving samples the image at `(u - 0.5) * tiles + 0.5`.
            let image = (trim.cell_origin() - 0.5) * tiles + 0.5;
            assert!((image - image.round()).abs() < 1e-5, "{tiles}: {image}");
        }
    }

    #[test]
    fn widths_outside_the_supported_band_are_rejected() {
        let trim = Trim::default();
        assert!(trim.validate().is_ok());
        for width in [0.0, Trim::MAX_WIDTH * 2.0, f32::NAN] {
            assert!(
                Trim {
                    width,
                    ..trim.clone()
                }
                .validate()
                .is_err()
            );
        }
    }
}

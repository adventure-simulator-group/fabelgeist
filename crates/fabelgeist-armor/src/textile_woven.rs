//! Figured weaves stored as one repeat each.
//!
//! A tile is a two-channel picture of one repeat of a woven pattern: the grey
//! channel is the share of main figure, the alpha channel the share of a
//! second, accent ink, and where both are zero lies ground cloth. Tiles are
//! made offline by
//! `scripts/textile_pattern_tile.py` from pictures of historic textiles and
//! are listed with their sources in `patterns/README.md`.
use serde::{Deserialize, Serialize};

use crate::GenerateError;

/// Most samples averaged along one axis of a texel.
const MAX_TAPS: f32 = 4.;

/// Which ink of a tile a layer draws.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WovenTone {
    /// The pattern's principal figure.
    Figure,
    /// The second, quieter ink.
    Accent,
}

/// A stored repeat of a figured weave.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WovenPattern {
    /// Pomegranates in a pointed-oval lattice of ribboned stems, after an
    /// Italian silk of the fifteenth century.
    ItalianPomegranate,
}

/// One decoded repeat. Sampling wraps on both axes.
pub(crate) struct WovenTile {
    width: u32,
    height: u32,
    /// Grey and alpha of each sample, interleaved.
    inks: Vec<u8>,
}

impl WovenPattern {
    pub const ALL: [Self; 1] = [Self::ItalianPomegranate];

    pub fn label(self) -> &'static str {
        match self {
            Self::ItalianPomegranate => "Italian pomegranate",
        }
    }

    /// Samples across and along the stored repeat.
    pub(crate) fn samples(self) -> [u32; 2] {
        match self {
            Self::ItalianPomegranate => [512, 960],
        }
    }

    fn encoded(self) -> &'static [u8] {
        match self {
            Self::ItalianPomegranate => include_bytes!("../patterns/italian-pomegranate.png"),
        }
    }
}

impl WovenTile {
    pub(crate) fn new(pattern: WovenPattern) -> Result<Self, GenerateError> {
        let tile = image::load_from_memory_with_format(pattern.encoded(), image::ImageFormat::Png)
            .map_err(|_| GenerateError::InvalidSurface)?
            .into_luma_alpha8();
        if [tile.width(), tile.height()] != pattern.samples() {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(Self {
            width: tile.width(),
            height: tile.height(),
            inks: tile.into_raw(),
        })
    }

    /// Share of `tone` ink in a texel at `point` that is `footprint` wide and
    /// high, both in repeats across and along. The tile's first row is the
    /// end of the repeat nearest the proximal edge. A texel wider than a
    /// tile sample is the mean of the samples it covers, so a pattern drawn
    /// smaller than it is stored does not break into specks.
    pub(crate) fn coverage(&self, tone: WovenTone, point: [f32; 2], footprint: [f32; 2]) -> f32 {
        let taps = [
            (footprint[0] * self.width as f32)
                .ceil()
                .clamp(1., MAX_TAPS),
            (footprint[1] * self.height as f32)
                .ceil()
                .clamp(1., MAX_TAPS),
        ];
        let mut total = 0.;
        for row in 0..taps[1] as u32 {
            for column in 0..taps[0] as u32 {
                total += self.sample(
                    tone,
                    [
                        point[0] + ((column as f32 + 0.5) / taps[0] - 0.5) * footprint[0],
                        point[1] + ((row as f32 + 0.5) / taps[1] - 0.5) * footprint[1],
                    ],
                );
            }
        }
        total / (taps[0] * taps[1])
    }

    /// Share of ink at one point, blended between the nearest samples.
    fn sample(&self, tone: WovenTone, point: [f32; 2]) -> f32 {
        let channel = match tone {
            WovenTone::Figure => 0,
            WovenTone::Accent => 1,
        };
        let across = point[0].rem_euclid(1.) * self.width as f32 - 0.5;
        let along = point[1].rem_euclid(1.) * self.height as f32 - 0.5;
        let column = across.floor();
        let row = along.floor();
        let blend = [across - column, along - row];
        let sample = |column_step: i64, row_step: i64| {
            let column = (column as i64 + column_step).rem_euclid(i64::from(self.width));
            let row = (row as i64 + row_step).rem_euclid(i64::from(self.height));
            f32::from(self.inks[2 * (row * i64::from(self.width) + column) as usize + channel])
                / 255.
        };
        let near = sample(0, 0) * (1. - blend[0]) + sample(1, 0) * blend[0];
        let far = sample(0, 1) * (1. - blend[0]) + sample(1, 1) * blend[0];
        near * (1. - blend[1]) + far * blend[1]
    }
}

//! Bounded empirical mixing of measured prepared paints, not arbitrary pigments.
mod calibration;
mod search;
use super::{PaintAppearance, PaintColors};
use crate::{Error, artwork::PaintTone, document::Ratio};
pub use calibration::{CALIBRATION_JSON, calibration_id, provenance};
pub use search::{
    BatchVolume, ColorTolerance, Match, MatchStatus, Quote, SearchRequest, SetupCost, StockPrice,
    Workshop,
};
use serde::{Deserialize, Serialize};

pub const WHITE_SCALE: u16 = 1000;
const GUM_PAINT_ROUGHNESS: Ratio = Ratio(0.72);

/// Stocks in the same gum-Arabic/parchment measurement series.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaintStock {
    LeadWhite,
    LeadTinYellow,
    YellowOchre,
    Azurite,
    Cinnabar,
    GrapeSeedBlack,
}
impl PaintStock {
    pub const ALL: [Self; 6] = [
        Self::LeadWhite,
        Self::LeadTinYellow,
        Self::YellowOchre,
        Self::Azurite,
        Self::Cinnabar,
        Self::GrapeSeedBlack,
    ];
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::LeadWhite => "Lead white",
            Self::LeadTinYellow => "Lead-tin yellow",
            Self::YellowOchre => "Yellow ochre (pure only)",
            Self::Azurite => "Azurite, extra fine",
            Self::Cinnabar => "Cinnabar",
            Self::GrapeSeedBlack => "Grape-seed black",
        }
    }
    pub fn supports_tints(self) -> bool {
        !matches!(self, Self::LeadWhite | Self::YellowOchre)
    }
}

/// Volume share of prepared white paint, in thousandths, within a measured family.
/// Private fields and checked deserialization prevent unsupported mixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "MixInput")]
pub struct StockMix {
    stock: PaintStock,
    white_permille: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MixInput {
    stock: PaintStock,
    white_permille: u16,
}
impl TryFrom<MixInput> for StockMix {
    type Error = Error;
    fn try_from(input: MixInput) -> Result<Self, Error> {
        Self::new(input.stock, input.white_permille)
    }
}
impl StockMix {
    pub fn new(stock: PaintStock, white_permille: u16) -> Result<Self, Error> {
        if white_permille > WHITE_SCALE || (!stock.supports_tints() && white_permille != 0) {
            return Err(Error::Invalid("unsupported prepared-stock mixture".into()));
        }
        Ok(Self {
            stock,
            white_permille,
        })
    }
    pub const fn pure(stock: PaintStock) -> Self {
        Self {
            stock,
            white_permille: 0,
        }
    }
    pub fn stock(self) -> PaintStock {
        self.stock
    }
    pub fn white_permille(self) -> u16 {
        self.white_permille
    }
    pub fn color(self) -> [u8; 3] {
        calibration::color(self)
    }
    /// Each component's share of the prepared batch, summing to WHITE_SCALE.
    pub fn shares(self) -> [u16; 6] {
        let mut shares = [0; 6];
        shares[self.stock.index()] = WHITE_SCALE - self.white_permille;
        shares[PaintStock::LeadWhite.index()] += self.white_permille;
        shares
    }
    /// Piecewise interpolation through measured spectra; no extrapolation.
    pub fn reflectance(self) -> Vec<f64> {
        calibration::reflectance(self)
    }
    pub fn measured_knot(self) -> bool {
        self.white_permille.is_multiple_of(WHITE_SCALE / 2)
    }
}

/// Independent physical recipes for the drawing's three painted tones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixedPaint {
    pub base: StockMix,
    pub shadow: StockMix,
    pub highlight: StockMix,
}
impl MixedPaint {
    pub const fn uniform(mix: StockMix) -> Self {
        Self {
            base: mix,
            shadow: mix,
            highlight: mix,
        }
    }
    pub fn appearance(self) -> PaintAppearance {
        PaintAppearance {
            colors: PaintColors {
                base: self.base.color(),
                shadow: self.shadow.color(),
                highlight: self.highlight.color(),
            },
            roughness: GUM_PAINT_ROUGHNESS,
        }
    }
    pub fn tone_mut(&mut self, tone: PaintTone) -> &mut StockMix {
        match tone {
            PaintTone::Base => &mut self.base,
            PaintTone::Shadow => &mut self.shadow,
            PaintTone::Highlight => &mut self.highlight,
        }
    }
}

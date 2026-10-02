//! Exhaustive least-cost search over the supported, quantized recipe set.
use super::{PaintStock, StockMix, WHITE_SCALE, calibration};
use crate::Error;
use serde::{Deserialize, Serialize};

const MICROLITERS_PER_ML: f64 = 1000.0;
const MICRO_UNITS_PER_UNIT: u64 = 1_000_000;
const MAX_BATCH_MICROLITERS: u32 = 1_000_000;
const MAX_PRICE_PER_ML: u32 = 1_000_000;
const ERROR_PRECISION: f64 = 1_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchVolume(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockPrice(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupCost(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColorTolerance(pub f64);

/// Illustrative accounting units, not a claim about historical currency/prices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workshop {
    pub prices_per_ml: [StockPrice; 6],
    pub available: [bool; 6],
    pub batch_microliters: BatchVolume,
    pub setup_cost: SetupCost,
    pub budget_units: Option<u32>,
}
impl Default for Workshop {
    fn default() -> Self {
        Self {
            prices_per_ml: [StockPrice(1); 6],
            available: [true; 6],
            batch_microliters: BatchVolume(10_000),
            setup_cost: SetupCost(0),
            budget_units: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    pub stock_milliliters: [f64; 6],
    /// Exact integer cost; one million micro-units is one accounting unit.
    pub cost_micro_units: u64,
}
impl Quote {
    pub fn cost_units(&self) -> f64 {
        self.cost_micro_units as f64 / MICRO_UNITS_PER_UNIT as f64
    }
}
impl Workshop {
    pub fn validate(&self) -> Result<(), Error> {
        if !(1..=MAX_BATCH_MICROLITERS).contains(&self.batch_microliters.0)
            || self.prices_per_ml.iter().any(|p| p.0 > MAX_PRICE_PER_ML)
        {
            return Err(Error::Invalid(
                "batch must be 1–1000000 µL; stock prices 0–1000000 units/mL".into(),
            ));
        }
        Ok(())
    }
    /// None means missing stock or over budget. Zero-quantity stocks do not block.
    pub fn quote(&self, mix: StockMix) -> Result<Option<Quote>, Error> {
        self.validate()?;
        Ok(self.quote_valid(mix))
    }
    fn quote_valid(&self, mix: StockMix) -> Option<Quote> {
        let shares = mix.shares();
        if shares
            .iter()
            .zip(self.available)
            .any(|(share, available)| *share > 0 && !available)
        {
            return None;
        }
        let cost_micro_units = shares
            .iter()
            .zip(self.prices_per_ml)
            .map(|(share, price)| {
                u64::from(*share) * u64::from(price.0) * u64::from(self.batch_microliters.0)
            })
            .sum::<u64>()
            + u64::from(self.setup_cost.0) * MICRO_UNITS_PER_UNIT;
        if self
            .budget_units
            .is_some_and(|budget| cost_micro_units > u64::from(budget) * MICRO_UNITS_PER_UNIT)
        {
            return None;
        }
        Some(Quote {
            stock_milliliters: shares.map(|s| {
                f64::from(s) / f64::from(WHITE_SCALE) * f64::from(self.batch_microliters.0)
                    / MICROLITERS_PER_ML
            }),
            cost_micro_units,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    /// A request only. The result always saves a supported StockMix.
    pub target_srgb: [u8; 3],
    pub tolerance: ColorTolerance,
    pub workshop: Workshop,
}
impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            target_srgb: [63, 99, 155],
            tolerance: ColorTolerance(3.0),
            workshop: Workshop::default(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchStatus {
    WithinTolerance,
    NearestOutsideTolerance,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Match {
    pub mix: StockMix,
    pub preview_srgb: [u8; 3],
    pub delta_e76: f64,
    pub status: MatchStatus,
    pub quote: Quote,
}
impl SearchRequest {
    /// Cheapest in tolerance; otherwise explicitly the nearest feasible recipe.
    /// Ties: cost/error, then stable stock and white-share order. Search covers
    /// every 0.1% share; it makes no continuous/global physical-optimum claim.
    pub fn solve(&self) -> Result<Option<Match>, Error> {
        self.workshop.validate()?;
        if !self.tolerance.0.is_finite() || !(0.0..=200.0).contains(&self.tolerance.0) {
            return Err(Error::Invalid(
                "color tolerance must be finite, 0–200 ΔE76".into(),
            ));
        }
        let target = calibration::lab(self.target_srgb);
        let tolerance = (self.tolerance.0 * ERROR_PRECISION).round() as u64;
        let mut best: Option<((u8, u64, u64), Match)> = None;
        for stock in PaintStock::ALL {
            // Pure white is represented once, under LeadWhite.
            let end = if stock.supports_tints() {
                WHITE_SCALE - 1
            } else {
                0
            };
            for white in 0..=end {
                let mix = StockMix::new(stock, white)?;
                let Some(quote) = self.workshop.quote_valid(mix) else {
                    continue;
                };
                let preview_srgb = mix.color();
                let lab = calibration::lab(preview_srgb);
                let error = (target
                    .iter()
                    .zip(lab)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
                    * ERROR_PRECISION)
                    .round() as u64;
                let within = error <= tolerance;
                let key = if within {
                    (0, quote.cost_micro_units, error)
                } else {
                    (1, error, quote.cost_micro_units)
                };
                if best.as_ref().is_none_or(|(old, _)| key < *old) {
                    best = Some((
                        key,
                        Match {
                            mix,
                            preview_srgb,
                            delta_e76: error as f64 / ERROR_PRECISION,
                            status: if within {
                                MatchStatus::WithinTolerance
                            } else {
                                MatchStatus::NearestOutsideTolerance
                            },
                            quote,
                        },
                    ));
                }
            }
        }
        Ok(best.map(|(_, result)| result))
    }
}

//! Shared relief dimensions; each armor construction supplies its own surface chart.
use crate::{DesignError, Millimeters, Permille};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// Raised, rounded flutes separated by smooth lands on an authored plate chart.
/// Width is a fraction of flute pitch, independent of count. Dimensions are
/// in the authored carrier chart; physical widths scale with the wearer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlateFluting {
    pub count: FluteCount,
    pub width: Permille,
    pub depth: Millimeters,
    /// Fraction of chart width occupied by the pattern at its top.
    pub spread: Permille,
    /// Lower pattern width relative to its top; 1000 adds no fan in the carrier chart.
    pub lower_spread: Permille,
    pub start: Permille,
    pub end: Permille,
    /// Fade length at each end, as a fraction of plate height.
    pub fade: Permille,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FluteCount(pub u16);

impl Default for PlateFluting {
    fn default() -> Self {
        Self {
            count: FluteCount(16),
            width: Permille(850),
            depth: Millimeters(2),
            spread: Permille(850),
            lower_spread: Permille(900),
            start: Permille(150),
            end: Permille(800),
            fade: Permille(100),
        }
    }
}

impl PlateFluting {
    pub const COUNT_RANGE: RangeInclusive<u16> = 2..=24;
    pub const WIDTH_RANGE: RangeInclusive<u16> = 350..=850;
    pub const DEPTH_RANGE: RangeInclusive<u16> = 1..=4;
    pub const SPREAD_RANGE: RangeInclusive<u16> = 400..=850;
    pub const LOWER_SPREAD_RANGE: RangeInclusive<u16> = 500..=1000;
    pub const FADE_RANGE: RangeInclusive<u16> = 100..=250;
    pub const MIN_START: u16 = 50;
    pub const MAX_END: u16 = 950;

    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !Self::COUNT_RANGE.contains(&self.count.0)
            || !Self::WIDTH_RANGE.contains(&self.width.0)
            || !Self::DEPTH_RANGE.contains(&self.depth.0)
            || !Self::SPREAD_RANGE.contains(&self.spread.0)
            || !Self::LOWER_SPREAD_RANGE.contains(&self.lower_spread.0)
            || self.start.0 < Self::MIN_START
            || self.end.0 > Self::MAX_END
            || self.end.0 <= self.start.0
            || !Self::FADE_RANGE.contains(&self.fade.0)
            || self.end.0 - self.start.0 < self.fade.0 * 2
        {
            return Err(DesignError::PlateFluting);
        }
        Ok(())
    }

    /// Across-chart samples include every flute crest, slope and land edge.
    pub fn columns(&self, base_segments: usize) -> Vec<f32> {
        const PROFILE_SEGMENTS: usize = 8;
        const MERGE_TOLERANCE: f32 = 1e-5;
        let pitch = self.spread.unit() / f32::from(self.count.0);
        let left = (1.0 - self.spread.unit()) * 0.5;
        let right = 1.0 - left;
        let minimum_land = pitch * (1.0 - self.width.unit()) * 0.5;
        // The feature grid owns the fluted field. Interleaving an unrelated
        // regular grid creates tiny strips whose offset walls can reverse.
        let mut samples = (0..=base_segments)
            .map(|i| i as f32 / base_segments as f32)
            .filter(|u| *u < left - minimum_land || *u > right + minimum_land)
            .collect::<Vec<_>>();
        samples.extend([left, right]);
        for flute in 0..self.count.0 {
            let center = (1.0 - self.spread.unit()) * 0.5 + (f32::from(flute) + 0.5) * pitch;
            for sample in 0..=PROFILE_SEGMENTS {
                samples.push(
                    center
                        + (sample as f32 / PROFILE_SEGMENTS as f32 - 0.5)
                            * pitch
                            * self.width.unit(),
                );
            }
        }
        samples.sort_by(f32::total_cmp);
        samples.dedup_by(|a, b| (*a - *b).abs() < MERGE_TOLERANCE);
        samples
    }

    /// Fan the interior chart while retaining its attachment boundaries.
    pub fn fan_coordinate(&self, u: f32, v: f32) -> f32 {
        let lateral = 2.0 * u - 1.0;
        let span = self.spread.unit();
        let fan = self.lower_spread.unit() + (1.0 - self.lower_spread.unit()) * smooth(v);
        let absolute = lateral.abs();
        let mapped = if absolute <= span {
            absolute * fan
        } else {
            span * fan + (absolute - span) * (1.0 - span * fan) / (1.0 - span)
        };
        (1.0 + lateral.signum() * mapped) * 0.5
    }

    pub fn relief(&self, u: f32, v: f32) -> f32 {
        let pitch = self.spread.unit() / f32::from(self.count.0);
        let start = (1.0 - self.spread.unit()) * 0.5;
        let slot = ((u - start) / pitch).floor();
        if slot < 0.0 || slot >= f32::from(self.count.0) {
            return 0.0;
        }
        let center = start + (slot + 0.5) * pitch;
        let distance = (u - center).abs() / (pitch * self.width.unit() * 0.5);
        if distance >= 1.0 {
            return 0.0;
        }
        let fade = smooth((v - self.start.unit()) / self.fade.unit())
            * smooth((self.end.unit() - v) / self.fade.unit());
        self.depth.metres() * fade * (1.0 + (std::f32::consts::PI * distance).cos()) * 0.5
    }

    /// Recover the authored field coordinate from a fitted fan chart.
    pub fn unfan_coordinate(&self, mapped: f32, v: f32) -> f32 {
        let lateral = 2.0 * mapped - 1.0;
        let span = self.spread.unit();
        let fan = self.lower_spread.unit() + (1.0 - self.lower_spread.unit()) * smooth(v);
        let absolute = lateral.abs();
        let u = if absolute <= span * fan {
            absolute / fan
        } else {
            span + (absolute - span * fan) * (1.0 - span) / (1.0 - span * fan)
        };
        (1.0 + lateral.signum() * u) * 0.5
    }
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

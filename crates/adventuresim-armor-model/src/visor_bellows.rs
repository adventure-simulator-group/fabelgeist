//! Horizontal formed visor folds, independent of longitudinal decorative flutes.
use crate::{DesignError, Millimeters, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisorBellows {
    pub count: u8,
    pub depth: Millimeters,
    /// Folded interval measured downward in the pierced visor's design chart.
    pub start: Permille,
    pub end: Permille,
    pub sharpness: Permille,
    /// Upward sweep of the folded face toward either cheek hinge.
    pub cheek_rise: Millimeters,
}

impl Default for VisorBellows {
    fn default() -> Self {
        Self {
            count: 3,
            depth: Millimeters(6),
            start: Permille(420),
            end: Permille(900),
            sharpness: Permille(800),
            cheek_rise: Millimeters(20),
        }
    }
}

impl VisorBellows {
    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !(1..=5).contains(&self.count)
            || !(2..=14).contains(&self.depth.0)
            || !(350..=550).contains(&self.start.0)
            || !(800..=950).contains(&self.end.0)
            || self.sharpness.0 > 1000
            || self.cheek_rise.0 > 35
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }

    pub(crate) fn relief(self, down: f32) -> f32 {
        if down <= self.start.unit() || down >= self.end.unit() {
            return 0.0;
        }
        let cycle = (down - self.start.unit()) / (self.end.unit() - self.start.unit())
            * f32::from(self.count);
        let phase = cycle.fract();
        let triangle = 1.0 - (2.0 * phase - 1.0).abs();
        let round = (1.0 - (std::f32::consts::TAU * phase).cos()) * 0.5;
        self.depth.metres() * (round + (triangle - round) * self.sharpness.unit())
    }

    pub(crate) fn rows(self) -> impl Iterator<Item = f64> {
        const SAMPLES_PER_FOLD: u16 = 8;
        let steps = u16::from(self.count) * SAMPLES_PER_FOLD;
        (0..=steps).map(move |i| {
            f64::from(self.start.0)
                + f64::from(self.end.0 - self.start.0) * f64::from(i) / f64::from(steps)
        })
    }
}

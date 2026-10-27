//! A complete circular field of spokes with a smooth inner and outer border.
use crate::{DesignError, FluteCount, Millimeters, Permille, PlateFluting};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadialFluting {
    pub count: FluteCount,
    pub width: Permille,
    pub depth: Millimeters,
    pub start: Permille,
    pub end: Permille,
    pub fade: Permille,
}

impl Default for RadialFluting {
    fn default() -> Self {
        Self {
            count: FluteCount(24),
            width: Permille(700),
            depth: Millimeters(2),
            start: Permille(250),
            end: Permille(950),
            fade: Permille(100),
        }
    }
}

impl RadialFluting {
    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !(4..=48).contains(&self.count.0)
            || !PlateFluting::WIDTH_RANGE.contains(&self.width.0)
            || !PlateFluting::DEPTH_RANGE.contains(&self.depth.0)
            || !(200..=700).contains(&self.start.0)
            || !(800..=1000).contains(&self.end.0)
            || !(50..=250).contains(&self.fade.0)
            || self.end.0 - self.start.0 < self.fade.0 * 2
        {
            return Err(DesignError::PlateFluting);
        }
        Ok(())
    }

    /// Columns around a disc: enough samples to shape every spoke.
    pub(crate) fn columns(self) -> usize {
        const SAMPLES_PER_SPOKE: usize = 12;
        usize::from(self.count.0) * SAMPLES_PER_SPOKE
    }
}

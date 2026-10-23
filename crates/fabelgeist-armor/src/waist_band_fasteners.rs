//! Parallel band closures and an optional return hook into a bodice ring.
use crate::{ClothClosures, ClothFastener, DesignError, GenerateError, Millimeters};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaistBandFasteners {
    /// Ordered distances down the front band, excluding its finished edges.
    pub closures: ClothClosures,
    /// Eye origin measured from the front band's closing edge.
    pub inset: Millimeters,
    /// A separately oriented hook catching the final metal bodice ring.
    pub return_hook: Option<ClothFastener>,
}
impl WaistBandFasteners {
    pub const INSET_RANGE: std::ops::RangeInclusive<u16> = 1..=10;
    pub fn validate(&self) -> Result<(), GenerateError> {
        self.closures.validate()?;
        if !Self::INSET_RANGE.contains(&self.inset.0)
            || !matches!(self.closures.fastener, ClothFastener::HookEye { .. })
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if let Some(hook) = &self.return_hook {
            hook.validate()?;
            if !matches!(hook, ClothFastener::HookEye { .. }) {
                return Err(DesignError::ParametricParameters.into());
            }
        }
        Ok(())
    }
}

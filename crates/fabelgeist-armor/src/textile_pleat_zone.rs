//! Bounded cloth-profile changes without changing repeat spacing or topology.
use crate::{ClothPleatProfile, DesignError, GenerateError, Permille, TextileRegion};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextilePleatZone {
    pub region: TextileRegion,
    pub profile: ClothPleatProfile,
    /// Profile blend width relative to this region's bounded axes.
    pub feather: Permille,
}

impl TextilePleatZone {
    pub(crate) fn validate(&self) -> Result<(), GenerateError> {
        self.region.validate()?;
        if self.feather.0 == 0 || self.feather.0 > 500 {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    pub(crate) fn blend(&self, point: [f32; 2]) -> f32 {
        let Some(local) = self.region.chart(point) else {
            return 0.;
        };
        let edge = match self.region {
            TextileRegion::Whole => return 1.,
            TextileRegion::Band {
                axis: crate::TextileAxis::Across,
                ..
            } => local[0].min(1. - local[0]),
            TextileRegion::Band {
                axis: crate::TextileAxis::Along,
                ..
            } => local[1].min(1. - local[1]),
            TextileRegion::Panel { .. } => {
                local[0].min(1. - local[0]).min(local[1].min(1. - local[1]))
            }
        };
        let share = (edge / self.feather.unit()).clamp(0., 1.);
        share * share * (3. - 2. * share)
    }
}

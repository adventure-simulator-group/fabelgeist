//! Absolute validity windows for strategic rights grants.

use super::RightsIdentityError;
use adventuresim_world_schema::calendar::StrategicMinute;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RightsValidity {
    pub valid_from_minute: StrategicMinute,
    pub valid_through_minute: Option<StrategicMinute>,
}

impl RightsValidity {
    pub fn try_new(
        valid_from_minute: StrategicMinute,
        valid_through_minute: Option<StrategicMinute>,
    ) -> Result<Self, RightsIdentityError> {
        if valid_through_minute.is_some_and(|through| through < valid_from_minute) {
            return Err(RightsIdentityError::InvalidValidityWindow);
        }
        Ok(Self {
            valid_from_minute,
            valid_through_minute,
        })
    }

    pub fn contains(self, minute: StrategicMinute) -> bool {
        minute >= self.valid_from_minute
            && self
                .valid_through_minute
                .is_none_or(|through| minute <= through)
    }
}

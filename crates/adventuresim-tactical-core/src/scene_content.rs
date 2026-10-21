//! Explicit visual tree family, independent of asset delivery readiness.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::{Precipitation, SceneEnvironment};

#[derive(Debug, thiserror::Error)]
#[error("grown catalogue currently requires clear weather without snow")]
pub struct UnsupportedTreeEnvironment;

/// Missing intent remains pending; it never means an implicit Recipe family.
#[derive(Component, Serialize, Deserialize, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
#[reflect(Component)]
pub enum SceneTreeContent {
    Recipe,
    Grown,
}

impl SceneTreeContent {
    pub fn validate_environment(
        self,
        environment: &SceneEnvironment,
    ) -> Result<(), UnsupportedTreeEnvironment> {
        if self == Self::Grown
            && (environment.weather.precipitation != Precipitation::Clear
                || environment.weather.intensity_bps != 0
                || environment.weather.snow_cover_bps != 0)
        {
            return Err(UnsupportedTreeEnvironment);
        }
        Ok(())
    }
}

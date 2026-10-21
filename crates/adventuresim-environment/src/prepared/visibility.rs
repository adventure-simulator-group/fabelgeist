//! Root-distance visibility, independent of spatial draw-batch ownership.

use std::ops::Range;

use bevy::camera::visibility::VisibilityRange;
use fabelgeist_math::units::Metres;
use serde::{Deserialize, Deserializer, Serialize};

use super::{PreparedError, Result};

/// An unrestricted tree or two positive-width native visibility margins.
/// Finite margins use the production tree material's crisp, quantized handoff,
/// not translucent alpha or screen-door fading. Projection and native culling
/// still bound the rendered view independently of this placement policy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TreeVisibility(Visibility);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Visibility {
    Unbounded,
    Distance {
        near_start: Metres,
        near_end: Metres,
        far_start: Metres,
        far_end: Metres,
    },
}

impl TreeVisibility {
    pub const fn unbounded() -> Self {
        Self(Visibility::Unbounded)
    }

    /// Near and far margins in root-to-LOD-view metres. Both margins must have
    /// positive width; unrestricted trees use the explicit unbounded role.
    pub fn distance(near: Range<Metres>, far: Range<Metres>) -> Result<Self> {
        let visibility = Self(Visibility::Distance {
            near_start: near.start,
            near_end: near.end,
            far_start: far.start,
            far_end: far.end,
        });
        visibility.validate()?;
        Ok(visibility)
    }

    /// Framework/GPU boundary: the same four metre endpoints Bevy supplies
    /// to its range kernel. Unbounded uses no finite range or infinity.
    pub fn native_range(self) -> Option<VisibilityRange> {
        match self.0 {
            Visibility::Unbounded => None,
            Visibility::Distance {
                near_start,
                near_end,
                far_start,
                far_end,
            } => Some(VisibilityRange {
                start_margin: near_start.metres()..near_end.metres(),
                end_margin: far_start.metres()..far_end.metres(),
                use_aabb: false,
            }),
        }
    }

    fn validate(self) -> Result<()> {
        let Some(range) = self.native_range() else {
            return Ok(());
        };
        let endpoints = [
            range.start_margin.start,
            range.start_margin.end,
            range.end_margin.start,
            range.end_margin.end,
        ];
        if !endpoints
            .into_iter()
            .all(|value| value == 0.0 || value.is_normal())
            || !(range.start_margin.end - range.start_margin.start).is_normal()
            || !(range.end_margin.end - range.end_margin.start).is_normal()
            || range.start_margin.start < 0.0
            || range.start_margin.start >= range.start_margin.end
            || range.start_margin.end > range.end_margin.start
            || range.end_margin.start >= range.end_margin.end
        {
            return Err(PreparedError::Invalid("invalid tree visibility margins"));
        }
        Ok(())
    }
}

impl TryFrom<VisibilityRange> for TreeVisibility {
    type Error = PreparedError;

    fn try_from(range: VisibilityRange) -> Result<Self> {
        if range.use_aabb {
            return Err(PreparedError::Invalid(
                "tree visibility requires a root anchor",
            ));
        }
        // Native framework endpoints are metres; admit their units once here.
        Self::distance(
            Metres::from_metres(range.start_margin.start)
                ..Metres::from_metres(range.start_margin.end),
            Metres::from_metres(range.end_margin.start)..Metres::from_metres(range.end_margin.end),
        )
    }
}

impl<'de> Deserialize<'de> for TreeVisibility {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let visibility = Self(Visibility::deserialize(deserializer)?);
        visibility.validate().map_err(serde::de::Error::custom)?;
        Ok(visibility)
    }
}

//! Deterministic source-framed placement fixtures, separate from game generation.
use super::*;
use adventuresim_tactical_core::prelude::SceneTerrain;

// Authored XYZ frame: X/Z horizontal area-world metres; Y is sampled terrain.
const SHARED_ROOT_OFFSETS: [Vec2; 3] = [Vec2::ZERO, Vec2::new(-12.0, 0.0), Vec2::new(12.0, 0.0)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum TreeLayout {
    Single,
    Shared,
}

impl TreeLayout {
    pub(super) fn root_count(self) -> usize {
        match self {
            Self::Single => 1,
            Self::Shared => SHARED_ROOT_OFFSETS.len(),
        }
    }

    pub(super) fn roots(self, terrain: &SceneTerrain) -> Result<Vec<Vec3>> {
        let offsets = match self {
            Self::Single => &SHARED_ROOT_OFFSETS[..1],
            Self::Shared => &SHARED_ROOT_OFFSETS[..],
        };
        offsets
            .iter()
            .map(|offset| {
                let height = terrain
                    .height_at(*offset)
                    .context("review root lacks terrain support")?;
                ensure!(height.is_finite(), "nonfinite review ground");
                Ok(Vec3::new(offset.x, height, offset.y))
            })
            .collect()
    }
}

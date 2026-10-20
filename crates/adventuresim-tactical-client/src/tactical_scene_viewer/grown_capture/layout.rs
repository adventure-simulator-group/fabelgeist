//! Three authored destination placements, independent of botanical identity.

use super::*;
use adventuresim_tactical_core::prelude::SceneTerrain;
use fabelgeist_math::units::Metres;

const ROOT_OFFSETS: [Metres; 3] = [
    Metres::from_metres(-12.0),
    Metres::from_metres(0.0),
    Metres::from_metres(12.0),
];
pub(super) const ROOT_COUNT: usize = ROOT_OFFSETS.len();

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct RootOrdinal(usize);

/// Bevy boundary: right-handed world metres, identity rotation and unit scale.
#[derive(Clone, Copy, Serialize)]
pub(super) struct RootPlacement {
    pub ordinal: RootOrdinal,
    pub frame: Transform,
}

#[derive(Clone, Copy, Serialize)]
pub(super) struct RootPlacements {
    placements: [RootPlacement; ROOT_COUNT],
}

impl RootOrdinal {
    pub(super) fn is_last(self) -> bool {
        self.0 + 1 == ROOT_COUNT
    }
}

impl RootPlacements {
    pub(super) fn on_terrain(terrain: &SceneTerrain) -> Result<Self> {
        let mut placements = Vec::new();
        for (ordinal, offset) in ROOT_OFFSETS.into_iter().enumerate() {
            let x = offset.metres();
            let height = terrain
                .height_at(Vec2::new(x, 0.0))
                .context("grown shared root lacks terrain support")?;
            ensure!(height.is_finite(), "nonfinite grown shared ground");
            placements.push(RootPlacement {
                ordinal: RootOrdinal(ordinal),
                frame: Transform::from_xyz(x, height, 0.0),
            });
        }
        Ok(Self {
            placements: placements
                .try_into()
                .map_err(|_| anyhow::anyhow!("grown shared root cardinality"))?,
        })
    }

    pub(super) fn iter(&self) -> impl ExactSizeIterator<Item = &RootPlacement> {
        self.placements.iter()
    }
}

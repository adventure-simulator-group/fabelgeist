//! Retained fitting outcomes and the independently produced exhibit preview.

use super::{Finish, Region};
use crate::GeneratedArmor;
use adventuresim_character_creator::item_design::ItemDesign;
use bevy::math::Vec3;

/// One catalog item on display, fitted in each of its placements.
pub(crate) struct Exhibit {
    pub name: String,
    pub region: Region,
    pub placements: Vec<adventuresim_core::equipment_assets::OwnedCatalogPlacement>,
    /// Each placement's pending, successful or failed fitting outcome.
    pub fitted: Vec<ExhibitFit>,
    /// A later preview failure, retaining its original cause separately.
    pub(super) preview_error: Option<anyhow::Error>,
    /// The catalog default when the armory opened, for reverting edits.
    pub loaded: ItemDesign,
    /// Whether it is plate steel, which can be built of small plates,
    /// engraved and trimmed.
    pub steel: bool,
    /// The finish used by the last preview attempt, including a failed attempt.
    pub(super) finished: Finish,
    /// Added to the worn position to hang the piece on the wall.
    pub offset: Vec3,
    pub(super) stale: bool,
}

/// Fitting has a pending state distinct from a completed failure.
pub(crate) enum ExhibitFit {
    Pending,
    Fitted(GeneratedArmor),
    Failed(anyhow::Error),
}

impl Exhibit {
    /// Exhibits come from wearable items, each with a nonempty surface placement.
    pub(crate) fn item(&self) -> adventuresim_core::equipment_assets::CatalogItem<'_> {
        self.placements[0].item()
    }

    /// Placements shown on the wall: one side, unless both are asked for.
    pub(crate) fn shown(&self, both_sides: bool) -> usize {
        if both_sides { self.placements.len() } else { 1 }
    }

    /// World bounds of the shown placements where they are worn.
    pub(crate) fn bounds(&self, both_sides: bool) -> Option<(Vec3, Vec3)> {
        self.fitted[..self.shown(both_sides)]
            .iter()
            .filter_map(ExhibitFit::mesh)
            .flat_map(|armor| &armor.positions)
            .map(|p| Vec3::from_array(*p))
            .fold(None, |bounds, p| {
                Some(bounds.map_or((p, p), |(lo, hi): (Vec3, Vec3)| (lo.min(p), hi.max(p))))
            })
    }

    pub(crate) fn errors(&self) -> impl Iterator<Item = &anyhow::Error> {
        self.fitted.iter().filter_map(ExhibitFit::error).chain(self.preview_error.iter())
    }

    pub(crate) fn triangles(&self) -> usize {
        self.fitted
            .iter()
            .filter_map(ExhibitFit::mesh)
            .map(|a| a.indices.len() / 3)
            .sum()
    }
}

impl ExhibitFit {
    pub(crate) fn mesh(&self) -> Option<&GeneratedArmor> {
        match self {
            Self::Fitted(armor) => Some(armor),
            Self::Pending | Self::Failed(_) => None,
        }
    }

    fn error(&self) -> Option<&anyhow::Error> {
        match self {
            Self::Failed(error) => Some(error),
            Self::Pending | Self::Fitted(_) => None,
        }
    }
}

impl From<anyhow::Result<GeneratedArmor>> for ExhibitFit {
    fn from(result: anyhow::Result<GeneratedArmor>) -> Self {
        match result {
            Ok(armor) => Self::Fitted(armor),
            Err(error) => Self::Failed(error),
        }
    }
}

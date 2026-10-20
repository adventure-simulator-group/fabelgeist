//! Native-level bindings on the existing physical obstacle roots.

use super::*;
use adventuresim_environment::prepared::{LibraryId, RecordSpan};
use adventuresim_environment::render::ForestPart;
use adventuresim_tactical_core::prelude::SceneObstacleOf;

#[derive(Clone)]
pub(in crate::presentation) struct GrownTreeStream {
    pub(in crate::presentation) scene: Entity,
    pub(in crate::presentation) visual: Entity,
    pub(super) source: GrownSource,
    bank: LibraryId,
    pub(super) level: Option<Level>,
    pub(super) groups: Option<GrownLevels<Entity>>,
    pub(super) root_frame: Transform,
}

#[derive(Component)]
pub(in crate::presentation) struct GrownVisualFrame;

#[derive(Component)]
pub(super) struct GrownLevelGroup {
    pub root: Entity,
    pub level: Level,
}

/// Actual source/partition binding; renderer proof joins the real Mesh3d/material.
#[derive(Component, Clone, PartialEq, Eq)]
pub(crate) struct GrownPartBinding {
    pub(crate) scene: Entity,
    pub(crate) root: Entity,
    pub(crate) bank: LibraryId,
    pub(crate) source: GrownSource,
    pub(crate) level: Level,
    pub(crate) role: ForestPart,
    pub(crate) records: RecordSpan,
}

impl GrownTreeStream {
    pub(in crate::presentation) fn attachment(&self, root: Entity) -> Result<GrownRootAttachment> {
        Ok(GrownRootAttachment {
            root,
            scene: self.scene,
            visual: self.visual,
            source: self.source,
            bank: self.bank,
            hierarchy: match &self.groups {
                Some(groups) => GrownRootHierarchy::Adaptive {
                    full: groups.full,
                    middle: groups.middle,
                    coarse: groups.coarse,
                },
                None => GrownRootHierarchy::Fixed {
                    level: self.level.context("fixed grown level missing")?,
                },
            },
            frame: self.root_frame,
        })
    }
    pub(in crate::presentation) fn selected_level(&self) -> Option<Level> {
        self.level
    }

    pub(super) fn new(
        scene: Entity,
        visual: Entity,
        source: GrownSource,
        catalogue: &GrownCatalogueAssets,
        selection: LevelSelection,
        root_frame: Transform,
        groups: Option<GrownLevels<Entity>>,
    ) -> Self {
        Self {
            scene,
            visual,
            source,
            bank: catalogue.bank.bank().id(),
            level: match selection {
                LevelSelection::Fixed(level) => Some(level),
                LevelSelection::Adaptive => None,
            },
            groups,
            root_frame,
        }
    }

    /// Native transform and Has<ChildOf> values are checked at the ECS boundary.
    pub(in super::super) fn verify(
        &self,
        cache: &GrownTreeCache,
        root: &Transform,
        owner: Option<&SceneObstacleOf>,
        parented: bool,
        visual_parent: Entity,
        root_entity: Entity,
    ) -> Result<()> {
        ensure!(
            visual_parent == root_entity,
            "grown visual frame belongs to another physical root"
        );
        ensure!(
            owner.is_some_and(|owner| owner.0 == self.scene) && !parented,
            "grown physical root scene/parent binding changed"
        );
        ensure!(
            *root == self.root_frame,
            "grown physical collider frame changed"
        );
        ensure!(
            cache.catalogue.bank.bank().id() == self.bank
                && match cache.selection {
                    LevelSelection::Fixed(level) =>
                        self.level == Some(level) && self.groups.is_none(),
                    LevelSelection::Adaptive => self.groups.is_some(),
                },
            "grown root catalogue/level binding changed"
        );
        let assets = cache
            .catalogue
            .assets(self.source, self.level.unwrap_or(Level::Coarse))?;
        ensure!(
            assets.source() == self.source.plant && assets.variant() == self.source.variant,
            "grown root source binding changed"
        );
        Ok(())
    }
}

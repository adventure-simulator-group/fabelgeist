//! Explicit active scene binding and the asset payload selected for it.

use super::*;
use anyhow::{Context, Result, ensure};

#[derive(Resource, Default)]
pub(in crate::presentation) struct TreePresentationCache {
    pub(super) active: Option<ActiveTreeCache>,
}

pub(super) struct ActiveTreeCache {
    pub binding: TreeSceneBinding,
    pub contents: TreeCacheContents,
}

/// Scene Entity is live authority; the complete environment is reuse identity.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct TreeSceneBinding {
    pub scene: Entity,
    pub environment: SceneEnvironment,
}

pub(in crate::presentation) struct RecipeTreeCache {
    pub(super) variants: std::collections::HashMap<Seed, CachedTreePresentation>,
    pub(super) oak_bark_material: Option<Handle<TacticalTreeBarkMaterial>>,
    pub(super) beech_bark_material: Option<Handle<TacticalTreeBarkMaterial>>,
    pub(super) oak_aggregate_bark_material: Option<Handle<TacticalTreeAggregateBarkMaterial>>,
    pub(super) beech_aggregate_bark_material: Option<Handle<TacticalTreeAggregateBarkMaterial>>,
    pub(super) terrain_heightmap: Handle<Image>,
    pub(super) terrain_height_range: Vec2,
}

pub(super) enum TreeCacheContents {
    Recipe(RecipeTreeCache),
    #[cfg(feature = "grown-trees")]
    Grown(super::grown::GrownTreeCache),
}

impl TreeCacheContents {
    /// Family projection for cross-scene Recipe reuse and its own counters.
    pub(super) fn as_recipe(&self) -> Option<&RecipeTreeCache> {
        match self {
            Self::Recipe(recipe) => Some(recipe),
            #[cfg(feature = "grown-trees")]
            Self::Grown(_) => None,
        }
    }
}

impl RecipeTreeCache {
    pub(in crate::presentation) fn weather_occlusion_branches(
        &self,
        cache_key: Seed,
    ) -> Option<&[super::super::TreeBranchSegment]> {
        self.variants
            .get(&cache_key)
            .map(|cached| cached.branches.as_ref())
    }
}

impl TreePresentationCache {
    #[cfg(feature = "grown-trees")]
    pub(in crate::presentation) fn lease_grown_catalogue(
        &self,
        scene: Entity,
        environment: &SceneEnvironment,
    ) -> Result<std::sync::Arc<super::grown::GrownCatalogueAssets>> {
        // An owned lease keeps this admitted asset closure alive independently
        // of the mutable active-scene cache selector.
        Ok(self.grown_for_scene(scene, environment)?.catalogue.clone())
    }

    #[cfg_attr(
        not(feature = "grown-trees"),
        expect(
            unused_variables,
            reason = "the shared family boundary uses the scene ownership inputs only when the optional grown runtime is compiled"
        )
    )]
    pub(in crate::presentation) fn ready_for_vista(
        &self,
        scene: Entity,
        environment: &SceneEnvironment,
        content: &TreeContentRequest,
    ) -> Result<bool> {
        match content {
            TreeContentRequest::Recipe => Ok(true),
            #[cfg(feature = "grown-trees")]
            TreeContentRequest::Grown(request) => {
                let Some(active) = &self.active else {
                    return Ok(false);
                };
                if active.binding.scene != scene {
                    return Ok(false);
                }
                let cache = self.grown_for_scene(scene, environment)?;
                ensure!(
                    cache.catalogue.matches(request) && cache.selection == request.selection(),
                    "vista grown catalogue contradicts scene content"
                );
                Ok(true)
            }
        }
    }

    #[cfg(feature = "grown-trees")]
    pub(super) fn grown_for_scene(
        &self,
        scene: Entity,
        environment: &SceneEnvironment,
    ) -> Result<&super::grown::GrownTreeCache> {
        let active = self.active.as_ref().context("tree cache is unbound")?;
        ensure!(
            active.binding.scene == scene && active.binding.environment == *environment,
            "grown tree cache belongs to another scene/environment"
        );
        match &active.contents {
            TreeCacheContents::Grown(grown) => Ok(grown),
            TreeCacheContents::Recipe(_) => anyhow::bail!("tree cache contains Recipe content"),
        }
    }

    pub(in crate::presentation) fn recipe(&self) -> Result<&RecipeTreeCache> {
        let active = self.active.as_ref().context("tree cache is unbound")?;
        match &active.contents {
            TreeCacheContents::Recipe(recipe) => Ok(recipe),
            #[cfg(feature = "grown-trees")]
            TreeCacheContents::Grown(_) => anyhow::bail!("tree cache contains grown content"),
        }
    }

    pub(super) fn recipe_mut(&mut self) -> Result<&mut RecipeTreeCache> {
        let active = self.active.as_mut().context("tree cache is unbound")?;
        match &mut active.contents {
            TreeCacheContents::Recipe(recipe) => Ok(recipe),
            #[cfg(feature = "grown-trees")]
            TreeCacheContents::Grown(_) => anyhow::bail!("tree cache contains grown content"),
        }
    }

    pub(in crate::presentation) fn recipe_for_scene(
        &self,
        scene: Entity,
        environment: &SceneEnvironment,
    ) -> Result<&RecipeTreeCache> {
        let active = self.active.as_ref().context("tree cache is unbound")?;
        ensure!(
            active.binding.scene == scene && active.binding.environment == *environment,
            "recipe tree cache belongs to another scene/environment"
        );
        self.recipe()
    }
}

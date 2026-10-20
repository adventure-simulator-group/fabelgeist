//! Grow neither sources nor meshes when an admitted catalogue is already held.

use super::*;
use crate::presentation::obstacles::tree::presentation::{
    ActiveTreeCache, RecipeTreeAssetResidencyDiagnostics, RetainedTreePresentations,
    TreeCacheContents, TreePresentationCache, TreeSceneBinding, residency::RetainedScene,
};

impl RetainedTreePresentations {
    pub(in super::super) fn activate_grown(
        &mut self,
        scene: Entity,
        environment: &SceneEnvironment,
        request: &GrownSceneRequest,
        cache: &mut TreePresentationCache,
        diagnostics: &mut RecipeTreeAssetResidencyDiagnostics,
        world: &mut World,
    ) -> Result<Arc<GrownCatalogueAssets>> {
        adventuresim_tactical_core::prelude::SceneTreeContent::Grown
            .validate_environment(environment)?;
        if let Some(active) = &cache.active
            && active.binding.scene == scene
            && active.binding.environment == *environment
        {
            let grown = cache.grown_for_scene(scene, environment)?;
            ensure!(
                grown.selection == request.selection(),
                "live fixed grown level changed"
            );
            grown.catalogue.verify_reuse(request, world)?;
            return Ok(grown.catalogue.clone());
        }
        let restored = self.scenes.iter().position(|retained| {
            retained.cache.binding.environment == *environment
                && matches!(&retained.cache.contents, TreeCacheContents::Grown(grown)
                    if grown.selection == request.selection() && grown.catalogue.matches(request))
        });
        let incoming = if let Some(index) = restored {
            let retained = &self.scenes[index];
            let TreeCacheContents::Grown(grown) = &retained.cache.contents else {
                anyhow::bail!("retained grown family changed");
            };
            grown.catalogue.verify_reuse(request, world)?;
            let mut incoming = self.scenes.remove(index).context("retained grown scene")?;
            incoming.cache.binding.scene = scene;
            incoming
        } else {
            let catalogue = self
                .matching_catalogue(request, cache)
                .map(|assets| {
                    assets.verify_reuse(request, world)?;
                    Ok::<_, anyhow::Error>(assets)
                })
                .transpose()?;
            let catalogue = match catalogue {
                Some(catalogue) => catalogue,
                None => {
                    GrownCatalogueAssets::build(request, self.matching_bank(request, cache), world)?
                }
            };
            RetainedScene {
                cache: ActiveTreeCache {
                    binding: TreeSceneBinding {
                        scene,
                        environment: environment.clone(),
                    },
                    contents: TreeCacheContents::Grown(GrownTreeCache {
                        catalogue,
                        selection: request.selection(),
                    }),
                },
                // This resource accounts only for Recipe assets, never Grown totals.
                diagnostics: RecipeTreeAssetResidencyDiagnostics::default(),
            }
        };
        self.replace_active(cache, incoming, diagnostics);
        self.update_diagnostics(cache, diagnostics);
        Ok(cache.grown_for_scene(scene, environment)?.catalogue.clone())
    }

    fn matching_catalogue(
        &self,
        request: &GrownSceneRequest,
        cache: &TreePresentationCache,
    ) -> Option<Arc<GrownCatalogueAssets>> {
        cache
            .active
            .iter()
            .chain(self.scenes.iter().rev().map(|retained| &retained.cache))
            .find_map(|active| match &active.contents {
                TreeCacheContents::Grown(grown) if grown.catalogue.matches(request) => {
                    Some(grown.catalogue.clone())
                }
                TreeCacheContents::Recipe(_) | TreeCacheContents::Grown(_) => None,
            })
    }

    fn matching_bank(
        &self,
        request: &GrownSceneRequest,
        cache: &TreePresentationCache,
    ) -> Option<Arc<GrownBankAssets>> {
        cache
            .active
            .iter()
            .chain(self.scenes.iter().rev().map(|retained| &retained.cache))
            .find_map(|active| match &active.contents {
                TreeCacheContents::Grown(grown) => grown.catalogue.bank_for(request),
                TreeCacheContents::Recipe(_) => None,
            })
    }
}

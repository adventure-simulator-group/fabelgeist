//! Admit one complete variant and attach roots through the ordinary streamer.
use super::*;

impl InstalledTreeVariant {
    pub(super) fn install(
        world: &mut World,
        roots: TreeRootBatch,
        source: VariantSource,
    ) -> Result<Vec<InstalledTreeSpecimen>> {
        // Preflight every root before activating or inserting any assets.
        roots.verify(world)?;
        let destination = &roots.destination;
        let (species, origin) = match &source {
            VariantSource::Generated(specimen) => (specimen.species, TreeAssetOrigin::Generated),
            VariantSource::Prepared(variant) => (
                variant.species,
                TreeAssetOrigin::Prepared(variant.identity.clone()),
            ),
        };
        let mut state = SystemState::<(
            ResMut<TreePresentationCache>,
            ResMut<RetainedTreePresentations>,
            ResMut<TreeAssetResidencyDiagnostics>,
            AssetStores,
        )>::new(world);
        let (key, parts, images) = {
            let (mut cache, mut retained, mut diagnostics, mut stores) = state.get_mut(world)?;
            retained.activate(
                &destination.environment,
                &destination.terrain,
                &mut cache,
                &mut diagnostics,
                &mut stores.images,
            );
            let admission = match source {
                VariantSource::Prepared(variant) => {
                    cache.admit_variant(*variant, &destination.terrain, &mut stores)?
                }
                VariantSource::Generated(specimen) => VariantAdmission {
                    key: cache.generate_review_variant(
                        &specimen,
                        &destination.environment,
                        &destination.terrain,
                        &mut stores,
                    )?,
                    assets_added: true,
                },
            };
            let installed = cache
                .variants
                .get(&admission.key)
                .context("installed tree cache entry")?;
            if admission.assets_added {
                diagnostics.add_resident_assets(&installed.resident_assets);
            }
            let parts = inspection::installed_parts(installed, &stores.cards)?;
            let images = stores.verify_parts(&parts)?;
            retained.update_diagnostics(&cache, &mut diagnostics);
            (admission.key, parts, images)
        };
        state.apply(world);
        let variant = Arc::new(Self {
            scene: destination.scene,
            environment: destination.environment.clone(),
            species,
            cache_key: key,
            origin,
            parts,
            images,
        });
        variant.attach(world, roots)
    }

    pub(crate) fn attach(
        self: &Arc<Self>,
        world: &mut World,
        roots: TreeRootBatch,
    ) -> Result<Vec<InstalledTreeSpecimen>> {
        roots.verify(world)?;
        ensure!(
            self.scene == roots.destination.scene
                && serde_json::to_vec(&self.environment)?
                    == serde_json::to_vec(&roots.destination.environment)?,
            "installed tree variant belongs to another scene"
        );
        if let TreeAssetOrigin::Prepared(identity) = &self.origin {
            identity.verify_environment(&roots.destination.environment)?;
        }
        let mut state = SystemState::<(Res<TreePresentationCache>, AssetStores)>::new(world);
        {
            let (cache, stores) = state.get_mut(world)?;
            ensure!(
                cache.terrain_scene_digest.as_deref()
                    == Some(self.environment.scene_digest.as_str()),
                "installed tree variant is not active"
            );
            let cached = cache
                .variants
                .get(&self.cache_key)
                .context("installed tree variant was evicted")?;
            ensure!(
                cached.origin == self.origin,
                "installed tree variant binding changed"
            );
            require_resident(cached, COMPLETE_STREAM_MASK)?;
            ensure!(
                inspection::installed_parts(cached, &stores.cards)? == self.parts,
                "installed tree variant part binding changed"
            );
            ensure!(
                stores.verify_parts(&self.parts)? == self.images,
                "installed tree material image binding changed"
            );
        }
        state.apply(world);
        let mut specimens = Vec::with_capacity(roots.positions.len());
        for position in roots.positions {
            let root = world
                .spawn((
                    Transform::from_translation(
                        position + super::super::super::source::source_normalization(),
                    ),
                    Visibility::Inherited,
                ))
                .id();
            spawn_cached_tree(
                &mut world.commands(),
                root,
                self.cache_key,
                self.species.name(),
            );
            specimens.push(InstalledTreeSpecimen {
                root,
                variant: Arc::clone(self),
            });
        }
        world.flush();
        Ok(specimens)
    }
}

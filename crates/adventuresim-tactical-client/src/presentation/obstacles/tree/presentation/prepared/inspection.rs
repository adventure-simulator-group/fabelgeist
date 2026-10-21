//! Exact retained cache identities and selected streamer inspection.
use super::*;

pub(super) fn installed_parts(
    cached: &CachedTreePresentation,
    cards: &Assets<TacticalTreeImpostorMaterial>,
) -> Result<Vec<InstalledTreePart>> {
    let mut parts = vec![
        InstalledTreePart {
            role: InstalledTreePartRole::DetailedTrunk,
            mesh: cached
                .trunk_mesh
                .as_ref()
                .context("installed detailed trunk")?
                .id(),
            material: InstalledTreeMaterial::Bark(cached.bark_material.id()),
        },
        InstalledTreePart {
            role: InstalledTreePartRole::MiddleTrunk,
            mesh: cached
                .mid_trunk_mesh
                .as_ref()
                .context("installed mid trunk")?
                .id(),
            material: InstalledTreeMaterial::Bark(cached.bark_material.id()),
        },
    ];
    let groups = cached.clusters.as_ref().context("installed groups")?;
    for group in groups {
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::Wood {
                primary_group: group.primary_group,
            },
            mesh: group
                .detailed_branch_mesh
                .as_ref()
                .context("installed crown wood")?
                .id(),
            material: InstalledTreeMaterial::Bark(cached.bark_material.id()),
        });
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::Buds {
                primary_group: group.primary_group,
            },
            mesh: group.bud_mesh.as_ref().context("installed buds")?.id(),
            material: InstalledTreeMaterial::Buds(cached.bud_material.id()),
        });
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::CamberedLeaves {
                primary_group: group.primary_group,
            },
            mesh: group
                .cambered_leaf_mesh
                .as_ref()
                .context("installed cambered leaves")?
                .id(),
            material: InstalledTreeMaterial::Leaves(cached.leaf_material.id()),
        });
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::FlatLeaves {
                primary_group: group.primary_group,
            },
            mesh: group
                .leaf_card_mesh
                .as_ref()
                .context("installed flat leaves")?
                .id(),
            material: InstalledTreeMaterial::Leaves(cached.leaf_material.id()),
        });
    }
    for (index, mesh) in cached.aggregate_branch_meshes.iter().enumerate() {
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::AggregateWood {
                lod: super::super::super::AggregateTreeLod::ALL[index],
            },
            mesh: mesh.as_ref().context("installed aggregate wood")?.id(),
            material: InstalledTreeMaterial::Aggregate(cached.aggregate_bark_material.id()),
        });
    }
    for (index, card) in cached.lod_cards.iter().enumerate() {
        let card = card.as_ref().context("installed canopy tier")?;
        let material = cards
            .get(&card.material)
            .context("installed canopy material")?;
        parts.push(InstalledTreePart {
            role: InstalledTreePartRole::CanopyCard {
                lod: index as u8 + 1,
            },
            mesh: card.mesh.id(),
            material: InstalledTreeMaterial::Card {
                material: card.material.id(),
                image: material.baked_color.id(),
            },
        });
    }
    Ok(parts)
}

/// Read-only cache inspection, including the actual screenshot observer.
#[derive(SystemParam)]
pub(crate) struct TreeCacheInspection<'w, 's> {
    cache: Res<'w, TreePresentationCache>,
    active: Res<'w, ActiveTacticalScene>,
    environments: Query<'w, 's, &'static SceneEnvironment>,
    requests: Query<'w, 's, &'static super::super::TreeContentRequest>,
    streams: Query<'w, 's, &'static StreamedTreePresentation>,
    meshes: Res<'w, Assets<Mesh>>,
    cards: Res<'w, Assets<TacticalTreeImpostorMaterial>>,
    images: Res<'w, Assets<Image>>,
    bark: Res<'w, Assets<TacticalTreeBarkMaterial>>,
    aggregate: Res<'w, Assets<TacticalTreeAggregateBarkMaterial>>,
    leaves: Res<'w, Assets<TacticalTreeLeafCardMaterial>>,
    buds: Res<'w, Assets<StandardMaterial>>,
}

impl TreeCacheInspection<'_, '_> {
    pub(crate) fn grown_root(
        &self,
        root: Entity,
    ) -> Result<super::super::grown::GrownRootAttachment> {
        let StreamedTreePresentation::Grown(stream) = self.streams.get(root)? else {
            anyhow::bail!("actual generated root has Recipe stream");
        };
        stream.attachment(root)
    }

    pub(crate) fn grown_level(
        &self,
        root: Entity,
    ) -> Result<Option<adventuresim_environment::render::Level>> {
        let StreamedTreePresentation::Grown(stream) = self.streams.get(root)? else {
            anyhow::bail!("actual generated root has Recipe stream");
        };
        Ok(stream.selected_level())
    }

    pub(crate) fn grown_scene(&self) -> Result<super::super::grown::GrownSceneAssets> {
        let scene = self
            .active
            .entity
            .context("grown cache inspection requires an active scene")?;
        let environment = self.environments.get(scene)?;
        let super::super::TreeContentRequest::Grown(request) = self.requests.get(scene)? else {
            anyhow::bail!("scene requests Recipe content");
        };
        adventuresim_tactical_core::prelude::SceneTreeContent::Grown
            .validate_environment(environment)?;
        let cache = self.cache.grown_for_scene(scene, environment)?;
        ensure!(
            cache.catalogue.matches(request) && cache.selection == request.selection(),
            "grown cache differs from actual scene intent"
        );
        Ok(super::super::grown::GrownSceneAssets::from_cache(
            scene, cache,
        ))
    }

    pub(crate) fn verify_card(
        &self,
        installed: &InstalledTreeSpecimen,
        role: InstalledTreePartRole,
        actual: &TreeImpostorProvenance,
    ) -> Result<()> {
        let InstalledTreePartRole::CanopyCard { lod } = role else {
            anyhow::bail!("card provenance supplied for non-card role");
        };
        ensure!(actual.lod == lod, "card child provenance tier drift");
        let index = usize::from(lod)
            .checked_sub(1)
            .context("card child tier zero")?;
        let expected = self
            .cache
            .recipe_for_scene(installed.variant.scene, &installed.variant.environment)?
            .variants
            .get(&installed.variant.cache_key)
            .and_then(|cached| cached.lod_cards.get(index))
            .and_then(Option::as_ref)
            .context("owned card has no production cache record")?;
        ensure!(
            expected.provenance == *actual,
            "owned card provenance drift"
        );
        Ok(())
    }

    pub(crate) fn verify_cluster(
        &self,
        installed: &InstalledTreeSpecimen,
        actual: &TreeLodCluster,
    ) -> Result<()> {
        let expected = self
            .cache
            .recipe_for_scene(installed.variant.scene, &installed.variant.environment)?
            .variants
            .get(&installed.variant.cache_key)
            .and_then(|cached| cached.clusters.as_ref())
            .and_then(|groups| {
                groups
                    .iter()
                    .find(|group| group.primary_group == actual.primary_group)
            })
            .context("owned crown group has no production cache record")?;
        ensure!(
            actual.center == expected.center && actual.radius == expected.radius,
            "owned crown cluster layout drift"
        );
        Ok(())
    }

    pub(crate) fn verify(
        &self,
        installed: &InstalledTreeSpecimen,
        representation: TreeCacheRepresentation,
    ) -> Result<()> {
        let stream = self.streams.get(installed.root)?;
        installed.verify_records(&self.cache, &self.cards, stream, representation)?;
        ensure!(
            self.active.entity == Some(installed.variant.scene),
            "inspected tree variant belongs to another scene"
        );
        ensure!(
            serde_json::to_vec(self.environments.get(installed.variant.scene)?)?
                == serde_json::to_vec(&installed.variant.environment)?,
            "inspected tree environment changed"
        );
        ensure!(
            dependencies::PartDependencies {
                meshes: &self.meshes,
                materials: &self.buds,
                bark: &self.bark,
                aggregate_bark: &self.aggregate,
                leaves: &self.leaves,
                cards: &self.cards,
                images: &self.images,
            }
            .verify(&installed.variant.parts)?
                == installed.variant.images,
            "retained prepared material image binding drift"
        );
        Ok(())
    }
}

impl InstalledTreeSpecimen {
    /// Inspect the actual retained cache and streamer without generating assets.
    fn verify_records(
        &self,
        cache: &TreePresentationCache,
        cards: &Assets<TacticalTreeImpostorMaterial>,
        stream: &StreamedTreePresentation,
        representation: TreeCacheRepresentation,
    ) -> Result<()> {
        let stream = stream.recipe()?;
        let cached = cache
            .recipe_for_scene(self.variant.scene, &self.variant.environment)?
            .variants
            .get(&self.variant.cache_key)
            .context("retained prepared entry")?;
        ensure!(
            cached.origin == self.variant.origin,
            "prepared origin/content drift"
        );
        require_resident(cached, COMPLETE_STREAM_MASK)?;
        ensure!(
            installed_parts(cached, cards)? == self.variant.parts,
            "retained cache part binding drift"
        );
        ensure!(
            stream.cache_key == self.variant.cache_key
                && stream.resident_mask == representation.stream_mask()
                && stream.active_mask == representation.stream_mask()
                && stream.resident_leaf_mask == representation.leaf_mask()
                && representation
                    .leaf()
                    .is_none_or(|leaf| stream.active_leaf == Some(leaf)),
            "actual cache root fixed representation drift"
        );
        Ok(())
    }
}

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
                material: card.material.id().untyped(),
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
            .variants
            .get(&installed.cache_key)
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
            .variants
            .get(&installed.cache_key)
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
        for part in &installed.parts {
            ensure!(
                self.meshes.contains(part.mesh),
                "retained prepared mesh missing"
            );
            let present = match part.material {
                InstalledTreeMaterial::Bark(id) => self.bark.contains(id),
                InstalledTreeMaterial::Aggregate(id) => self.aggregate.contains(id),
                InstalledTreeMaterial::Leaves(id) => self.leaves.contains(id),
                InstalledTreeMaterial::Buds(id) => self.buds.contains(id),
                InstalledTreeMaterial::Card { material, image } => {
                    self.cards
                        .contains(material.try_typed::<TacticalTreeImpostorMaterial>()?)
                        && self.images.contains(image)
                }
            };
            ensure!(present, "retained prepared material/image missing");
        }
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
        let cached = cache
            .variants
            .get(&self.cache_key)
            .context("retained prepared entry")?;
        ensure!(
            match (&cached.origin, self.origin) {
                (TreeAssetOrigin::Generated, InstalledTreeOrigin::Generated) => true,
                (TreeAssetOrigin::Prepared(actual), InstalledTreeOrigin::Prepared(expected)) =>
                    *actual == expected,
                _ => false,
            },
            "prepared origin/content drift"
        );
        require_resident(cached, COMPLETE_STREAM_MASK)?;
        ensure!(
            installed_parts(cached, cards)? == self.parts,
            "retained cache part binding drift"
        );
        ensure!(
            stream.cache_key == self.cache_key
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

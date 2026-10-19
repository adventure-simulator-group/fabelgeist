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
pub(crate) struct PreparedCacheInspection<'w, 's> {
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

impl PreparedCacheInspection<'_, '_> {
    pub(crate) fn verify(&self, installed: &InstalledCacheSpecimen) -> Result<()> {
        let stream = self.streams.get(installed.root)?;
        installed.verify_records(&self.cache, &self.cards, stream)?;
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

impl InstalledCacheSpecimen {
    /// Inspect the actual retained cache and streamer without generating assets.
    fn verify_records(
        &self,
        cache: &TreePresentationCache,
        cards: &Assets<TacticalTreeImpostorMaterial>,
        stream: &StreamedTreePresentation,
    ) -> Result<()> {
        let cached = cache
            .variants
            .get(&self.cache_key)
            .context("retained prepared entry")?;
        ensure!(
            matches!(cached.origin, TreeAssetOrigin::Prepared(content) if content == self.content_id),
            "prepared origin/content drift"
        );
        require_resident(cached, COMPLETE_STREAM_MASK)?;
        ensure!(
            installed_parts(cached, cards)? == self.parts,
            "retained cache part binding drift"
        );
        ensure!(
            stream.cache_key == self.cache_key
                && stream.resident_mask == FULL_STREAM_MASK
                && stream.active_mask == FULL_STREAM_MASK
                && stream.resident_leaf_mask == 1
                && stream.active_leaf == Some(TreeLeafRepresentation::TexturedMesh),
            "actual cache root is not fixed Full/cambered"
        );
        Ok(())
    }
}

//! Untimed local admission controls for the explicit shared-root capture fixture.
use super::*;
use serde::Serialize;

use footprint::Footprint;

mod footprint;

#[derive(Serialize)]
pub(crate) struct RootAdmissionReport {
    controls: Vec<RootControl>,
}

#[derive(Serialize)]
struct RootControl {
    case: RootControlCase,
    outcome: RootControlOutcome,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RootControlCase {
    InvalidSecondRoot,
    EmptyBatch,
    StaleRootBatch,
    StaleVariant,
    ChangedEnvironment,
    EvictedVariant,
    ConflictingOrigin,
    MissingMesh,
    MissingBarkMaterial,
    MissingAggregateMaterial,
    MissingLeafMaterial,
    MissingBudMaterial,
    MissingCardMaterial,
    MissingImage { image: AssetId<Image> },
    LaterAttachment,
    Readmission,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum RootControlOutcome {
    RejectedWithoutMutation,
    AttachedWithoutNewAssets,
    ReadmittedWithoutNewAssets,
}

impl RootAdmissionReport {
    pub(crate) fn run(
        world: &mut World,
        installed: &InstalledTreeSpecimen,
        repeated: PreparedTreeCache,
        positions: &[Vec3],
    ) -> Result<Self> {
        let mut report = Self {
            controls: Vec::new(),
        };
        let position = *positions.first().context("control root position")?;
        let variant = &installed.variant;
        let before = Footprint::capture(world)?;
        let invalid = TreeRootBatch::from_area_world_metres(world, [position, Vec3::NAN]);
        ensure!(invalid.is_err(), "invalid second root was admitted");
        before.verify(world)?;
        report.rejected(RootControlCase::InvalidSecondRoot);
        let empty = TreeRootBatch::from_area_world_metres(world, []);
        ensure!(empty.is_err(), "empty root batch was admitted");
        before.verify(world)?;
        report.rejected(RootControlCase::EmptyBatch);
        report.scene_controls(world, variant, position)?;
        report.cache_controls(world, variant, position)?;
        report.asset_controls(world, variant, position)?;
        report.attachment_controls(world, variant, repeated, position)?;
        before.verify(world)?;
        Ok(report)
    }

    fn rejected(&mut self, case: RootControlCase) {
        self.controls.push(RootControl {
            case,
            outcome: RootControlOutcome::RejectedWithoutMutation,
        });
    }

    fn scene_controls(
        &mut self,
        world: &mut World,
        variant: &Arc<InstalledTreeVariant>,
        position: Vec3,
    ) -> Result<()> {
        let before = Footprint::capture(world)?;
        let roots = TreeRootBatch::from_area_world_metres(world, [position])?;
        let environment = world
            .get::<SceneEnvironment>(variant.scene)
            .context("control environment")?
            .clone();
        let terrain = world
            .get::<SceneTerrain>(variant.scene)
            .context("control terrain")?
            .clone();
        let other_scene = world.spawn((environment.clone(), terrain)).id();
        world.resource_mut::<ActiveTacticalScene>().entity = Some(other_scene);
        let stale_batch = variant.attach(world, roots);
        let stale_variant = TreeRootBatch::from_area_world_metres(world, [position])
            .and_then(|roots| variant.attach(world, roots));
        world.resource_mut::<ActiveTacticalScene>().entity = Some(variant.scene);
        world.despawn(other_scene);
        ensure!(
            stale_batch.is_err() && stale_variant.is_err(),
            "stale scene authority accepted"
        );
        before.verify(world)?;
        self.rejected(RootControlCase::StaleRootBatch);
        self.rejected(RootControlCase::StaleVariant);
        let mut changed = environment.clone();
        changed.absolute_minute = changed.absolute_minute.saturating_add_minutes(1);
        ensure!(
            changed.absolute_minute != environment.absolute_minute,
            "control minute saturated"
        );
        world.entity_mut(variant.scene).insert(changed);
        let result = TreeRootBatch::from_area_world_metres(world, [position])
            .and_then(|roots| variant.attach(world, roots));
        world.entity_mut(variant.scene).insert(environment);
        ensure!(result.is_err(), "changed full environment accepted");
        before.verify(world)?;
        self.rejected(RootControlCase::ChangedEnvironment);
        Ok(())
    }

    fn cache_controls(
        &mut self,
        world: &mut World,
        variant: &Arc<InstalledTreeVariant>,
        position: Vec3,
    ) -> Result<()> {
        let before = Footprint::capture(world)?;
        let cached = world
            .resource_mut::<TreePresentationCache>()
            .variants
            .remove(&variant.cache_key)
            .context("control cache entry")?;
        let removed = Footprint::capture(world)?;
        let roots = TreeRootBatch::from_area_world_metres(world, [position])?;
        let result = variant.attach(world, roots);
        removed.verify(world)?;
        world
            .resource_mut::<TreePresentationCache>()
            .variants
            .insert(variant.cache_key, cached);
        ensure!(result.is_err(), "evicted variant regenerated or accepted");
        before.verify(world)?;
        self.rejected(RootControlCase::EvictedVariant);
        let original = {
            let mut cache = world.resource_mut::<TreePresentationCache>();
            let cached = cache
                .variants
                .get_mut(&variant.cache_key)
                .context("control origin")?;
            std::mem::replace(&mut cached.origin, TreeAssetOrigin::Generated)
        };
        let changed = Footprint::capture(world)?;
        let roots = TreeRootBatch::from_area_world_metres(world, [position])?;
        let result = variant.attach(world, roots);
        changed.verify(world)?;
        world
            .resource_mut::<TreePresentationCache>()
            .variants
            .get_mut(&variant.cache_key)
            .context("restore control origin")?
            .origin = original;
        ensure!(result.is_err(), "conflicting cache origin accepted");
        before.verify(world)?;
        self.rejected(RootControlCase::ConflictingOrigin);
        Ok(())
    }

    fn asset_controls(
        &mut self,
        world: &mut World,
        variant: &Arc<InstalledTreeVariant>,
        position: Vec3,
    ) -> Result<()> {
        let part = variant.parts.first().context("control mesh")?;
        Self::missing_asset(world, variant, position, part.mesh)?;
        self.rejected(RootControlCase::MissingMesh);
        let mut materials = std::collections::HashSet::new();
        for part in &variant.parts {
            if !materials.insert(part.material) {
                continue;
            }
            let case = match part.material {
                InstalledTreeMaterial::Bark(id) => {
                    Self::missing_asset(world, variant, position, id)?;
                    RootControlCase::MissingBarkMaterial
                }
                InstalledTreeMaterial::Aggregate(id) => {
                    Self::missing_asset(world, variant, position, id)?;
                    RootControlCase::MissingAggregateMaterial
                }
                InstalledTreeMaterial::Leaves(id) => {
                    Self::missing_asset(world, variant, position, id)?;
                    RootControlCase::MissingLeafMaterial
                }
                InstalledTreeMaterial::Buds(id) => {
                    Self::missing_asset(world, variant, position, id)?;
                    RootControlCase::MissingBudMaterial
                }
                InstalledTreeMaterial::Card { material, .. } => {
                    Self::missing_asset(world, variant, position, material)?;
                    RootControlCase::MissingCardMaterial
                }
            };
            self.rejected(case);
        }
        let mut images: Vec<_> = variant.images.iter().copied().collect();
        images.sort_unstable();
        for image in images {
            Self::missing_asset(world, variant, position, image)?;
            self.rejected(RootControlCase::MissingImage { image });
        }
        Ok(())
    }

    fn missing_asset<A: Asset>(
        world: &mut World,
        variant: &Arc<InstalledTreeVariant>,
        position: Vec3,
        id: AssetId<A>,
    ) -> Result<()> {
        let before = Footprint::capture(world)?;
        let value = world
            .resource_mut::<Assets<A>>()
            .remove(id)
            .context("control dependency")?;
        let removed = Footprint::capture(world)?;
        let roots = TreeRootBatch::from_area_world_metres(world, [position])?;
        let result = variant.attach(world, roots);
        removed.verify(world)?;
        world.resource_mut::<Assets<A>>().insert(id, value)?;
        ensure!(
            result.is_err(),
            "missing dependency accepted or regenerated"
        );
        before.verify(world)?;
        Ok(())
    }

    fn attachment_controls(
        &mut self,
        world: &mut World,
        variant: &Arc<InstalledTreeVariant>,
        repeated: PreparedTreeCache,
        position: Vec3,
    ) -> Result<()> {
        let before = Footprint::capture(world)?;
        let positions = TreeRootBatch::from_area_world_metres(world, [position])?;
        let roots = variant.attach(world, positions)?;
        ensure!(
            roots.len() == 1 && roots.iter().all(|root| Arc::ptr_eq(&root.variant, variant)),
            "later attachment lost shared authority"
        );
        before.verify_assets(world)?;
        for root in roots {
            world.despawn(root.root);
        }
        before.verify(world)?;
        self.controls.push(RootControl {
            case: RootControlCase::LaterAttachment,
            outcome: RootControlOutcome::AttachedWithoutNewAssets,
        });
        let positions = TreeRootBatch::from_area_world_metres(world, [position])?;
        let roots = repeated.spawn(world, positions)?;
        ensure!(
            roots.len() == 1
                && roots
                    .iter()
                    .all(|root| root.variant.origin == variant.origin
                        && root.variant.parts == variant.parts
                        && root.variant.images == variant.images),
            "readmission changed shared bindings"
        );
        before.verify_assets(world)?;
        for root in roots {
            world.despawn(root.root);
        }
        before.verify(world)?;
        self.controls.push(RootControl {
            case: RootControlCase::Readmission,
            outcome: RootControlOutcome::ReadmittedWithoutNewAssets,
        });
        Ok(())
    }
}

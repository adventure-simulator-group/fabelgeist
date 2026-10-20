//! Retained tag admission is distinct from per-frame GPU proof capacity.

use super::*;

pub(super) struct TagCapacity(usize);

#[derive(Default)]
pub(super) struct RetainedTaggedMembers {
    entities: Vec<Entity>,
}

impl Default for TagCapacity {
    fn default() -> Self {
        Self(MAX_FIXTURE_MEMBERS)
    }
}

impl TagCapacity {
    /// Native collection cardinalities at a finite capture admission boundary.
    fn for_checkpoint_census(installed: usize, checkpoints: usize) -> Result<Self> {
        ensure!(
            checkpoints > 0,
            "tag census requires a finite checkpoint sequence"
        );
        let background_union = checkpoints
            .checked_mul(MAX_FIXTURE_MEMBERS)
            .context("finite background tag capacity overflow")?;
        let capacity = installed
            .checked_add(background_union)
            .context("retained tag capacity overflow")?;
        ensure!(
            capacity < u32::MAX as usize,
            "retained tag identity capacity overflow"
        );
        Ok(Self(capacity))
    }
}

impl OrdinaryBatches {
    pub(in super::super) fn admit_retained_members(
        &self,
        world: &mut World,
        mut entities: Vec<Entity>,
        checkpoints: usize,
    ) -> Result<serde_json::Value> {
        let declared = entities.len();
        entities.sort_unstable();
        entities.dedup();
        ensure!(
            entities.len() == declared,
            "retained census contains duplicate entities"
        );
        ensure!(
            self.0
                .tags
                .lock()
                .map_err(|_| anyhow::anyhow!("tree tag lock"))?
                .is_empty(),
            "tag census must be admitted before first observation"
        );
        let capacity = TagCapacity::for_checkpoint_census(declared, checkpoints)?;
        let capacity_count = capacity.0;
        *self
            .0
            .tag_capacity
            .lock()
            .map_err(|_| anyhow::anyhow!("tree tag capacity lock"))? = capacity;
        self.tag_entities(world, entities.clone())?;
        self.0
            .retained_tags
            .lock()
            .map_err(|_| anyhow::anyhow!("retained tag lock"))?
            .entities = entities;
        Ok(
            serde_json::json!({"installed_tree_members":declared,"checkpoint_count":checkpoints,"per_frame_member_limit":MAX_FIXTURE_MEMBERS,"retained_tag_capacity":capacity_count,"derivation":"installed tree members plus one full per-frame background bound per finite checkpoint; active GPU limits unchanged"}),
        )
    }

    pub(in super::super) fn prepare_tags(
        &self,
        world: &mut World,
        inventory: &inventory::CurrentInventory,
    ) -> Result<()> {
        // Active expected-members/readback limits retain their original owner.
        self.check_retained_tags(|entity| world.get::<MeshTag>(entity).cloned())?;
        inventory.validate_capacity()?;
        let entities = inventory
            .members
            .iter()
            .map(|member| member.entity)
            .chain(inventory.background.iter().map(|member| member.entity))
            .collect();
        self.tag_entities(world, entities)
    }

    pub(super) fn check_retained_tags(
        &self,
        lookup: impl Fn(Entity) -> Option<MeshTag>,
    ) -> Result<()> {
        let retained = self
            .0
            .retained_tags
            .lock()
            .map_err(|_| anyhow::anyhow!("retained tag lock"))?;
        let saved = self
            .0
            .tags
            .lock()
            .map_err(|_| anyhow::anyhow!("tree tag lock"))?;
        for entity in &retained.entities {
            let expected = saved
                .get(entity)
                .context("retained tree tag identity missing")?;
            ensure!(
                lookup(*entity).is_some_and(|actual| actual.0 == expected.0),
                "retained tree tag removed or changed, including inactive level"
            );
        }
        Ok(())
    }

    fn tag_entities(&self, world: &mut World, mut entities: Vec<Entity>) -> Result<()> {
        let capacity = self
            .0
            .tag_capacity
            .lock()
            .map_err(|_| anyhow::anyhow!("tree tag capacity lock"))?
            .0;
        let mut saved = self
            .0
            .tags
            .lock()
            .map_err(|_| anyhow::anyhow!("tree tag lock"))?;
        for (entity, actual) in world.query::<(Entity, &MeshTag)>().iter(world) {
            ensure!(
                saved
                    .get(&entity)
                    .is_some_and(|expected| expected.0 == actual.0)
                    || (!saved.contains_key(&entity) && actual.0 == 0),
                "unexpected diagnostic mesh tag"
            );
        }
        entities.sort_unstable();
        entities.dedup();
        let new = entities
            .iter()
            .filter(|entity| !saved.contains_key(entity))
            .count();
        ensure!(
            saved
                .len()
                .checked_add(new)
                .is_some_and(|needed| needed <= capacity),
            "complete retained tag census exceeds admitted capacity"
        );
        for entity in entities {
            if let Some(tag) = saved.get(&entity) {
                ensure!(
                    world
                        .get::<MeshTag>(entity)
                        .is_some_and(|actual| actual.0 == tag.0),
                    "owned tag disappeared or changed"
                );
                continue;
            }
            let tag = MeshTag(u32::try_from(
                saved.len().checked_add(1).context("tree tag overflow")?,
            )?);
            saved.insert(entity, tag.clone());
            world.entity_mut(entity).insert(tag);
        }
        Ok(())
    }
}

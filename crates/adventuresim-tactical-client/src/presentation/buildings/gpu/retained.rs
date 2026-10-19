//! Transfer complete city buffers between the active owner and a bounded cache.
use super::*;

/// GPU assets and hidden draw anchors belonging to one inactive city.
pub(in crate::presentation) struct RetainedCityGeometry {
    owner: PresentationOwner,
    scene: CityGpuScene,
}

impl RetainedCityGeometry {
    /// Detach only a fully prepared city; incomplete candidates remain active.
    pub(in crate::presentation) fn take(
        world: &mut World,
        owner: PresentationOwner,
        root: Entity,
    ) -> Option<Self> {
        if !is_ready(owner) {
            return None;
        }
        if world
            .get_resource::<CityGpuScenes>()?
            .owners
            .get(owner)
            .publication_root
            != Some(root)
        {
            return None;
        }
        let scene = std::mem::take(
            world
                .get_resource_mut::<CityGpuScenes>()?
                .owners
                .get_mut(owner),
        );
        READY.get(owner).store(false, Ordering::Relaxed);
        for batch in scene.batches.iter() {
            if let Ok(mut entity) = world.get_entity_mut(batch.anchor) {
                entity.insert((Visibility::Hidden, RetainedCityAnchor));
            }
        }
        Some(Self { owner, scene })
    }

    /// Restore the exact handles; render extraction must acknowledge them again.
    pub(in crate::presentation) fn restore(self, world: &mut World) {
        reset(world, self.owner);
        for batch in self.scene.batches.iter() {
            if let Ok(mut entity) = world.get_entity_mut(batch.anchor) {
                entity.remove::<RetainedCityAnchor>();
            }
        }
        let empty = self.scene.count == 0;
        *world
            .resource_mut::<CityGpuScenes>()
            .owners
            .get_mut(self.owner) = self.scene;
        READY.get(self.owner).store(empty, Ordering::Relaxed);
    }

    pub(in crate::presentation) fn discard(self, world: &mut World) {
        self.scene.discard(world);
    }
}

impl CityGpuScene {
    pub(super) fn discard(self, world: &mut World) {
        for batch in self.batches.iter() {
            if let Ok(entity) = world.get_entity_mut(batch.anchor) {
                entity.despawn();
            }
        }
    }
}

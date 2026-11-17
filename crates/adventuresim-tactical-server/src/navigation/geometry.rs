//! Snapshot the authoritative collision input at every navigation mesh request.

use super::*;
use bevy::ecs::{change_detection::Tick, system::SystemParam};
use std::collections::BTreeMap;

type NavigableColliders<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ColliderOf,
        Ref<'static, Collider>,
        &'static adventuresim_tactical_core::avian3d::prelude::Position,
        &'static Rotation,
    ),
>;

#[derive(Clone, Copy, Debug)]
pub(super) struct ColliderSnapshot {
    pub(super) position: Vec3,
    pub(super) rotation: Quat,
    pub(super) shape_revision: Tick,
}

#[derive(SystemParam)]
pub(super) struct NavigationGeometry<'w, 's> {
    colliders: NavigableColliders<'w, 's>,
    bodies: Query<'w, 's, (&'static RigidBody, Has<SceneDoor>, Has<SceneWindow>)>,
}

impl NavigationGeometry<'_, '_> {
    pub(super) fn has_static_surface(&self) -> bool {
        self.colliders.iter().any(|(_, owner, _, _, _)| {
            self.bodies
                .get(owner.body)
                .is_ok_and(|(body, door, window)| is_navigation_surface(body, door, window))
        })
    }

    pub(super) fn snapshot(&self) -> BTreeMap<Entity, ColliderSnapshot> {
        self.colliders
            .iter()
            .filter_map(|(entity, owner, collider, position, rotation)| {
                let (body, door, window) = self.bodies.get(owner.body).ok()?;
                is_navigation_surface(body, door, window).then_some((
                    entity,
                    ColliderSnapshot {
                        position: position.0,
                        rotation: rotation.0,
                        shape_revision: collider.last_changed(),
                    },
                ))
            })
            .collect()
    }
}

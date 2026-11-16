//! The playable terrain window bounds navigation, even when accepted support
//! topology includes distant geographic rings for presentation.

use adventuresim_tactical_core::prelude::*;
use bevy::{ecs::system::SystemParam, math::bounding::Aabb3d, prelude::*};

type BoundedColliders<'w, 's> = Query<
    'w,
    's,
    (
        &'static Collider,
        &'static ColliderAabb,
        &'static ColliderOf,
    ),
>;

#[derive(SystemParam)]
pub(super) struct NavigationScene<'w, 's> {
    terrain: Query<'w, 's, &'static SceneTerrain>,
    colliders: BoundedColliders<'w, 's>,
    bodies: Query<'w, 's, (&'static RigidBody, Has<SceneDoor>, Has<SceneWindow>)>,
}

impl NavigationScene<'_, '_> {
    pub(super) fn bounds(&self) -> Option<Aabb3d> {
        let terrain = self.terrain.single().ok()?;
        let half_width = terrain.width() * 0.5;
        let half_depth = terrain.depth() * 0.5;
        let mut bottom = f32::INFINITY;
        let mut top = f32::NEG_INFINITY;
        for (collider, bounds, owner) in &self.colliders {
            let Ok((body, door, window)) = self.bodies.get(owner.body) else {
                continue;
            };
            // Invisible boundary half-spaces constrain physics but have no
            // triangulated navigation surface or finite vertical extent.
            if (!body.is_static() && !door && !window)
                || collider.shape_scaled().as_halfspace().is_some()
                || bounds.min.x > half_width
                || bounds.max.x < -half_width
                || bounds.min.z > half_depth
                || bounds.max.z < -half_depth
            {
                continue;
            }
            bottom = bottom.min(bounds.min.y);
            top = top.max(bounds.max.y);
        }
        if !bottom.is_finite() || !top.is_finite() || bottom > top {
            return None;
        }
        // SceneTerrain is centred at the scene origin. Preserve the collision
        // world's vertical range and agent headroom while admitting only its
        // playable X/Z window. A flat surface still needs a nonempty volume.
        Some(Aabb3d {
            min: Vec3::new(-half_width, bottom, -half_depth).into(),
            max: Vec3::new(
                half_width,
                top + CharacterDimensions::default().body_height_metres,
                half_depth,
            )
            .into(),
        })
    }
}

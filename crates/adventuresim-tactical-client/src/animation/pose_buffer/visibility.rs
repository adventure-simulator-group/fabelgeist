//! Animation visibility follows every active 3D view, including portrait cameras.
use super::*;
use bevy::camera::{
    primitives::{Frustum, Sphere},
    visibility::RenderLayers,
};

pub(super) type AnimationViews<'w, 's> = Query<
    'w,
    's,
    (
        &'static Camera,
        &'static GlobalTransform,
        &'static Frustum,
        Option<&'static RenderLayers>,
    ),
    With<Camera3d>,
>;

pub(super) fn outside_views(
    position: Vec3,
    layers: Option<&RenderLayers>,
    views: &AnimationViews,
) -> bool {
    let layers = layers.cloned().unwrap_or_default();
    let sphere = Sphere {
        center: position.into(),
        radius: pose_tuning().cull_radius_metres,
    };
    let mut has_view = false;
    for (camera, transform, frustum, camera_layers) in views {
        if !camera.is_active {
            continue;
        }
        has_view = true;
        if layers.intersects(&camera_layers.cloned().unwrap_or_default())
            && position.distance_squared(transform.translation())
                <= pose_tuning().cull_distance_metres.powi(2)
            && frustum.intersects_sphere(&sphere, false)
        {
            return false;
        }
    }
    has_view
}

impl PoseBufferRig {
    #[cfg(any(target_family = "wasm", test))]
    pub(crate) fn has_presented_pose(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn strategic_pose_culling_uses_active_views_and_matching_layers() {
        let mut world = World::new();
        world.spawn((
            Camera3d::default(),
            Camera {
                is_active: false,
                ..default()
            },
            GlobalTransform::from_translation(Vec3::splat(1_000.0)),
            Frustum::default(),
        ));
        let portrait = world
            .spawn((
                Camera3d::default(),
                Camera::default(),
                GlobalTransform::IDENTITY,
                Frustum::default(),
                RenderLayers::layer(25),
            ))
            .id();
        let check = |views: AnimationViews| {
            outside_views(Vec3::ZERO, Some(&RenderLayers::layer(25)), &views)
        };
        assert!(!world.run_system_once(check).unwrap());
        world.entity_mut(portrait).insert(RenderLayers::layer(24));
        assert!(world.run_system_once(check).unwrap());
        world.entity_mut(portrait).insert(RenderLayers::layer(25));
        world
            .entity_mut(portrait)
            .insert(GlobalTransform::from_translation(Vec3::splat(1_000.0)));
        assert!(world.run_system_once(check).unwrap());
    }
}

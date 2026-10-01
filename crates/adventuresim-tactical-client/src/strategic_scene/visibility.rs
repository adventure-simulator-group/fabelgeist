//! Enforce the finite actor-camera range for ordinary meshes as well as the GPU city.
use super::views::StrategicCamera;
use bevy::{
    camera::{
        primitives::{Aabb, Frustum},
        visibility::{NoFrustumCulling, VisibilitySystems, VisibleEntities},
    },
    prelude::*,
};
use std::any::TypeId;

pub(super) fn install(app: &mut App) {
    app.add_systems(
        PostUpdate,
        enforce_far_plane.after(VisibilitySystems::CheckVisibility),
    );
}

fn enforce_far_plane(
    mut cameras: Query<(&Camera, &Frustum, &mut VisibleEntities), With<StrategicCamera>>,
    meshes: Query<(&Aabb, &GlobalTransform), Without<NoFrustumCulling>>,
) {
    // Bevy deliberately omits the far plane from ordinary CPU mesh culling.
    // Filter each camera's list independently; shadow caster lists and the
    // aggregate ViewVisibility must remain available to other views.
    for (camera, frustum, mut visible) in &mut cameras {
        if !camera.is_active {
            continue;
        }
        let plane = frustum.half_spaces[5].normal_d();
        visible.get_mut(TypeId::of::<Mesh3d>()).retain(|entity| {
            let Ok((aabb, transform)) = meshes.get(*entity) else {
                return true;
            };
            let world = transform.affine();
            let center = world.transform_point3a(aabb.center);
            let radius = aabb
                .half_extents
                .dot((world.matrix3.transpose() * bevy::math::Vec3A::from(plane.truncate())).abs());
            plane.truncate().dot(center.into()) + plane.w >= -radius
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;

    #[test]
    fn finite_actor_view_retains_crossing_bounds_and_preserves_other_views() {
        let mut app = App::new();
        app.add_systems(Update, enforce_far_plane);
        let entities = [2.0, 5.5, 8.0].map(|distance| {
            app.world_mut()
                .spawn((
                    Aabb::from_min_max(-Vec3::ONE, Vec3::ONE),
                    GlobalTransform::from_translation(Vec3::Z * -distance),
                ))
                .id()
        });
        let mut cameras = Vec::new();
        for far in [5.0, 10.0] {
            let projection = PerspectiveProjection { far, ..default() };
            let frustum = projection.compute_frustum(&GlobalTransform::IDENTITY);
            let mut visible = VisibleEntities::default();
            visible.get_mut(TypeId::of::<Mesh3d>()).extend(entities);
            cameras.push(
                app.world_mut()
                    .spawn((StrategicCamera, Camera::default(), frustum, visible))
                    .id(),
            );
        }
        app.update();
        let list = |camera| {
            app.world()
                .get::<VisibleEntities>(camera)
                .unwrap()
                .get(TypeId::of::<Mesh3d>())
        };
        assert_eq!(list(cameras[0]), &entities[..2]);
        assert_eq!(list(cameras[1]), &entities);
    }
}

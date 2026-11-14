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
        (enforce_far_plane, wait_for_installation)
            .chain()
            .after(VisibilitySystems::CheckVisibility),
    );
}

#[expect(
    clippy::type_complexity,
    reason = "Initial scene and bootstrap camera mesh lists share the installation gate"
)]
fn wait_for_installation(
    view: Option<Res<super::protocol::StrategicView>>,
    installed: Res<super::status::SceneInstallationReady>,
    scene: Option<Res<super::RetainedScene>>,
    mut opened: Local<bool>,
    mut previous_scene: Local<Option<Entity>>,
    mut cameras: Query<
        &mut VisibleEntities,
        (
            Without<crate::presentation::RegionalMapCamera>,
            Or<(
                With<StrategicCamera>,
                With<crate::presentation::TacticalGameplayCamera>,
            )>,
        ),
    >,
) {
    let root = scene.as_ref().and_then(|scene| scene.root);
    if *previous_scene != root {
        *previous_scene = root;
        *opened = false;
    } else {
        *opened |= installed.0;
    }
    if *opened || view.is_none() {
        return;
    }
    // Keep atmosphere baking, light visibility, uploads, and pose initialization
    // running. Only postpone mesh specialization until the final environment is
    // installed. Reopen for each city; weather changes within an installed city
    // never close this gate. The first frame of a replacement ignores the old
    // city's readiness, which is published at the end of the previous frame.
    for mut visible in &mut cameras {
        visible.get_mut(TypeId::of::<Mesh3d>()).clear();
    }
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
    fn initial_mesh_gate_preserves_other_views_and_never_recloses() {
        let mut app = App::new();
        app.init_resource::<super::super::status::SceneInstallationReady>()
            .add_systems(Update, wait_for_installation);
        let mesh = app.world_mut().spawn_empty().id();
        let cameras = [
            app.world_mut().spawn(StrategicCamera).id(),
            app.world_mut()
                .spawn(crate::presentation::TacticalGameplayCamera)
                .id(),
            app.world_mut()
                .spawn((StrategicCamera, crate::presentation::RegionalMapCamera))
                .id(),
        ];
        let populate = |app: &mut App| {
            for camera in cameras {
                let mut visible = VisibleEntities::default();
                visible.get_mut(TypeId::of::<Mesh3d>()).push(mesh);
                visible.get_mut(TypeId::of::<PointLight>()).push(mesh);
                app.world_mut().entity_mut(camera).insert(visible);
            }
        };
        let meshes = |app: &App| {
            cameras.map(|camera| {
                let visible = app.world().get::<VisibleEntities>(camera).unwrap();
                assert_eq!(visible.get(TypeId::of::<PointLight>()), &[mesh]);
                visible.get(TypeId::of::<Mesh3d>()).len()
            })
        };
        populate(&mut app);
        app.update();
        assert_eq!(meshes(&app), [1, 1, 1], "tactical mode is unaffected");
        app.insert_resource(super::super::protocol::StrategicView {
            revision: 1,
            location: "town".into(),
            places: vec![],
            people: vec![],
            active_place: None,
            selected: None,
            street: None,
            stage: None,
            forge: None,
            portraits: vec![],
        });
        app.update();
        assert_eq!(meshes(&app), [0, 0, 1], "only scene meshes wait");
        populate(&mut app);
        app.world_mut()
            .resource_mut::<super::super::status::SceneInstallationReady>()
            .0 = true;
        app.update();
        assert_eq!(meshes(&app), [1, 1, 1]);
        app.world_mut()
            .resource_mut::<super::super::status::SceneInstallationReady>()
            .0 = false;
        app.update();
        assert_eq!(meshes(&app), [1, 1, 1], "later rebakes cannot blank views");
        // A city replacement must not inherit the old city's ready latch.
        let root = app.world_mut().spawn_empty().id();
        app.insert_resource(super::super::RetainedScene {
            root: Some(root),
            ..default()
        });
        app.world_mut()
            .resource_mut::<super::super::status::SceneInstallationReady>()
            .0 = true;
        app.update();
        assert_eq!(
            meshes(&app),
            [0, 0, 1],
            "ignore stale readiness on replacement"
        );
        populate(&mut app);
        app.world_mut()
            .resource_mut::<super::super::status::SceneInstallationReady>()
            .0 = false;
        app.update();
        assert_eq!(meshes(&app), [0, 0, 1]);
        populate(&mut app);
        app.world_mut()
            .resource_mut::<super::super::status::SceneInstallationReady>()
            .0 = true;
        app.update();
        assert_eq!(meshes(&app), [1, 1, 1], "destination opens when installed");
    }

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

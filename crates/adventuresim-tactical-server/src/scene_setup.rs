use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

pub(crate) struct SceneGeometryPlugin;

impl Plugin for SceneGeometryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(super::terrain_collision::on_scene_terrain_added)
            .add_observer(super::openings::on_scene_building_added)
            .add_observer(super::openings::on_scene_boundary_added)
            .add_observer(super::furniture::on_furniture_added)
            .add_observer(super::furniture::on_group_added)
            .add_observer(super::furniture::on_vista_furniture_added);
    }
}

pub(crate) fn spawn_world_bounds(commands: &mut Commands, width: f32, depth: f32) {
    commands.spawn((
        RigidBody::Static,
        CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
        Transform::default(),
        children![
            (
                Collider::half_space(Vec3::X),
                Transform::from_xyz(-width * 0.5, 0.0, 0.0)
            ),
            (
                Collider::half_space(Vec3::NEG_X),
                Transform::from_xyz(width * 0.5, 0.0, 0.0)
            ),
            (
                Collider::half_space(Vec3::Z),
                Transform::from_xyz(0.0, 0.0, -depth * 0.5)
            ),
            (
                Collider::half_space(Vec3::NEG_Z),
                Transform::from_xyz(0.0, 0.0, depth * 0.5)
            )
        ],
    ));
}

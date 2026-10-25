//! Retained residents outside live views must not be skinned for shadow maps.
use super::{SceneMesh, views::StrategicCamera};
use bevy::{camera::visibility::RenderLayers, light::NotShadowCaster, prelude::*};

#[derive(Component)]
pub(super) struct InactiveCharacterShadow;

#[expect(
    clippy::type_complexity,
    reason = "the query reads render layers and shadow ownership together"
)]
pub(super) fn sync_character_shadows(
    mut commands: Commands,
    cameras: Query<(&Camera, &RenderLayers), With<StrategicCamera>>,
    meshes: Query<
        (
            Entity,
            &RenderLayers,
            Has<NotShadowCaster>,
            Has<InactiveCharacterShadow>,
        ),
        With<SceneMesh>,
    >,
) {
    let active = cameras
        .iter()
        .filter(|(camera, _)| camera.is_active)
        .fold(RenderLayers::none(), |layers, (_, camera)| {
            layers.union(camera)
        });
    for (entity, layers, disabled, owned) in &meshes {
        if layers.intersects(&active) {
            if owned {
                commands
                    .entity(entity)
                    .remove::<(NotShadowCaster, InactiveCharacterShadow)>();
            }
        } else if !disabled {
            commands
                .entity(entity)
                .insert((NotShadowCaster, InactiveCharacterShadow));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_views_restore_only_the_shadow_flags_owned_by_character_culling() {
        let mut app = App::new();
        app.add_systems(Update, sync_character_shadows);
        let stage = app
            .world_mut()
            .spawn((StrategicCamera, Camera::default(), RenderLayers::layer(3)))
            .id();
        let first = app
            .world_mut()
            .spawn((SceneMesh, RenderLayers::layer(3)))
            .id();
        let second = app
            .world_mut()
            .spawn((SceneMesh, RenderLayers::layer(4)))
            .id();
        let authored = app
            .world_mut()
            .spawn((SceneMesh, RenderLayers::layer(4), NotShadowCaster))
            .id();
        app.update();
        assert!(app.world().get::<NotShadowCaster>(first).is_none());
        assert!(app.world().get::<NotShadowCaster>(second).is_some());
        let portrait = app
            .world_mut()
            .spawn((StrategicCamera, Camera::default(), RenderLayers::layer(4)))
            .id();
        app.update();
        assert!(app.world().get::<NotShadowCaster>(second).is_none());
        assert!(app.world().get::<NotShadowCaster>(authored).is_some());
        app.world_mut()
            .get_mut::<Camera>(portrait)
            .unwrap()
            .is_active = false;
        app.update();
        assert!(app.world().get::<NotShadowCaster>(second).is_some());
        app.world_mut()
            .entity_mut(stage)
            .insert(RenderLayers::layer(4));
        app.update();
        assert!(app.world().get::<NotShadowCaster>(first).is_some());
        assert!(app.world().get::<NotShadowCaster>(second).is_none());
        assert!(app.world().get::<NotShadowCaster>(authored).is_some());
    }
}

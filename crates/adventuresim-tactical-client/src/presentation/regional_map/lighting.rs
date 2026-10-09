//! Isolated map lighting from the existing celestial presentation authority.
use super::RegionalMapRoot;
use crate::presentation::sky::{TacticalMoonlight, TacticalSunlight};
use bevy::{camera::visibility::RenderLayers, prelude::*};

#[derive(Component)]
pub(super) struct MapLightSource(Entity);

#[expect(
    clippy::type_complexity,
    reason = "Copy the two authoritative celestial lights into an isolated presentation layer"
)]
pub(super) fn sync(
    mut commands: Commands,
    roots: Query<Entity, With<RegionalMapRoot>>,
    sources: Query<
        (Entity, Ref<DirectionalLight>, Ref<Transform>),
        (
            Or<(With<TacticalSunlight>, With<TacticalMoonlight>)>,
            Without<MapLightSource>,
        ),
    >,
    mut targets: Query<(
        Entity,
        &MapLightSource,
        &mut DirectionalLight,
        &mut Transform,
    )>,
) {
    let Ok(root) = roots.single() else {
        return;
    };
    for (entity, source, _, _) in &targets {
        if sources.get(source.0).is_err() {
            commands.entity(entity).despawn();
        }
    }
    for (source_entity, light, transform) in &sources {
        let existing = targets
            .iter_mut()
            .find(|(_, source, _, _)| source.0 == source_entity);
        if existing.is_some() && !light.is_changed() && !transform.is_changed() {
            continue;
        }
        let mut map_light = *light;
        // The initial terrain view needs no short-range tactical shadow maps.
        map_light.shadow_maps_enabled = false;
        // The sky owner already adapts east/up/north celestial vectors into
        // Bevy east/up/south. Unlike terrain samples, its Z needs no reflection.
        let map_transform = *transform;
        if let Some((_, _, mut light, mut transform)) = existing {
            *light = map_light;
            *transform = map_transform;
        } else {
            commands.spawn((
                Name::new("Regional celestial light"),
                MapLightSource(source_entity),
                map_light,
                map_transform,
                ChildOf(root),
                RenderLayers::layer(crate::strategic_scene::protocol::REGIONAL_MAP_LAYER),
            ));
        }
    }
}

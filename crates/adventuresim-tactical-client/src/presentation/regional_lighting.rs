//! Isolated map lighting from the existing celestial presentation authority.
use crate::presentation::ownership::RegionalMapRoot;
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
        (
            Entity,
            Ref<DirectionalLight>,
            Ref<Transform>,
            Option<&TacticalSunlight>,
        ),
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
    for (source_entity, light, transform, sunlight) in &sources {
        let existing = targets
            .iter_mut()
            .find(|(_, source, _, _)| source.0 == source_entity);
        if existing.is_some() && !light.is_changed() && !transform.is_changed() {
            continue;
        }
        let mut map_light = *light;
        // The source supplies top-of-atmosphere sunlight: its PBR atmosphere
        // normally applies planetary horizon occlusion. Map views omit that
        // atmospheric pass, so never expose raw sunlight below their shared
        // local horizon. Transform::back is the direction towards the light.
        if sunlight.is_some() && transform.back().y <= 0.0 {
            map_light.illuminance = 0.0;
        }
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
                RenderLayers::layer(crate::presentation::ownership::REGIONAL_MAP_LAYER),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_world_schema::{
        calendar::StrategicMinute,
        coordinates::{LatitudeMicrodegrees, LongitudeMicrodegrees},
    };

    #[test]
    fn map_light_tracks_source_daylight_without_underground_sunlight_at_night() {
        let mut app = App::new();
        app.add_systems(Update, sync);
        app.world_mut().spawn(RegionalMapRoot);
        let source = app
            .world_mut()
            .spawn((
                TacticalSunlight,
                DirectionalLight {
                    illuminance: 100_000.0,
                    shadow_maps_enabled: true,
                    ..default()
                },
                Transform::from_translation(Vec3::new(1.0, 1.0, 0.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
            ))
            .id();
        app.update();
        let mut targets = app
            .world_mut()
            .query_filtered::<(&DirectionalLight, &Transform), With<MapLightSource>>();
        let (daylight, direction) = targets.single(app.world()).unwrap();
        assert_eq!(daylight.illuminance, 100_000.0);
        assert!(!daylight.shadow_maps_enabled);
        assert_eq!(*direction, *app.world().get::<Transform>(source).unwrap());

        // The canonical browser capture's clock and geographic origin put the
        // sun below the horizon. Use the same celestial producer as the sky,
        // then adapt its east/up/north direction to Bevy east/up/south.
        let dusk = adventuresim_core::celestial::celestial_directions(
            StrategicMinute::new(123_456),
            LatitudeMicrodegrees::from_degrees(50.5).unwrap(),
            LongitudeMicrodegrees::from_degrees(10.5).unwrap(),
        );
        assert!(dusk.sun[1] < 0.0);
        let direction = Vec3::new(dusk.sun[0], dusk.sun[1], -dusk.sun[2]);
        app.world_mut()
            .entity_mut(source)
            .insert(Transform::from_translation(direction).looking_at(Vec3::ZERO, Vec3::Y));
        app.update();
        let (nightlight, _) = targets.single(app.world()).unwrap();
        assert_eq!(nightlight.illuminance, 0.0);
        assert_eq!(
            app.world()
                .get::<DirectionalLight>(source)
                .unwrap()
                .illuminance,
            100_000.0,
            "the actor's atmospheric lighting remains owned by the sky"
        );

        app.world_mut().entity_mut(source).despawn();
        app.update();
        assert_eq!(targets.iter(app.world()).count(), 0);
    }
}

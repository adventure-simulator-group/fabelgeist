//! Multiple clipped camera views, all targeting the single game canvas.
use super::{
    RetainedScene,
    buildings::ROOM_LAYER,
    protocol::{CanvasRect, StrategicView},
};
use bevy::{camera::visibility::RenderLayers, prelude::*};

#[derive(Resource, Default)]
pub(super) struct ViewCameras(Vec<Entity>);

#[derive(Component)]
pub(super) struct StrategicCamera;

const HEAD_HEIGHT_METRES: f32 = 1.65;
const PORTRAIT_DISTANCE_METRES: f32 = 0.30;
// Actor views retain nearby room/street context. The header alone needs the
// distant skyline; drawing that entire city behind an interior wastes GPU work.
const ACTOR_VIEW_DISTANCE_METRES: f32 = 128.0;
use super::staging::CONVERSATION_DISTANCE_METRES;

pub(super) struct ViewSpec {
    pub cache: Option<super::cached_views::ViewKey>,
    pub rect: CanvasRect,
    pub transform: Transform,
    pub projection: Projection,
    pub layers: RenderLayers,
}

fn person_view(
    rect: CanvasRect,
    anchor: Vec3,
    facing: Quat,
    portrait: bool,
    layers: RenderLayers,
) -> ViewSpec {
    let target = anchor + Vec3::Y * if portrait { HEAD_HEIGHT_METRES } else { 1.2 };
    let distance = if portrait {
        PORTRAIT_DISTANCE_METRES
    } else {
        CONVERSATION_DISTANCE_METRES
    };
    ViewSpec {
        cache: None,
        rect,
        transform: Transform::from_translation(target + facing * Vec3::Z * distance)
            .looking_at(target, Vec3::Y),
        projection: PerspectiveProjection {
            fov: crate::presentation::TacticalCameraSetup::default()
                .vertical_fov_degrees
                .to_radians(),
            near: 0.05,
            far: ACTOR_VIEW_DISTANCE_METRES,
            ..default()
        }
        .into(),
        layers,
    }
}

pub(super) fn sync_views(
    mut commands: Commands,
    requested: Option<Res<StrategicView>>,
    scene: Res<RetainedScene>,
    mut retained: ResMut<ViewCameras>,
    mut cache: super::cached_views::CacheWorld,
    windows: Query<&Window>,
    mut cameras: Query<
        (
            &mut Camera,
            &mut Transform,
            &mut Projection,
            &mut RenderLayers,
        ),
        With<StrategicCamera>,
    >,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    if requested.as_ref().is_some_and(|view| !view.is_changed()) && !scene.is_changed() {
        return;
    }
    let size = window.resolution.physical_size();
    let specs = view_specs(requested.as_deref(), &scene);
    cache.sync(&mut commands, &specs, &scene, requested.as_deref(), window);
    let live = specs
        .iter()
        .filter(|spec| spec.cache.is_none())
        .collect::<Vec<_>>();
    for (index, spec) in live.iter().enumerate() {
        if let Some(entity) = retained.0.get(index)
            && let Ok((mut camera, mut transform, mut projection, mut layers)) =
                cameras.get_mut(*entity)
        {
            spec.rect.apply(&mut camera, size);
            *transform = spec.transform;
            *projection = spec.projection.clone();
            *layers = spec.layers.clone();
        } else {
            let mut camera = Camera {
                order: index as isize + 1,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.13, 0.16, 0.19)),
                ..default()
            };
            spec.rect.apply(&mut camera, size);
            retained.0.push(
                commands
                    .spawn((
                        StrategicCamera,
                        crate::presentation::interior_lighting::FixedViewExposure,
                        Camera3d::default(),
                        camera,
                        spec.transform,
                        spec.projection.clone(),
                        spec.layers.clone(),
                        Msaa::Off,
                        bevy::camera::Exposure::SUNLIGHT,
                    ))
                    .id(),
            );
        }
    }
    for entity in retained.0.iter().skip(live.len()) {
        if let Ok((mut camera, ..)) = cameras.get_mut(*entity) {
            camera.is_active = false;
        }
    }
}

fn view_specs(view: Option<&StrategicView>, scene: &RetainedScene) -> Vec<ViewSpec> {
    let mut specs = Vec::new();
    if let Some(view) = view {
        if let Some(rect) = view.street
            && let Some(street) = &scene.street
        {
            specs.push(ViewSpec {
                cache: Some(super::cached_views::ViewKey::Street),
                rect,
                transform: street.camera(),
                projection: PerspectiveProjection {
                    near: 0.1,
                    fov: street.vertical_fov(),
                    far: 60_000.0,
                    ..default()
                }
                .into(),
                layers: RenderLayers::default(),
            });
        }
        if let Some(rect) = view.stage {
            let subject = view.selected.or_else(|| {
                view.people
                    .iter()
                    .find(|person| {
                        Some(&person.place) == view.active_place.as_ref()
                            && person.presentation == super::protocol::PersonPresentation::Scene
                    })
                    .map(|person| person.id)
            });
            let anchor = subject
                .and_then(|id| scene.people.get(&id).map(|person| person.anchor))
                .or_else(|| {
                    view.active_place
                        .as_ref()
                        .and_then(|id| scene.venues.get(id))
                        .map(|v| v.anchor)
                });
            if let Some(anchor) = anchor {
                let layers = focused_layers(scene, subject);
                let facing = subject
                    .and_then(|id| scene.people.get(&id))
                    .map_or(Quat::IDENTITY, |person| person.facing);
                specs.push(person_view(rect, anchor, facing, false, layers));
            }
        }
        let mut portrait_slots = std::collections::HashMap::new();
        for portrait in &view.portraits {
            if let Some(person) = scene.people.get(&portrait.id) {
                let mut spec = person_view(
                    portrait.rect,
                    person.anchor,
                    person.facing,
                    true,
                    focused_layers(scene, Some(portrait.id)),
                );
                let slot = portrait_slots.entry(portrait.id).or_insert(0);
                spec.cache = Some(super::cached_views::ViewKey::Portrait {
                    person: portrait.id,
                    slot: *slot,
                });
                *slot += 1;
                specs.push(spec);
            }
        }
    }
    specs
}

fn focused_layers(
    scene: &RetainedScene,
    selected: Option<super::protocol::PortraitId>,
) -> RenderLayers {
    let mut layers = RenderLayers::layer(ROOM_LAYER);
    for (id, person) in &scene.people {
        if selected.is_none_or(|selected| selected == *id) {
            layers = layers.with(person.layer);
        }
    }
    layers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic_scene::{RetainedPerson, protocol::PortraitId};
    #[test]
    fn one_character_can_appear_in_two_portrait_slots() {
        let id = PortraitId(1);
        let scene = RetainedScene {
            people: [(
                id,
                RetainedPerson {
                    entity: Entity::PLACEHOLDER,
                    anchor: Vec3::ZERO,
                    facing: Quat::IDENTITY,
                    layer: 2,
                },
            )]
            .into(),
            ..default()
        };
        let rect = CanvasRect {
            x: 0,
            y: 0,
            width: 64,
            height: 64,
            full_width: 64,
            full_height: 64,
            offset_x: 0.0,
            offset_y: 0.0,
        };
        let view = StrategicView {
            revision: 1,
            location: "city".into(),
            places: vec![],
            people: vec![],
            active_place: None,
            selected: None,
            street: None,
            stage: None,
            forge: None,
            portraits: vec![
                super::super::protocol::PortraitView { id, rect },
                super::super::protocol::PortraitView {
                    id,
                    rect: CanvasRect { x: 100, ..rect },
                },
            ],
        };
        let specs = view_specs(Some(&view), &scene);
        assert_eq!(specs.len(), 2);
        assert_ne!(specs[0].cache, specs[1].cache);
        assert_eq!(specs[0].transform, specs[1].transform);
    }
    #[test]
    fn every_building_shares_one_street_camera() {
        let street = super::super::street::Street {
            bays: (0..11)
                .map(|id| super::super::street::StreetBay {
                    id: id.to_string(),
                    width: 20.0,
                })
                .collect(),
            width: 220.0,
            height: 40.0,
            front: Vec3::ZERO,
        };
        let scene = RetainedScene {
            street: Some(street),
            ..default()
        };
        let view = StrategicView {
            revision: 1,
            location: "city".into(),
            places: vec![],
            people: vec![],
            active_place: None,
            selected: None,
            street: Some(CanvasRect {
                x: 0,
                y: 0,
                width: 1000,
                height: 200,
                full_width: 1100,
                full_height: 200,
                offset_x: 50.0,
                offset_y: 0.0,
            }),
            stage: None,
            forge: None,
            portraits: vec![],
        };
        let specs = view_specs(Some(&view), &scene);
        assert_eq!(specs.len(), 1);
        assert_eq!(
            specs[0].cache,
            Some(super::super::cached_views::ViewKey::Street)
        );
        assert!(matches!(specs[0].projection, Projection::Perspective(_)));
    }
    #[test]
    fn focused_views_exclude_other_characters_but_keep_the_room() {
        let scene = RetainedScene {
            people: [
                (
                    PortraitId(1),
                    RetainedPerson {
                        entity: Entity::PLACEHOLDER,
                        anchor: Vec3::ZERO,
                        facing: Quat::IDENTITY,
                        layer: ROOM_LAYER + 1,
                    },
                ),
                (
                    PortraitId(2),
                    RetainedPerson {
                        entity: Entity::PLACEHOLDER,
                        anchor: Vec3::Z,
                        facing: Quat::IDENTITY,
                        layer: ROOM_LAYER + 2,
                    },
                ),
            ]
            .into(),
            ..default()
        };
        let focus = focused_layers(&scene, Some(PortraitId(1)));
        assert!(focus.intersects(&RenderLayers::layer(ROOM_LAYER)));
        assert!(focus.intersects(&RenderLayers::layer(ROOM_LAYER + 1)));
        assert!(!focus.intersects(&RenderLayers::layer(ROOM_LAYER + 2)));
        assert!(focused_layers(&scene, None).intersects(&RenderLayers::layer(ROOM_LAYER + 2)));
    }
}

/// Copy the tactical camera's actual sky, exposure and post-processing state.
/// The gameplay marker remains unique for input and environment bake systems.
#[expect(
    clippy::type_complexity,
    reason = "Camera appearance is inherited from the single tactical camera"
)]
pub(super) fn sync_environment(
    mut commands: Commands,
    source: Query<
        (
            Ref<bevy::camera::Exposure>,
            Ref<bevy::core_pipeline::tonemapping::Tonemapping>,
            Ref<bevy::pbr::DistanceFog>,
            Ref<Msaa>,
            Option<Ref<EnvironmentMapLight>>,
            Option<Ref<bevy::pbr::AtmosphereSettings>>,
        ),
        With<crate::presentation::TacticalGameplayCamera>,
    >,
    targets: Query<(Entity, Ref<StrategicCamera>)>,
) {
    let Ok((exposure, tone, fog, msaa, environment, atmosphere)) = source.single() else {
        return;
    };
    let changed = exposure.is_changed()
        || tone.is_changed()
        || fog.is_changed()
        || msaa.is_changed()
        || environment.as_ref().is_some_and(|value| value.is_changed())
        || atmosphere.as_ref().is_some_and(|value| value.is_changed());
    for (entity, marker) in &targets {
        if !changed && !marker.is_added() {
            continue;
        }
        let mut target = commands.entity(entity);
        target.insert((*exposure, *tone, (*fog).clone(), *msaa));
        if let Some(environment) = &environment {
            target.insert((**environment).clone());
        }
        if let Some(atmosphere) = &atmosphere {
            target.insert((**atmosphere).clone());
        }
    }
}

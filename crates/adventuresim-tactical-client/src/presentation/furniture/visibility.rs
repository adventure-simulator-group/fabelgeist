//! Interior fixtures remain visible while any scene camera can see owner detail.
use std::collections::BTreeMap;

use adventuresim_tactical_core::prelude::*;
use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::presentation::{TacticalCloudOffscreenCamera, buildings};

/// Explicit standalone model exhibition; gameplay always requires a building owner.
#[derive(Resource)]
pub(crate) struct InteriorFurnitureExhibition;

type SceneCameras<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static Camera,
        &'static GlobalTransform,
        Option<&'static RenderLayers>,
    ),
    (With<Camera3d>, Without<TacticalCloudOffscreenCamera>),
>;

pub(super) fn update_interior_visibility(
    exhibition: Option<Res<InteriorFurnitureExhibition>>,
    cameras: SceneCameras,
    buildings: Query<(&SceneBuilding, &GlobalTransform)>,
    mut furniture: Query<(&SceneFurniture, &mut Visibility)>,
) {
    let scene_layers = RenderLayers::default();
    let cameras: Vec<_> = cameras
        .iter()
        .filter(|(camera, _, layers)| {
            camera.is_active && layers.is_none_or(|layers| layers.intersects(&scene_layers))
        })
        .map(|(_, transform, _)| transform.translation())
        .collect();
    // Every mesh of one building uses this root as its VisibilityRange origin.
    // Retain all fixtures through the detail fade, then hide them together.
    let visible_owners: BTreeMap<_, _> = buildings
        .iter()
        .map(|(building, transform)| {
            let visible = cameras.iter().any(|camera| {
                camera.distance(transform.translation()) < buildings::DETAIL_LOD_END_END_METRES
            });
            (building.id, visible)
        })
        .collect();
    for (instance, mut visibility) in &mut furniture {
        let FurnitureLocation::Interior { building_id, .. } = instance.location else {
            continue;
        };
        let next =
            if exhibition.is_some() || visible_owners.get(&building_id).copied().unwrap_or(false) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        if *visibility != next {
            *visibility = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use adventuresim_building_generator::{
        BuildingArchetype, BuildingProgram,
        furniture::{FurnitureKey, FurnitureKind, FurnitureVariant},
    };

    use super::*;

    fn owner(app: &mut App, id: u64) {
        app.world_mut().spawn((
            SceneBuilding {
                id,
                program: BuildingProgram::fixture(BuildingArchetype::FachwerkMerchantHouse, 47),
                orientation: BuildingOrientation::from_radians(0.0).unwrap(),
            },
            GlobalTransform::IDENTITY,
        ));
    }

    fn fixture(app: &mut App, id: u64, x: f32) -> Entity {
        app.world_mut()
            .spawn((
                SceneFurniture {
                    id: FurnitureInstanceId(id),
                    key: FurnitureKey {
                        kind: FurnitureKind::Bed,
                        variant: FurnitureVariant::Compact,
                    },
                    location: FurnitureLocation::Interior {
                        building_id: 1,
                        room_id: 2,
                        storey: 1,
                    },
                },
                GlobalTransform::from_xyz(x, 0.0, 0.0),
                Visibility::Hidden,
            ))
            .id()
    }

    fn camera(app: &mut App, x: f32) -> Entity {
        app.world_mut()
            .spawn((Camera3d::default(), GlobalTransform::from_xyz(x, 0.0, 0.0)))
            .id()
    }

    #[test]
    fn opposite_ends_of_a_building_follow_owner_detail_together() {
        let mut app = App::new();
        app.add_systems(Update, update_interior_visibility);
        owner(&mut app, 1);
        let near = fixture(&mut app, 1, 40.0);
        let far = fixture(&mut app, 2, -40.0);
        let primary_camera = camera(&mut app, 60.0);
        app.update();
        for entity in [near, far] {
            assert_eq!(
                app.world().get::<Visibility>(entity),
                Some(&Visibility::Inherited)
            );
        }
        // The near fixture remains only 31m away, but its owner's detail is gone.
        app.world_mut()
            .entity_mut(primary_camera)
            .insert(GlobalTransform::from_xyz(71.0, 0.0, 0.0));
        app.update();
        for entity in [near, far] {
            assert_eq!(
                app.world().get::<Visibility>(entity),
                Some(&Visibility::Hidden)
            );
        }
        // A second relevant camera retains the complete interior for both views.
        camera(&mut app, 0.0);
        app.update();
        for entity in [near, far] {
            assert_eq!(
                app.world().get::<Visibility>(entity),
                Some(&Visibility::Inherited)
            );
        }
    }

    #[test]
    fn missing_owner_requires_explicit_exhibition() {
        let mut app = App::new();
        app.add_systems(Update, update_interior_visibility);
        let fixture = fixture(&mut app, 1, 0.0);
        camera(&mut app, 0.0);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(fixture),
            Some(&Visibility::Hidden)
        );
        owner(&mut app, 1);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(fixture),
            Some(&Visibility::Inherited)
        );
        let mut specimen = *app.world().get::<SceneFurniture>(fixture).unwrap();
        specimen.location = FurnitureLocation::Interior {
            building_id: 0,
            room_id: 0,
            storey: 0,
        };
        app.world_mut().entity_mut(fixture).insert(specimen);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(fixture),
            Some(&Visibility::Hidden)
        );
        app.insert_resource(InteriorFurnitureExhibition);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(fixture),
            Some(&Visibility::Inherited)
        );
    }

    #[test]
    fn inactive_and_offscreen_cameras_do_not_keep_interiors_visible() {
        let mut app = App::new();
        app.add_systems(Update, update_interior_visibility);
        owner(&mut app, 1);
        let fixture = fixture(&mut app, 1, 0.0);
        let inactive = camera(&mut app, 0.0);
        app.world_mut()
            .get_mut::<Camera>(inactive)
            .unwrap()
            .is_active = false;
        let other_layer = camera(&mut app, 0.0);
        app.world_mut()
            .entity_mut(other_layer)
            .insert(RenderLayers::layer(3));
        let cloud = camera(&mut app, 0.0);
        app.world_mut()
            .entity_mut(cloud)
            .insert(TacticalCloudOffscreenCamera);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(fixture),
            Some(&Visibility::Hidden)
        );
    }
}

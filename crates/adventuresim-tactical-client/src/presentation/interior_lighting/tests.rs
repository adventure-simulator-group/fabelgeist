use super::*;
use adventuresim_building_generator::{
    BuildingArchetype, BuildingProgram, Cell, Direction, Opening, OpeningKind, Room, RoomKind,
    StoreyPlan, WallSegment, generate,
};

fn test_field() -> InteriorField {
    let mut plan = generate(&BuildingProgram::fixture(
        BuildingArchetype::FachwerkCottage,
        7,
    ))
    .unwrap();
    plan.storey_height_metres = 3.0;
    plan.storeys = vec![StoreyPlan {
        level: 0,
        rooms: vec![
            Room {
                id: 0,
                kind: RoomKind::CommonRoom,
                cells: vec![Cell::new(0, 0), Cell::new(1, 0), Cell::new(2, 0)],
            },
            Room {
                id: 1,
                kind: RoomKind::Storage,
                cells: vec![Cell::new(0, 1)],
            },
        ],
        walls: vec![WallSegment {
            cell: Cell::new(0, 0),
            direction: Direction::West,
            inside_room: 0,
            outside_room: None,
        }],
        openings: vec![Opening {
            wall: 0,
            kind: OpeningKind::Window,
            width_metres: 1.0,
            sill_metres: 1.0,
            height_metres: 1.0,
        }],
    }];
    InteriorField::from_plan(&plan, Vec3::new(2.0, 2.0, 2.0))
}

#[test]
fn daylight_is_directional_and_falls_off_into_the_room() {
    let field = test_field();
    let near = field
        .sample(Vec3::new(0.75, 1.5, 0.75) - Vec3::splat(2.0))
        .unwrap();
    let far = field
        .sample(Vec3::new(3.75, 1.5, 0.75) - Vec3::splat(2.0))
        .unwrap();
    assert!(near.negative.x > near.positive.x);
    assert!(near.negative.x > far.negative.x);
    assert!(near.negative.y > 0.0, "ceilings receive diffuse bounce");
}

#[test]
fn daylight_stays_out_of_adjacent_rooms_courtyards_and_other_floors() {
    let field = test_field();
    let offset = Vec3::splat(2.0);
    let storage = field.sample(Vec3::new(0.75, 1.5, 2.25) - offset).unwrap();
    assert_eq!(storage.positive.truncate(), Vec3::ZERO);
    assert_eq!(storage.negative.truncate(), Vec3::ZERO);
    assert!(field.sample(Vec3::new(2.25, 1.5, 2.25) - offset).is_none());
    assert!(field.sample(Vec3::new(0.75, 3.1, 0.75) - offset).is_none());
    assert!(field.sample(Vec3::new(-0.1, 1.5, 0.75) - offset).is_none());
}

#[test]
fn rotated_building_uses_the_same_room_coordinates() {
    let field = test_field();
    let transform = Transform::from_xyz(40.0, 8.0, -12.0)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    let local = Vec3::new(0.75, 1.5, 0.75) - Vec3::splat(2.0);
    let world = transform.transform_point(local);
    let recovered = transform.compute_affine().inverse().transform_point3(world);
    assert_eq!(field.sample(local), field.sample(recovered));
}

#[test]
fn resident_fields_are_bounded_and_removed_buildings_leave_no_light() {
    let mut app = App::new();
    app.init_resource::<Assets<ShaderBuffer>>()
        .init_resource::<PresentedCelestialLighting>()
        .add_systems(Update, upload_field);
    let data = GpuField::default();
    let buffer = app
        .world_mut()
        .resource_mut::<Assets<ShaderBuffer>>()
        .add(ShaderBuffer::from(data.clone()));
    app.insert_resource(InteriorLightingGpu {
        buffer,
        data,
        selection: Vec::new(),
        _shader: Handle::default(),
    });
    app.world_mut()
        .spawn((TacticalGameplayCamera, GlobalTransform::default()));
    let field = test_field();
    let mut buildings = Vec::new();
    for index in 0..20 {
        buildings.push(
            app.world_mut()
                .spawn((
                    field.clone(),
                    GlobalTransform::from_translation(Vec3::X * index as f32 * 3.0),
                ))
                .id(),
        );
    }
    app.update();
    let count = app.world().resource::<InteriorLightingGpu>().data.counts.x;
    assert!(count > 0 && count < buildings.len() as u32);
    for building in buildings {
        app.world_mut().despawn(building);
    }
    app.update();
    let gpu = app.world().resource::<InteriorLightingGpu>();
    assert_eq!(gpu.data.counts.x, 0);
    assert!(gpu.selection.is_empty());
}

#[test]
fn exposure_adapts_during_pause_recovers_outdoors_and_preserves_night() {
    use crate::presentation::{ActiveTacticalScene, update_presented_celestial_lighting};
    use adventuresim_tactical_core::prelude::SceneEnvironmentFixture;
    use bevy::camera::Exposure;
    use std::time::Duration;

    let mut app = App::new();
    app.init_resource::<Time<Real>>()
        .init_resource::<Time<Virtual>>()
        .init_resource::<ActiveTacticalScene>()
        .init_resource::<PresentedCelestialLighting>()
        .init_resource::<exposure::InteriorExposure>()
        .add_systems(
            Update,
            (
                update_presented_celestial_lighting,
                exposure::adapt_exposure,
            )
                .chain(),
        );
    app.world_mut().resource_mut::<Time<Virtual>>().pause();
    let mut environment =
        SceneEnvironmentFixture::TemperateHills.snapshot("interior-exposure-test");
    environment.absolute_minute = 340_440;
    let scene = app.world_mut().spawn(environment.clone()).id();
    app.world_mut().resource_mut::<ActiveTacticalScene>().entity = Some(scene);
    app.world_mut()
        .spawn((test_field(), GlobalTransform::default()));
    let camera = app
        .world_mut()
        .spawn((
            TacticalGameplayCamera,
            GlobalTransform::from_translation(Vec3::new(0.75, 1.5, 2.25) - Vec3::splat(2.0)),
            Exposure::SUNLIGHT,
        ))
        .id();
    let step = |app: &mut App| {
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
    };
    for _ in 0..600 {
        step(&mut app);
    }
    let baseline = app
        .world()
        .resource::<PresentedCelestialLighting>()
        .snapshot
        .as_ref()
        .unwrap()
        .exposure_ev100;
    assert!(app.world().get::<Exposure>(camera).unwrap().ev100 < baseline - 1.0);
    assert_eq!(app.world().resource::<Time<Virtual>>().elapsed_secs(), 0.0);
    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from_translation(Vec3::splat(100.0)));
    for _ in 0..300 {
        step(&mut app);
    }
    assert_eq!(app.world().get::<Exposure>(camera).unwrap().ev100, baseline);
    environment.absolute_minute += 12 * 60;
    app.world_mut().entity_mut(scene).insert(environment);
    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from_translation(
            Vec3::new(0.75, 1.5, 2.25) - Vec3::splat(2.0),
        ));
    for _ in 0..300 {
        step(&mut app);
    }
    let celestial = app.world().resource::<PresentedCelestialLighting>();
    assert_eq!(daylight_response(celestial).w, 0.0);
    assert_eq!(
        app.world().get::<Exposure>(camera).unwrap().ev100,
        celestial.snapshot.as_ref().unwrap().exposure_ev100
    );
    app.world_mut().resource_mut::<ActiveTacticalScene>().entity = None;
    step(&mut app);
    assert_eq!(
        app.world().get::<Exposure>(camera).unwrap().ev100,
        Exposure::SUNLIGHT.ev100
    );
}

#[test]
fn standard_material_edits_reach_the_interior_renderer_without_duplicate_draws() {
    use bevy::asset::AssetPlugin;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<StandardMaterial>()
        .init_asset::<InteriorMaterial>()
        .init_resource::<material::InteriorMaterials>()
        .insert_resource(InteriorLightingGpu {
            buffer: Handle::default(),
            data: GpuField::default(),
            selection: Vec::new(),
            _shader: Handle::default(),
        })
        .add_systems(Update, material::prepare_materials);
    let source = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });
    let entity = app.world_mut().spawn(MeshMaterial3d(source.clone())).id();
    app.update();
    let rendered = app
        .world()
        .get::<MeshMaterial3d<InteriorMaterial>>(entity)
        .unwrap()
        .0
        .clone();
    assert!(
        app.world()
            .get::<MeshMaterial3d<StandardMaterial>>(entity)
            .is_none()
    );
    assert_eq!(
        app.world().get::<InteriorMaterialSource>(entity).unwrap().0,
        source
    );
    app.world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .get_mut(&source)
        .unwrap()
        .base_color = Color::BLACK;
    app.update();
    app.update();
    assert_eq!(
        app.world()
            .resource::<Assets<InteriorMaterial>>()
            .get(&rendered)
            .unwrap()
            .base
            .base_color,
        Color::BLACK
    );
}

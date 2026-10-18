use super::*;
use protocol::{Person, Place, PlaceKind};

#[test]
fn layers_are_revisited_when_an_existing_mesh_is_attached_to_a_model() {
    let mut app = App::new();
    app.add_systems(Update, inherit_model_layers.run_if(model_hierarchy_changed));
    let model = app.world_mut().spawn(SceneModel(3)).id();
    let mesh = app.world_mut().spawn(Mesh3d(Handle::default())).id();
    app.update();
    app.update();
    assert!(app.world().get::<RenderLayers>(mesh).is_none());
    app.world_mut().entity_mut(mesh).insert(ChildOf(model));
    app.update();
    assert_eq!(
        app.world().get::<RenderLayers>(mesh),
        Some(&RenderLayers::layer(3))
    );
    app.update();
    assert!(app.world().get::<SceneMesh>(mesh).is_some());
}

#[test]
fn strategic_equipment_gets_bounds_even_when_tactical_meshes_disable_automatic_bounds() {
    use bevy::camera::{primitives::Aabb, visibility::NoFrustumCulling};
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .add_systems(Update, cull_equipment);
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::default());
    let entity = app
        .world_mut()
        .spawn((SceneMesh, Mesh3d(mesh), NoFrustumCulling))
        .id();
    app.update();
    assert!(app.world().get::<NoFrustumCulling>(entity).is_none());
    assert!(
        app.world()
            .get::<Aabb>(entity)
            .unwrap()
            .half_extents
            .min_element()
            > 0.5
    );
}

#[test]
fn procedural_carry_recipes_survive_the_browser_appearance_boundary() {
    use adventuresim_core::equipment_presentation::GeneratedAppearance;
    use adventuresim_weapon_model as weapon;
    let design = weapon::default_design("longsword").unwrap();
    let holder = weapon::default_holder_design(&design).unwrap();
    let appearance = GeneratedAppearance {
        generator_version: weapon::GENERATOR_VERSION,
        design_hash: weapon::design_hash(&design).0,
        recipe: weapon::encode(&design).unwrap(),
    };
    let fitted_holder = GeneratedAppearance {
        generator_version: weapon::HOLDER_GENERATOR_VERSION,
        design_hash: weapon::holder_design_hash(&holder).0,
        recipe: weapon::encode_holder(&holder).unwrap(),
    };
    let encoded = serde_json::to_string(&appearance).unwrap();
    let decoded: GeneratedAppearance = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        weapon::design_hash(&weapon::decode(&decoded.recipe).unwrap()).0,
        appearance.design_hash
    );
    if let Ok(directory) = std::env::var("STRATEGIC_SCENE_FIXTURE_DIR") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(
            directory.join("carry.json"),
            serde_json::to_vec(&serde_json::json!({
                "weapon": appearance, "holder": fitted_holder,
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn strategic_scene_keeps_character_entities_without_tactical_authority() {
    let mut app = App::new();
    let root = app.world_mut().spawn_empty().id();
    let place = PlaceId("inn".into());
    let id = PortraitId(42);
    let rect = protocol::CanvasRect {
        x: 0,
        y: 0,
        width: 64,
        height: 64,
        full_width: 64,
        full_height: 64,
        offset_x: 0.0,
        offset_y: 0.0,
    };
    app.insert_resource(RetainedScene {
        root: Some(root),
        venues: [(
            place.clone(),
            Venue {
                anchor: Vec3::ZERO,
                approach: Vec3::ZERO,
                positions: vec![Transform::default(), Transform::from_translation(Vec3::X)],
            },
        )]
        .into(),
        ..default()
    });
    app.insert_resource(StrategicView {
        revision: 1,
        location: "town".into(),
        places: vec![Place {
            id: place.clone(),
            kind: PlaceKind::Inn,
        }],
        people: vec![
            Person {
                id: PortraitId(7),
                place: place.clone(),
                presentation: protocol::PersonPresentation::Scene,
                equipment: vec![],
            },
            Person {
                id,
                place: place.clone(),
                presentation: protocol::PersonPresentation::Scene,
                equipment: vec![],
            },
        ],
        active_place: Some(place),
        selected: Some(id),
        street: None,
        stage: Some(rect),
        forge: None,
        portraits: vec![],
    });
    let outfit = serde_json::from_value(serde_json::json!({
        "id": "9007199254740993", "item": "belt", "placement": "worn",
        "occupancies": [{"anchor": {"Character": "front_belt"}, "channel": "accessory",
            "order": 0, "requirement_index": 0, "capacity_index": 0}],
        "weapon": null, "holder": null,
    }))
    .unwrap();
    app.world_mut().resource_mut::<StrategicView>().people[1].equipment = vec![outfit];
    app.add_systems(Update, (retain_people, equipment::sync_equipment).chain());
    app.update();
    let entity = app.world().resource::<RetainedScene>().people[&id].entity;
    let layer = app.world().resource::<RetainedScene>().people[&id].layer;
    assert!(
        !app.world()
            .resource::<RetainedScene>()
            .people
            .contains_key(&PortraitId(7))
    );
    assert_eq!(
        app.world().resource::<RetainedScene>().people[&id].anchor,
        Vec3::X,
        "unseen earlier residents still determine the visible resident's rank"
    );
    let item = app
        .world_mut()
        .query_filtered::<Entity, With<equipment::SceneEquipment>>()
        .single(app.world())
        .unwrap();
    assert!(
        app.world()
            .get::<adventuresim_tactical_core::prelude::TacticalEquipmentPhysical>(item)
            .unwrap()
            .is_valid()
    );
    assert_eq!(
        app.world()
            .get::<adventuresim_tactical_core::prelude::ItemOf>(item)
            .unwrap()
            .0,
        entity
    );
    app.world_mut().resource_mut::<StrategicView>().selected = None;
    app.update();
    assert!(
        app.world()
            .resource::<RetainedScene>()
            .people
            .contains_key(&PortraitId(7)),
        "the stage fallback loads its first scene resident"
    );
    app.world_mut().resource_mut::<StrategicView>().selected = Some(id);
    app.update();
    assert_eq!(
        app.world().resource::<RetainedScene>().people[&id].entity,
        entity
    );
    assert_eq!(app.world().get::<CharacterId>(entity).unwrap().0, 42);
    assert_eq!(
        app.world().resource::<RetainedScene>().people[&id].layer,
        layer
    );
    assert!(
        app.world().get_entity(item).is_ok(),
        "navigation retains fitted equipment"
    );
    assert!(
        app.world()
            .get::<adventuresim_tactical_core::prelude::Player>(entity)
            .is_none()
    );
    assert!(
        app.world()
            .get::<adventuresim_tactical_core::prelude::CharacterController>(entity)
            .is_none()
    );
    let place = app.world().resource::<StrategicView>().people[0]
        .place
        .clone();
    app.world_mut()
        .resource_mut::<StrategicView>()
        .people
        .push(Person {
            id: PortraitId(99),
            place,
            presentation: protocol::PersonPresentation::Portrait,
            equipment: vec![],
        });
    app.update();
    assert!(
        !app.world()
            .resource::<RetainedScene>()
            .people
            .contains_key(&PortraitId(99))
    );
    app.world_mut()
        .resource_mut::<StrategicView>()
        .portraits
        .push(protocol::PortraitView {
            id: PortraitId(99),
            rect,
        });
    app.update();
    let scene = app.world().resource::<RetainedScene>();
    assert_eq!(scene.people[&id].anchor, Vec3::X);
    assert_eq!(scene.people[&PortraitId(99)].anchor, Vec3::ZERO);
    assert_ne!(scene.people[&id].layer, scene.people[&PortraitId(99)].layer);
    app.world_mut().resource_mut::<StrategicView>().people[1]
        .equipment
        .clear();
    app.update();
    assert!(
        app.world().get_entity(item).is_err(),
        "unequipping removes its presentation entity"
    );
    // Exceed both native and Wasm inline layer capacity. Each resident must
    // still have an independent camera layer without a constructor panic.
    let place = app.world().resource::<StrategicView>().people[0]
        .place
        .clone();
    app.world_mut()
        .resource_mut::<StrategicView>()
        .people
        .extend((200..300).map(|id| Person {
            id: PortraitId(id),
            place: place.clone(),
            presentation: protocol::PersonPresentation::Scene,
            equipment: vec![],
        }));
    app.world_mut()
        .resource_mut::<StrategicView>()
        .portraits
        .extend((200..300).map(|id| protocol::PortraitView {
            id: PortraitId(id),
            rect,
        }));
    app.update();
    let scene = app.world().resource::<RetainedScene>();
    assert_eq!(scene.people.len(), 103);
    let layers: std::collections::HashSet<_> =
        scene.people.values().map(|person| person.layer).collect();
    assert_eq!(layers.len(), 103);
    {
        let mut view = app.world_mut().resource_mut::<StrategicView>();
        view.stage = None;
        view.portraits.clear();
        view.forge = Some(rect);
        view.selected = Some(PortraitId(4000));
        view.people.push(Person {
            id: PortraitId(4000),
            place,
            presentation: protocol::PersonPresentation::Scene,
            equipment: vec![],
        });
    }
    app.update();
    assert!(
        !app.world()
            .resource::<RetainedScene>()
            .people
            .contains_key(&PortraitId(4000)),
        "forge or map selection alone does not install an unseen character"
    );
    app.world_mut()
        .resource_mut::<StrategicView>()
        .people
        .retain(|person| person.id != id);
    app.update();
    assert!(
        app.world().get_entity(entity).is_err(),
        "a resident removed from the roster no longer retains its presentation"
    );
    {
        let mut view = app.world_mut().resource_mut::<StrategicView>();
        view.stage = Some(rect);
        view.selected = None;
        view.active_place = None;
    }
    app.update();
    assert!(
        app.world()
            .resource::<RetainedScene>()
            .people
            .contains_key(&PortraitId(4000)),
        "an unfocused stage retains its existing all-resident layer policy"
    );
    {
        let mut view = app.world_mut().resource_mut::<StrategicView>();
        view.stage = None;
        view.portraits.push(protocol::PortraitView {
            id: PortraitId(99),
            rect,
        });
        let place = view.people[0].place.clone();
        view.people.push(Person {
            id: PortraitId(5000),
            place,
            presentation: protocol::PersonPresentation::Scene,
            equipment: vec![],
        });
    }
    app.update();
    assert!(
        !app.world()
            .resource::<RetainedScene>()
            .people
            .contains_key(&PortraitId(5000)),
        "a portrait-only page does not install its other unseen roster members"
    );
}

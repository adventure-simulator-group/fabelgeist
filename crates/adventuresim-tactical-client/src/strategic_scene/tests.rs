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
    app.insert_resource(RetainedScene {
        root: Some(root),
        venues: [(
            place.clone(),
            Venue {
                anchor: Vec3::ZERO,
                approach: Vec3::ZERO,
                positions: vec![Transform::default()],
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
        people: vec![Person {
            id,
            place: place.clone(),
            presentation: protocol::PersonPresentation::Scene,
            equipment: vec![],
        }],
        active_place: Some(place),
        selected: Some(id),
        street: None,
        stage: None,
        forge: None,
        portraits: vec![],
    });
    let outfit = serde_json::from_value(serde_json::json!({
        "id": "9007199254740993", "item": "leather_belt", "placement": "worn",
        "occupancies": [{"anchor": {"Character": "front_belt"}, "channel": "accessory",
            "order": 0, "requirement_index": 0, "capacity_index": 0}],
        "weapon": null, "holder": null,
    }))
    .unwrap();
    app.world_mut().resource_mut::<StrategicView>().people[0].equipment = vec![outfit];
    app.add_systems(Update, (retain_people, equipment::sync_equipment).chain());
    app.update();
    let entity = app.world().resource::<RetainedScene>().people[&id].entity;
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
    assert_eq!(
        app.world().resource::<RetainedScene>().people[&id].entity,
        entity
    );
    assert_eq!(app.world().get::<CharacterId>(entity).unwrap().0, 42);
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
    let scene = app.world().resource::<RetainedScene>();
    assert_eq!(scene.people[&id].anchor, Vec3::ZERO);
    assert_eq!(scene.people[&PortraitId(99)].anchor, Vec3::ZERO);
    assert_ne!(scene.people[&id].layer, scene.people[&PortraitId(99)].layer);
    app.world_mut().resource_mut::<StrategicView>().people[0]
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
    app.update();
    let scene = app.world().resource::<RetainedScene>();
    assert_eq!(scene.people.len(), 102);
    let layers: std::collections::HashSet<_> =
        scene.people.values().map(|person| person.layer).collect();
    assert_eq!(layers.len(), 102);
}

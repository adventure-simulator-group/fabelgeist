use super::*;

fn empty_body() -> RuntimeBody {
    RuntimeBody {
        domain: "test".into(),
        positions: vec![],
        normals: vec![],
        faces: vec![],
        texcoords: vec![],
        texcoord_faces: vec![],
        joint_indices: vec![],
        joint_weights: vec![],
        joint_names: vec!["l_lowarm".into()],
        global_joint_states: vec![],
        device: Default::default(),
    }
}

fn ready_cache() -> RuntimeEquipmentBodyCache {
    RuntimeEquipmentBodyCache {
        body: Some(body::CanonicalBody {
            body: empty_body(),
            names: vec![],
            targets: vec![],
            rig: rig::FittingRig::default(),
        }),
        bracer_design: Some(Default::default()),
        breastplate_design: Some(Default::default()),
        ..default()
    }
}

fn character_key(id: u64) -> FitKey {
    FitKey {
        shape: BodyShapeKey::new(
            &[],
            Some(id),
            CharacterProportions::from_seed(fabelgeist_determinism::Seed::from_u64(id)),
            Default::default(),
        ),
        item: "vambrace".into(),
        placement: "left".into(),
        layers: Vec::new(),
    }
}

#[test]
fn physical_fit_survives_hold_drop_and_placeholder_rebuild_then_refits_new_wearer() {
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .insert_resource(RuntimeEquipmentWarmup::ready_for_tests())
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<WeaponMeshCache>()
        .add_systems(
            Update,
            (
                crate::animation::cache_humanoid_rigs,
                spawn_item_placeholders,
                generate_runtime_equipment_models,
                sync_procedural_equipment_skins,
                render_binding::sync_render_bindings,
                update_item_placeholders,
            )
                .chain(),
        );
    let first = actor(app.world_mut(), 1);
    let second = actor(app.world_mut(), 2);
    let mut cache = ready_cache();
    for id in [1, 2] {
        let key = character_key(id);
        cache.bodies.insert(
            key.shape.clone(),
            FittedBody {
                body: Arc::new(empty_body()),
                inverse_bindposes: default(),
            },
        );
        cache.models.insert(
            key,
            Ok(CachedEquipment {
                generated: Arc::new(empty_armor()),
                parts: vec![CachedPart {
                    mesh: app
                        .world_mut()
                        .resource_mut::<Assets<Mesh>>()
                        .add(fixture_mesh()),
                    material: default(),
                }],
                rigid_center: Vec3::new(2.0, 3.0, 0.0),
                sockets: default(),
            }),
        );
    }
    app.insert_resource(cache);
    let item = app
        .world_mut()
        .spawn((
            ItemOf(first),
            Transform::IDENTITY,
            ItemProperties {
                id: "vambrace".into(),
                weight: 1.0,
            },
            worn_topology(),
            TacticalEquipmentPhysical {
                dimensions_m: Vec3::splat(0.2),
                grip_to_tip_m: 0.0,
                striking_head_length_m: 0.0,
                anchor_offset_m: Vec3::ZERO,
            },
        ))
        .id();
    app.update();
    assert_presentation(app.world_mut(), true, false);
    assert_invalid_outfit_recovers(&mut app);
    assert_eq!(
        app.world().get::<LastFit>(item).unwrap().0,
        character_key(1)
    );
    app.world_mut().entity_mut(item).insert((
        EquipSlot::HoldingLeft,
        EquipmentTopology {
            placement_id: Some("left_hand".into()),
            ..default()
        },
    ));
    app.update();
    assert_presentation(app.world_mut(), false, true);
    assert_eq!(
        app.world().get::<LastFit>(item).unwrap().0,
        character_key(1)
    );
    app.world_mut()
        .entity_mut(item)
        .insert((TacticalSceneItem, EquipmentTopology::default()))
        .remove::<(ItemOf, EquipSlot)>();
    app.update();
    assert_presentation(app.world_mut(), false, false);
    assert_eq!(
        app.world().get::<LastFit>(item).unwrap().0,
        character_key(1)
    );
    app.world_mut()
        .entity_mut(item)
        .remove::<TacticalSceneItem>()
        .insert((
            ItemOf(second),
            EquipSlot::HoldingRight,
            EquipmentTopology {
                placement_id: Some("right_hand".into()),
                ..default()
            },
        ));
    app.update();
    assert_presentation(app.world_mut(), false, true);
    assert_eq!(
        app.world().get::<LastFit>(item).unwrap().0,
        character_key(1)
    );
    app.world_mut()
        .entity_mut(item)
        .remove::<EquipSlot>()
        .insert(worn_topology());
    app.update();
    assert_presentation(app.world_mut(), true, false);
    assert_eq!(
        app.world().get::<LastFit>(item).unwrap().0,
        character_key(2)
    );
    assert_eq!(
        app.world_mut()
            .query::<&ProceduralEquipmentPart>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query::<&ProceduralEquipmentFailed>()
            .iter(app.world())
            .count(),
        0
    );
    assert!(
        app.world()
            .resource::<RuntimeEquipmentBodyCache>()
            .pending
            .is_none()
    );
}

fn assert_invalid_outfit_recovers(app: &mut App) {
    let root = app
        .world_mut()
        .query_filtered::<Entity, With<RuntimeEquipmentPresentation>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .get_mut::<RuntimeEquipmentPresentation>(root)
        .unwrap()
        .placement_id = "invalid-placement".into();
    app.update();
    assert!(app.world().get::<ProceduralEquipmentFailed>(root).is_some());
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden)
    );
    app.world_mut()
        .get_mut::<RuntimeEquipmentPresentation>(root)
        .unwrap()
        .placement_id = "left".into();
    app.update();
    assert!(app.world().get::<ProceduralEquipmentFailed>(root).is_none());
    assert_presentation(app.world_mut(), true, false);
}

fn empty_armor() -> GeneratedArmor {
    GeneratedArmor {
        design_hash: [0; 32],
        surface_domain: String::new(),
        positions: vec![],
        normals: vec![],
        texcoords: vec![],
        joint_indices: vec![],
        joint_weights: vec![],
        indices: vec![],
        faces: vec![],
        trim: None,
        grids: vec![],
        morphs: vec![],
        components: vec![],
    }
}

fn actor(world: &mut World, id: u64) -> Entity {
    use crate::animation::{AnimationRigScene, HumanoidBone, StrategicModel};
    let owner = world.spawn((CharacterId(id), StrategicModel)).id();
    world.spawn(AnimationRigScene(owner));
    for role in [
        BoneRole::ForearmLeft,
        BoneRole::WeaponLeft,
        BoneRole::WeaponRight,
    ] {
        world.spawn((
            HumanoidBone { owner, role },
            MhrBone { owner },
            Name::new(if role == BoneRole::ForearmLeft {
                "l_lowarm"
            } else {
                "hand"
            }),
        ));
    }
    owner
}

fn worn_topology() -> EquipmentTopology {
    EquipmentTopology {
        placement_id: Some("left".into()),
        occupancies: vec![EquipmentTopologyOccupancy {
            occupancy_id: "arm".into(),
            anchor: TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::LeftArm),
            channel: EquipmentChannel::Accessory,
            order: 0,
            requirement_index: 0,
            capacity_index: 0,
        }],
    }
}

fn fixture_mesh() -> Mesh {
    use bevy::mesh::VertexAttributeValues;
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    // The unused fourth vertex must not affect the carried geometry's center.
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [1.0, 2.0, 0.0],
            [3.0, 2.0, 0.0],
            [2.0, 4.0, 0.0],
            [100.0; 3],
        ],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[0; 4]; 4]),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[1.0, 0.0, 0.0, 0.0]; 4]);
    mesh
}

fn assert_presentation(world: &mut World, worn: bool, held: bool) {
    let (entity, mesh, visibility) = world
        .query_filtered::<(Entity, &Mesh3d, &Visibility), With<ProceduralEquipmentPart>>()
        .single(world)
        .unwrap();
    assert_eq!(world.get::<SkinnedMesh>(entity).is_some(), worn);
    assert_eq!(*visibility, Visibility::Inherited);
    let positions = world
        .resource::<Assets<Mesh>>()
        .get(&mesh.0)
        .unwrap()
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    assert_eq!(
        positions[0],
        if worn {
            [1.0, 2.0, 0.0]
        } else {
            [-1.0, -1.0, 0.0]
        }
    );
    let (root, visibility) = world
        .query_filtered::<(Entity, &Visibility), With<ItemPlaceholder>>()
        .single(world)
        .unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    assert_eq!(world.get::<HeldWeaponConstraint>(root).is_some(), held);
    assert_eq!(world.get::<ChildOf>(root).is_some(), worn);
}

#[test]
fn cache_eviction_releases_old_bodies_and_keeps_recent_fits() {
    let mut cache = ready_cache();
    for id in 0..=CACHED_FIT_LIMIT as u64 {
        let key = character_key(id);
        cache.last_used.insert(key.clone(), id);
        cache.bodies.insert(
            key.shape.clone(),
            FittedBody {
                body: Arc::new(empty_body()),
                inverse_bindposes: default(),
            },
        );
        cache.models.insert(key, Err("fixture failure".into()));
    }
    cache.trim();
    assert!(!cache.models.contains_key(&character_key(0)));
    assert!(!cache.bodies.contains_key(&character_key(0).shape));
    assert!(
        cache
            .models
            .contains_key(&character_key(CACHED_FIT_LIMIT as u64))
    );
    assert_eq!(cache.models.len(), CACHED_FIT_LIMIT);
}

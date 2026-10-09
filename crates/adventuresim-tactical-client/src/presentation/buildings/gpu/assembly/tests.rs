use super::*;

#[test]
fn components_share_geometry_and_pose_but_follow_their_buildings_lod() {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::new(1.0, 2.0, 3.0));
    let mut parts = Vec::new();
    for x in [0.0, 30.0] {
        let root = world.spawn_empty().id();
        for level in [1, 2] {
            parts.push(Part {
                entity: None,
                root,
                transform: Mat4::from_translation(Vec3::X * x),
                local_transform: if level == 1 {
                    Mat4::from_translation(Vec3::Y * 5.0)
                } else {
                    Mat4::IDENTITY
                },
                uv_offset: Vec2::new(0.5, 0.25),
                mesh: mesh.clone(),
                material: Handle::default(),
                level,
                fade: None,
            });
        }
    }
    let packed = pack(&world, input::Group::from_parts(parts.iter().cloned())).unwrap();
    let owners: Vec<_> = packed
        .buildings
        .iter()
        .filter(|record| record.levels.y != COMPONENT_RECORD)
        .collect();
    assert_eq!(owners.len(), 2);
    assert!(owners.iter().all(|record| record.levels.x == 6));
    assert_eq!(owners[0].bounds.w, owners[1].bounds.w);
    assert!(owners[0].bounds.w > 5.0);
    assert_eq!(
        packed.buildings.len(),
        4,
        "two shared component poses, not one per placement"
    );
    assert_eq!(packed.geometry.pages[0].vertices.len(), 24 * 3);
    let jobs = &packed.batches.values().next().unwrap().1;
    for range in &jobs.ranges {
        let component = &packed.buildings[(range.geometry.w >> COMPONENT_INDEX_SHIFT) as usize];
        assert_eq!(component.levels.y, COMPONENT_RECORD);
        assert_eq!(
            component.uv_offset.truncate().truncate(),
            Vec2::new(0.5, 0.25)
        );
        for owner in
            &jobs.owners[range.geometry.x as usize..(range.geometry.x + range.instances.x) as usize]
        {
            assert_eq!(packed.buildings[*owner as usize].levels.x, 6);
        }
    }
}

#[test]
fn queued_buildings_share_geometry_without_per_part_render_entities() {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<Assets<material::CityMaterial>>();
    world.init_resource::<Assets<ShaderBuffer>>();
    world.init_resource::<PendingGpuCities>();
    world.init_resource::<CityGpuScenes>();
    let mut mesh = Mesh::from(Cuboid::new(2.0, 3.0, 4.0));
    // Repeated indexed triangles span 66 draw clusters, including a partial
    // final cluster, without adding unique geometry.
    let original: Vec<_> = mesh.indices().unwrap().iter().collect();
    let indices = original
        .into_iter()
        .cycle()
        .take(12_483)
        .map(|index| index as u32)
        .collect();
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    for x in [0.0, 20.0] {
        let root = world.spawn_empty().id();
        for level in [1, 2 | FACADE_OVERLAY_FLAG] {
            world
                .resource_mut::<PendingGpuCities>()
                .owners
                .get_mut(PresentationOwner::Scene)
                .parts
                .push(Part {
                    entity: None,
                    root,
                    transform: Mat4::from_translation(Vec3::new(x, 0.0, 0.0)),
                    local_transform: Mat4::IDENTITY,
                    uv_offset: Vec2::ZERO,
                    mesh: mesh.clone(),
                    material: material.clone(),
                    level,
                    fade: None,
                });
        }
    }
    assert_eq!(world.query::<&Mesh3d>().iter(&world).count(), 0);
    let packed = pack(
        &world,
        world
            .resource::<PendingGpuCities>()
            .owners
            .get(PresentationOwner::Scene)
            .groups(None)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(packed.buildings[1].bounds.x, 20.0);
    assert_eq!(packed.buildings[0].levels.x, 6);
    let map_parts = world
        .resource::<PendingGpuCities>()
        .owners
        .get(PresentationOwner::Scene)
        .parts
        .clone();
    world
        .resource_mut::<PendingGpuCities>()
        .owners
        .get_mut(PresentationOwner::RegionalMap)
        .parts = map_parts;
    assemble(&mut world);
    assert_eq!(
        world
            .resource::<CityGpuScenes>()
            .owners
            .get(PresentationOwner::RegionalMap)
            .count,
        2
    );
    assert_ne!(
        world
            .resource::<CityGpuScenes>()
            .owners
            .get(PresentationOwner::RegionalMap)
            .buildings
            .id(),
        world
            .resource::<CityGpuScenes>()
            .owners
            .get(PresentationOwner::Scene)
            .buildings
            .id()
    );
    let scene = world
        .resource::<CityGpuScenes>()
        .owners
        .get(PresentationOwner::Scene);
    assert_eq!(scene.count, 2);
    assert_eq!(scene.batches.len(), 1);
    assert_eq!(scene.batches[0].ranges, 2);
    assert_eq!(scene.batches[0].capacity, 264);
    let source = world
        .resource::<Assets<ShaderBuffer>>()
        .get(&scene.batches[0].source)
        .unwrap()
        .data
        .as_ref()
        .unwrap();
    let overlay_ranges = source
        .as_chunks::<{ size_of::<DrawRange>() }>()
        .0
        .iter()
        .filter(|range| {
            u32::from_le_bytes(range[12..16].try_into().unwrap()) & FACADE_OVERLAY_FLAG != 0
        })
        .count();
    assert_eq!(
        overlay_ranges, 1,
        "shadow policy preserves each range's surface role"
    );
    assert_eq!(
        world
            .resource::<Assets<ShaderBuffer>>()
            .get(&scene.batches[0].source)
            .unwrap()
            .data
            .as_ref()
            .unwrap()
            .len(),
        2 * size_of::<DrawRange>(),
        "source records share placement lists across geometry ranges"
    );
    let vertices = world
        .resource::<Assets<ShaderBuffer>>()
        .get(&scene.batches[0].vertices)
        .unwrap();
    assert_eq!(
        vertices.data.as_ref().unwrap().len(),
        24 * 3 * size_of::<Vec4>(),
        "one canonical cube, even across two buildings and two LODs"
    );
    assert!(
        world
            .resource::<PendingGpuCities>()
            .owners
            .get(PresentationOwner::Scene)
            .parts
            .is_empty()
    );
    let batches = world.query::<&Mesh3d>().iter(&world).count();
    assemble(&mut world);
    assert_eq!(world.query::<&Mesh3d>().iter(&world).count(), batches);
}

#[test]
fn replacing_city_discards_unpublished_instances() {
    let mut world = World::new();
    let root = world.spawn_empty().id();
    world.init_resource::<PendingGpuCities>();
    *world
        .resource_mut::<PendingGpuCities>()
        .owners
        .get_mut(PresentationOwner::Scene) = PendingGpuBuildings {
        parts: vec![Part {
            entity: None,
            root,
            transform: Mat4::IDENTITY,
            local_transform: Mat4::IDENTITY,
            uv_offset: Vec2::ZERO,
            mesh: Handle::default(),
            material: Handle::default(),
            level: 1,
            fade: None,
        }],
        ..Default::default()
    };
    world.init_resource::<CityGpuScenes>();
    world
        .resource_mut::<CityGpuScenes>()
        .owners
        .get_mut(PresentationOwner::RegionalMap)
        .count = 7;
    let map_anchor = world
        .spawn((CityBatchAnchor, PresentationOwner::RegionalMap))
        .id();
    super::super::reset(&mut world, PresentationOwner::Scene);
    assert_eq!(
        world
            .resource::<CityGpuScenes>()
            .owners
            .get(PresentationOwner::RegionalMap)
            .count,
        7
    );
    assert!(world.get_entity(map_anchor).is_ok());
    assert!(
        world
            .resource::<PendingGpuCities>()
            .owners
            .get(PresentationOwner::Scene)
            .parts
            .is_empty()
    );
    assert_eq!(
        world
            .resource::<CityGpuScenes>()
            .owners
            .get(PresentationOwner::Scene)
            .count,
        0
    );
    super::super::reset(&mut world, PresentationOwner::RegionalMap);
    assert!(world.get_entity(map_anchor).is_err());
}

#[test]
fn only_static_outdoor_props_enter_shared_gpu_geometry() {
    use crate::presentation::furniture::VistaFurniturePresentation;
    use adventuresim_building_generator::furniture::{
        FurnitureKey, FurnitureKind, FurnitureVariant,
    };
    use adventuresim_tactical_core::prelude::{
        FurnitureGroupId, FurnitureInstanceId, FurnitureLocation, SceneFurniture,
    };
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    let mesh = world.resource_mut::<Assets<Mesh>>().add(Cuboid::default());
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    let mut roots = Vec::new();
    for index in 0..4 {
        let location = if index == 3 {
            FurnitureLocation::Interior {
                building_id: 1.into(),
                room_id: adventuresim_building_generator::RoomIndex::from_serialized(2),
                storey: adventuresim_building_generator::StoreyIndex::from_serialized(0),
            }
        } else {
            FurnitureLocation::Outdoor {
                group_id: FurnitureGroupId(1),
            }
        };
        let mut root = world.spawn(SceneFurniture {
            id: FurnitureInstanceId(index),
            key: FurnitureKey::natural(FurnitureKind::Barrel, FurnitureVariant::Compact),
            location,
        });
        if index != 2 {
            root.insert(VistaFurniturePresentation);
        }
        let root = root.id();
        roots.push(root);
        world.spawn((
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            GlobalTransform::from_xyz(index as f32 * 20.0, 0.0, 0.0),
        ));
    }
    let parts = props::parts(&mut world);
    assert_eq!(parts.len(), 2);
    assert_eq!(
        parts.iter().map(|part| part.root).collect::<Vec<_>>(),
        roots[..2]
    );
    let packed = pack(&world, input::Group::from_parts(parts.iter().cloned())).unwrap();
    assert_eq!(packed.buildings.len(), 2);
    assert_eq!(packed.geometry.pages[0].vertices.len(), 24 * 3);
    assert_eq!(packed.geometry.pages[0].indices.len(), 36);
    for (index, object) in packed.buildings.iter().enumerate() {
        assert_eq!(object.bounds.x, index as f32 * 20.0);
        assert_eq!(
            object.levels,
            UVec4::new(4, 1, 180.0_f32.to_bits(), 230.0_f32.to_bits())
        );
    }
}

use super::*;

#[test]
fn repeated_buildings_share_geometry_and_release_per_part_render_entities() {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<Assets<material::CityMaterial>>();
    world.init_resource::<Assets<ShaderBuffer>>();
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
        for level in [BuildingRenderLevel::Lod1, BuildingRenderLevel::Lod2] {
            world.spawn((
                ChildOf(root),
                GlobalTransform::from_translation(Vec3::new(x, 0.0, 0.0)),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                PresentedBuildingMesh {
                    scope: BuildingPresentationScope::DistantCity,
                    level,
                    material: if matches!(level, BuildingRenderLevel::Lod2) {
                        adventuresim_building_generator::BuildingLodMaterial::FacadeDetails
                    } else {
                        adventuresim_building_generator::BuildingLodMaterial::Timber
                    },
                    triangles: 4_161,
                },
            ));
        }
    }
    assemble(&mut world);
    let scene = world.resource::<CityGpuScene>();
    assert_eq!(scene.count, 2);
    assert_eq!(scene.batches.len(), 1);
    assert_eq!(scene.batches[0].ranges, 4);
    assert_eq!(scene.batches[0].capacity, 264);
    let source = world
        .resource::<Assets<ShaderBuffer>>()
        .get(&scene.batches[0].source)
        .unwrap()
        .data
        .as_ref()
        .unwrap();
    let overlay_ranges = source
        .chunks_exact(size_of::<UVec4>())
        .filter(|range| {
            u32::from_le_bytes(range[12..16].try_into().unwrap()) & FACADE_OVERLAY_FLAG != 0
        })
        .count();
    assert_eq!(
        overlay_ranges, 2,
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
        4 * size_of::<UVec4>(),
        "source records scale with mesh ranges, not triangle clusters"
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
    assert_eq!(
        world.query::<&PresentedBuildingMesh>().iter(&world).count(),
        0
    );
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
                building_id: 1,
                room_id: 2,
                storey: 0,
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
    let packed = pack(&world, &parts);
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

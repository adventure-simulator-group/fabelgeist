use super::*;
use fabelgeist_armor::{ArmorComponent, ArmorComponentRole};

#[test]
fn material_parts_preserve_skin_uvs_and_relative_position_when_dropped() {
    let red = ArmorComponentMaterial {
        base_color: [1.0, 0.0, 0.0, 1.0],
        metallic: 0.0,
        roughness: 0.9,
    };
    let gold = ArmorComponentMaterial {
        base_color: [1.0, 0.8, 0.0, 1.0],
        metallic: 0.0,
        roughness: 0.8,
    };
    let armor = GeneratedArmor {
        design_hash: [0; 32],
        surface_domain: String::new(),
        positions: vec![
            [1.0, 2.0, 0.0],
            [3.0, 2.0, 0.0],
            [2.0, 4.0, 0.0],
            [5.0, 2.0, 0.0],
            [7.0, 2.0, 0.0],
            [6.0, 4.0, 0.0],
            [100.0; 3],
        ],
        normals: vec![[0.0, 0.0, 1.0]; 7],
        texcoords: vec![[0.25, 0.75]; 7],
        joint_indices: vec![[2; 8]; 7],
        joint_weights: vec![[0.125; 8]; 7],
        indices: vec![0, 1, 2, 3, 4, 5],
        faces: Vec::new(),
        trim: None,
        grids: Vec::new(),
        morphs: Vec::new(),
        components: vec![
            ArmorComponent {
                role: ArmorComponentRole::OuterFabric,
                vertices: 0..3,
                indices: 0..3,
                hinge: None,
                mount: None,
                material: Some(red),
            },
            ArmorComponent {
                role: ArmorComponentRole::Undercloth,
                vertices: 3..6,
                indices: 3..6,
                hinge: None,
                mount: None,
                material: Some(gold),
            },
        ],
    };
    let center = referenced_center(&armor);
    assert_eq!(center, Vec3::new(4.0, 3.0, 0.0));
    let mut meshes = Assets::default();
    let mut materials = Assets::default();
    let parts = equipment_parts(&armor, "linen_tunic", &mut meshes, &mut materials);
    assert_eq!(parts.len(), 2);
    assert_eq!(
        materials.get(&parts[0].material).unwrap().base_color,
        Color::srgba(1.0, 0.0, 0.0, 1.0)
    );
    assert_eq!(
        materials.get(&parts[1].material).unwrap().base_color,
        Color::srgba(1.0, 0.8, 0.0, 1.0)
    );
    for part in &parts {
        let mesh = meshes.get(&part.mesh).unwrap();
        assert_eq!(mesh.count_vertices(), 3);
        assert_eq!(
            mesh.indices().unwrap().iter().collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap(),
            &VertexAttributeValues::Float32x2(vec![[0.25, 0.75]; 3])
        );
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX).unwrap(),
            &VertexAttributeValues::Uint16x4(vec![[2, 0, 0, 0]; 3])
        );
    }
    let mut app = App::new();
    app.insert_resource(meshes).add_systems(
        Update,
        crate::equipment::render_binding::sync_render_bindings,
    );
    let item = app.world_mut().spawn(TacticalSceneItem).id();
    let entities = parts
        .iter()
        .map(|part| {
            app.world_mut()
                .spawn((
                    ProceduralEquipmentPart::new(item, default(), vec![], center),
                    Mesh3d(part.mesh.clone()),
                    Visibility::Hidden,
                ))
                .id()
        })
        .collect::<Vec<_>>();
    app.update();
    let first_vertices = entities
        .iter()
        .map(|&entity| {
            let mesh = &app.world().get::<Mesh3d>(entity).unwrap().0;
            let mesh = app.world().resource::<Assets<Mesh>>().get(mesh).unwrap();
            assert!(!mesh.contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX));
            Vec3::from_array(
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap()[0],
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first_vertices,
        [Vec3::new(-3.0, -1.0, 0.0), Vec3::new(1.0, -1.0, 0.0)]
    );
}

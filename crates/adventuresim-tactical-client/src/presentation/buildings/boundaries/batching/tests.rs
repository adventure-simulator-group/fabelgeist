use super::*;
use adventuresim_tactical_core::prelude::CityBoundaryMaterial;
use bevy::mesh::VertexAttributeValues;

fn member(centre: Vec3) -> CityBoundaryMember {
    CityBoundaryMember {
        centre_metres: centre,
        size_metres: Vec3::new(7.0, 2.0, 0.4),
        yaw_radians: 0.7,
        material: CityBoundaryMaterial::Masonry,
    }
}

fn surface(mesh: &Mesh, origin: Vec3) -> Vec<(Vec3, Vec3, Vec2, Vec4)> {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions");
    };
    let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        panic!("normals");
    };
    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
        panic!("UVs");
    };
    let Some(VertexAttributeValues::Float32x4(tangents)) = mesh.attribute(Mesh::ATTRIBUTE_TANGENT)
    else {
        panic!("tangents");
    };
    mesh.indices()
        .unwrap()
        .iter()
        .map(|index| {
            (
                Vec3::from_array(positions[index]) + origin,
                Vec3::from_array(normals[index]),
                Vec2::from_array(uvs[index]),
                Vec4::from_array(tangents[index]),
            )
        })
        .collect()
}

#[test]
fn merged_enclosures_preserve_world_surfaces_uvs_and_elevation() {
    let material = Handle::default();
    let mut batches = BoundaryBatches::default();
    let members = [
        member(Vec3::new(-12.0, 1.0, 4.0)),
        member(Vec3::new(-8.0, 3.0, 8.0)),
    ];
    let mut expected = Vec::new();
    for member in members {
        let transform = Transform::from_translation(member.centre_metres + Vec3::Y * 15.0)
            .with_rotation(Quat::from_rotation_y(member.yaw_radians));
        let original = crate::presentation::recipe_mesh::metric_cuboid(member.size_metres)
            .transformed_by(transform);
        expected.extend(surface(&original, Vec3::ZERO));
        batches.insert(member, 15.0, material.clone());
    }
    let mut merged = batches.finish();
    assert_eq!(merged.len(), 1);
    let batch = merged.next().unwrap();
    let actual = surface(&batch.mesh, batch.origin);
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(actual.0.abs_diff_eq(expected.0, 0.0001));
        assert!(actual.1.abs_diff_eq(expected.1, 0.0001));
        assert_eq!(actual.2, expected.2);
        assert!(actual.3.abs_diff_eq(expected.3, 0.0001));
    }
}

#[test]
fn spatial_cells_and_distinct_materials_remain_separate() {
    let mut materials = Assets::<StandardMaterial>::default();
    let stone = materials.add(StandardMaterial::default());
    let timber = materials.add(StandardMaterial::default());
    let mut batches = BoundaryBatches::default();
    for x in [1.0, 5.0, 129.0, -1.0] {
        batches.insert(member(Vec3::new(x, 0.0, 0.0)), 0.0, stone.clone());
    }
    batches.insert(member(Vec3::X), 0.0, timber.clone());
    assert_eq!(batches.members, 5);
    let merged: Vec<_> = batches.finish().collect();
    assert_eq!(merged.len(), 4);
    assert_eq!(
        merged
            .iter()
            .filter(|batch| batch.material == timber)
            .count(),
        1
    );
    assert!(merged.iter().any(|batch| batch.origin.x < 0.0));
    assert!(merged.iter().any(|batch| batch.origin.x > 0.0));
}

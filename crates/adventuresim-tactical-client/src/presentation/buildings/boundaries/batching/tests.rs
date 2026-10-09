use super::*;
use adventuresim_tactical_core::{
    city_layout::grounding::BoundarySupportElement, prelude::CityBoundaryMaterial,
};
use bevy::mesh::VertexAttributeValues;

fn cell(centre: Vec3) -> BoundarySupportCell {
    BoundarySupportCell::new(
        [
            centre + Vec3::new(0.0, 2.0, 0.0),
            centre + Vec3::new(7.0, 3.0, 0.0),
            centre + Vec3::new(0.0, 2.0, 0.4),
            centre,
            centre + Vec3::new(7.0, 1.0, 0.0),
            centre + Vec3::new(0.0, 0.0, 0.4),
        ]
        .map(|p| {
            adventuresim_building_generator::spatial_geometry::Position::from_metres(p).unwrap()
        }),
        CityBoundaryMaterial::Masonry,
        BoundarySupportElement::Wall(0),
    )
    .unwrap()
}

#[test]
fn merged_terraced_enclosures_keep_exact_world_vertices_and_finite_surface_attributes() {
    let mut batches = BoundaryBatches::default();
    let cells = [
        cell(Vec3::new(-12.0, 1.0, 4.0)),
        cell(Vec3::new(-8.0, 3.0, 8.0)),
    ];
    for cell in &cells {
        batches.insert(cell, 15.0, Handle::default());
    }
    let merged: Vec<_> = batches.finish().collect();
    assert_eq!(merged.len(), 1);
    let batch = &merged[0];
    let Some(VertexAttributeValues::Float32x3(positions)) =
        batch.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions")
    };
    assert_eq!(positions.len(), 48);
    for position in positions {
        let world = Vec3::from_array(*position) + batch.origin;
        assert!(
            cells
                .iter()
                .flat_map(|cell| cell.native_positions())
                .any(|p| (p + Vec3::Y * 15.0).abs_diff_eq(world, 0.0001))
        );
    }
    let Some(VertexAttributeValues::Float32x3(normals)) =
        batch.mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        panic!("normals")
    };
    assert!(
        normals
            .iter()
            .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 0.0001)
    );
    let Some(VertexAttributeValues::Float32x4(tangents)) =
        batch.mesh.attribute(Mesh::ATTRIBUTE_TANGENT)
    else {
        panic!("tangents")
    };
    assert!(tangents.iter().all(|t| Vec4::from_array(*t).is_finite()));
}

#[test]
fn spatial_cells_and_shared_materials_bound_enclosure_draws() {
    let mut materials = Assets::<StandardMaterial>::default();
    let stone = materials.add(StandardMaterial::default());
    let timber = materials.add(StandardMaterial::default());
    let mut batches = BoundaryBatches::default();
    for x in [1.0, 5.0, 129.0, -10.0] {
        batches.insert(&cell(Vec3::new(x, 0.0, 0.0)), 0.0, stone.clone());
    }
    batches.insert(&cell(Vec3::X), 0.0, timber);
    assert_eq!(batches.members, 5);
    assert_eq!(batches.finish().len(), 4);
}

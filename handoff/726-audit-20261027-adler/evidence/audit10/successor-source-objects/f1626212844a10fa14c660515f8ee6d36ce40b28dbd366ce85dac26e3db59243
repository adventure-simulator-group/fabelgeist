use super::*;
use crate::city_layout::grounding::tests::Fixture;

#[test]
fn foundation_cells_round_trip_dated_fixture_vertices_topology_and_members() {
    let fixture = Fixture::load();
    let source = fixture.source();
    let mesh = fixture
        .selected_plan(&source)
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap();
    let document = serde_json::to_value(&mesh).unwrap();
    assert!(document["solid_triangles"].is_array());
    assert!(document["support_triangles"].is_array());
    let human: PropertyFoundationMesh = serde_json::from_value(document).unwrap();
    assert_eq!(human, mesh);
    let mut bytes = Vec::new();
    ciborium::into_writer(&mesh, &mut bytes).unwrap();
    let restored: PropertyFoundationMesh = ciborium::from_reader(bytes.as_slice()).unwrap();
    assert_eq!(restored, mesh);
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&restored.positions),
        bytemuck::cast_slice::<_, u8>(&mesh.positions)
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&restored.cut_faces),
        bytemuck::cast_slice::<_, u8>(&mesh.cut_faces)
    );
    assert_eq!(restored.volume_cubic_metres(), mesh.volume_cubic_metres());
    let encoded = postcard::to_allocvec(&mesh).unwrap();
    assert_eq!(
        postcard::from_bytes::<PropertyFoundationMesh>(&encoded).unwrap(),
        mesh
    );
    let packed: ciborium::value::Value = ciborium::from_reader(bytes.as_slice()).unwrap();
    let ciborium::value::Value::Map(fields) = packed else {
        panic!("foundation cells are a map")
    };
    assert_eq!(fields.len(), 4);
    assert!(fields.iter().all(|(key, _)| !matches!(key, ciborium::value::Value::Text(key) if key == "solid_triangles" || key == "support_triangles")));
}

#[test]
fn incomplete_or_changed_prisms_cannot_be_silently_encoded_as_canonical_cells() {
    let fixture = Fixture::load();
    let source = fixture.source();
    let mesh = fixture
        .selected_plan(&source)
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap();
    let mut changed = mesh.clone();
    changed.solid_triangles[0].swap(1, 2);
    assert!(ciborium::into_writer(&changed, Vec::new()).is_err());
    changed = mesh.clone();
    changed.support_triangles[0].swap(1, 2);
    assert!(postcard::to_allocvec(&changed).is_err());
    changed = mesh.clone();
    changed.positions.pop();
    assert!(ciborium::into_writer(&changed, Vec::new()).is_err());
    let cells = FoundationCells {
        property_id: mesh.property_id,
        member_building_ids: mesh.member_building_ids,
        positions: mesh.positions[..5].to_vec(),
        cut_faces: mesh.cut_faces,
    };
    let mut bytes = Vec::new();
    ciborium::into_writer(&cells, &mut bytes).unwrap();
    assert!(ciborium::from_reader::<PropertyFoundationMesh, _>(bytes.as_slice()).is_err());
    bytes.pop();
    assert!(ciborium::from_reader::<PropertyFoundationMesh, _>(bytes.as_slice()).is_err());
}

#[test]
fn malformed_binary_cells_reject_nonfinite_pairs_reversed_heights_and_members() {
    let fixture = Fixture::load();
    let source = fixture.source();
    let mesh = fixture
        .selected_plan(&source)
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap();
    for case in 0..5 {
        let mut wire = FoundationCells {
            property_id: mesh.property_id,
            member_building_ids: mesh.member_building_ids.clone(),
            positions: mesh.positions.clone(),
            cut_faces: mesh.cut_faces.clone(),
        };
        match case {
            0 => wire.positions[0].y = f32::NAN,
            1 => wire.positions[3].x += 1.0,
            2 => wire.positions[3].y = wire.positions[0].y + 1.0,
            3 => wire.member_building_ids.push(wire.member_building_ids[0]),
            _ => wire.cut_faces.push([Vec3::splat(f32::INFINITY); 3]),
        }
        let encoded = postcard::to_allocvec(&wire).unwrap();
        assert!(postcard::from_bytes::<PropertyFoundationMesh>(&encoded).is_err());
    }
}

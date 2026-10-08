use super::*;
use adventuresim_tactical_core::city_layout::CityPropertyId;

fn foundation(property: u64, cells: &[[Vec3; 3]]) -> PropertyFoundationMesh {
    foundation_with_cuts(property, cells, Vec::new())
}
fn foundation_with_cuts(
    property: u64,
    cells: &[[Vec3; 3]],
    cuts: Vec<[Vec3; 3]>,
) -> PropertyFoundationMesh {
    use adventuresim_building_generator::spatial_geometry::Position;
    use adventuresim_tactical_core::city_layout::grounding::PropertyMembers;
    let prisms = cells
        .iter()
        .map(|top| {
            [
                top[0],
                top[1],
                top[2],
                top[0] - Vec3::Y * 2.2,
                top[1] - Vec3::Y * 2.2,
                top[2] - Vec3::Y * 2.2,
            ]
            .map(|p| Position::from_metres(p).unwrap())
        })
        .collect();
    PropertyFoundationMesh::from_prisms(
        CityPropertyId(property),
        PropertyMembers::new(vec![property.into()]).unwrap(),
        prisms,
        cuts.into_iter()
            .map(|t| t.map(|p| Position::from_metres(p).unwrap()))
            .collect(),
    )
    .unwrap()
}

fn adjacent_cells() -> [[Vec3; 3]; 2] {
    let a = Vec3::new(0.0, 2.0, 0.0);
    let b = Vec3::new(1.0, 2.0, 0.0);
    let c = Vec3::new(0.0, 2.0, 1.0);
    let d = Vec3::new(1.0, 2.0, 1.0);
    [[a, b, c], [b, d, c]]
}

fn surface(foundations: Vec<PropertyFoundationMesh>) -> BoundedSettlementTerrain {
    BoundedSettlementTerrain::from_foundations(
        foundations,
        Vec::new(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn region_selection_preserves_crossing_cut_faces_and_complete_boundary_cells() {
    let cells = adjacent_cells();
    let near = foundation(41, &cells);
    let distant_cells = cells.map(|cell| cell.map(|p| p + Vec3::X * 100.0));
    let unrelated = foundation(42, &distant_cells);
    let cut_owner = foundation_with_cuts(
        43,
        &distant_cells,
        vec![[
            Vec3::new(-2.0, -1.0, 0.25),
            Vec3::new(2.0, -1.0, 0.25),
            Vec3::new(2.0, 1.0, 0.25),
        ]],
    );
    let surface = surface(vec![near, unrelated, cut_owner]);
    let original = serde_json::to_vec(&surface).unwrap();
    let regions = [[Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5)]];
    let complete = GroundPresentation::new(&surface);
    let selected = GroundPresentation::in_rectangles(&surface, &regions);
    let clipped = |presentation: &GroundPresentation<'_>| {
        presentation
            .triangles(None)
            .flat_map(|triangle| {
                super::super::clip::PreparedTriangle::new(triangle)
                    .in_rectangle(regions[0][0], regions[0][1])
            })
            .map(|triangle| triangle.map(|p| p.to_array().map(f32::to_bits)))
            .collect::<Vec<_>>()
    };
    assert_eq!(clipped(&selected), clipped(&complete));
    assert!(!clipped(&selected).is_empty());
    assert_eq!(selected.foundations.len(), 2);
    assert_eq!(
        selected.foundations[1].mesh.property_id(),
        CityPropertyId(43)
    );
    assert_eq!(serde_json::to_vec(&surface).unwrap(), original);
}

#[test]
fn coincident_internal_sides_are_removed_without_changing_bearings_or_closed_cells() {
    for yaw in [0.0, 0.73] {
        let transform = Quat::from_rotation_y(yaw);
        let cells =
            adjacent_cells().map(|cell| cell.map(|p| transform * p + Vec3::new(37.5, 5.0, -81.25)));
        let surface = surface(vec![foundation(41, &cells)]);
        let original = serde_json::to_vec(&surface).unwrap();
        let all: Vec<_> = surface.presentation_triangles().collect();
        let presentation = GroundPresentation::new(&surface);
        let visible: Vec<_> = presentation.triangles(None).collect();
        assert_eq!(all.len(), 16);
        assert_eq!(visible.len(), 12, "only the shared side pair is removed");
        assert!(visible.iter().all(|face| all.contains(face)));
        for f in surface.foundations() {
            for indices in f.support_triangles() {
                assert!(visible.contains(&indices.map(|i| f.positions()[i as usize])));
            }
            // Downward buried bottoms remain available in ordinary geometry.
            for face in f.solid_triangles().iter().skip(1).step_by(8) {
                assert!(visible.contains(&face.map(|i| f.positions()[i as usize])));
            }
            let point = cells[0].iter().copied().sum::<Vec3>() / 3.0;
            let hit = surface
                .highest_surface_at(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                        point.xz(),
                    )
                    .unwrap(),
                )
                .unwrap();
            let height = hit.elevation.metres();
            assert!((height - point.y).abs() < 0.001);
            assert!(
                f.collider()
                    .unwrap()
                    .into_solid()
                    .unwrap()
                    .cast_ray(
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        point + Vec3::Y,
                        Vec3::NEG_Y,
                        2.0,
                        false
                    )
                    .is_some()
            );
        }
        assert_eq!(serde_json::to_vec(&surface).unwrap(), original);
    }
}

#[test]
fn separate_owners_different_terraces_and_unmatched_faces_are_retained() {
    let cells = adjacent_cells();
    let separate = surface(vec![
        foundation(41, &cells[..1]),
        foundation(42, &cells[1..]),
    ]);
    assert_eq!(
        GroundPresentation::new(&separate).triangles(None).count(),
        16
    );
    let stepped = [cells[0], cells[1].map(|p| p + Vec3::Y)];
    let stepped = surface(vec![foundation(41, &stepped)]);
    assert_eq!(
        GroundPresentation::new(&stepped).triangles(None).count(),
        16
    );
    let unmatched = surface(vec![foundation(41, &cells[..1])]);
    assert_eq!(
        GroundPresentation::new(&unmatched).triangles(None).count(),
        8
    );
}

#[test]
fn duplicate_or_ambiguous_side_occurrences_cannot_hide_geometry() {
    let cells = adjacent_cells();
    let same_direction = surface(vec![foundation(41, &[cells[0], cells[0]])]);
    assert_eq!(
        GroundPresentation::new(&same_direction)
            .triangles(None)
            .count(),
        16
    );
    let ambiguous = surface(vec![foundation(41, &[cells[0], cells[1], cells[0]])]);
    assert_eq!(
        GroundPresentation::new(&ambiguous).triangles(None).count(),
        24
    );
}

#[test]
fn source_and_cut_faces_are_retained_bit_for_bit() {
    let f = foundation_with_cuts(41, &adjacent_cells(), vec![[Vec3::ZERO, Vec3::Y, Vec3::X]]);
    let source = BoundedSettlementTerrain::from_foundations(
        vec![f],
        vec![[Vec3::ZERO, Vec3::X, Vec3::Z].map(|p| {
            adventuresim_building_generator::spatial_geometry::Position::from_metres(p).unwrap()
        })],
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
            .unwrap(),
    )
    .unwrap();
    let presentation = GroundPresentation::new(&source);
    let triangles: Vec<_> = presentation.triangles(None).collect();
    assert_eq!(triangles[0], [Vec3::ZERO, Vec3::Z, Vec3::X]);
    assert_eq!(triangles[1], source.foundations()[0].cut_faces()[0]);
}

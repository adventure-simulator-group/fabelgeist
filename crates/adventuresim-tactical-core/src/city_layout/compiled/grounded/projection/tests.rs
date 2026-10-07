use super::*;
use bevy::math::{Vec3, Vec3Swizzles};

fn accepted() -> (GroundedCitySceneLayout, GeographicSurface) {
    let (layout, source, policy) = super::super::tests::fixture();
    let grounded = SelectedCityGrounding::select(&layout, &source, policy)
        .unwrap()
        .compile()
        .unwrap();
    (grounded, source)
}

#[test]
fn compact_round_trip_reconstructs_exact_closed_support_and_retains_promotion() {
    let (accepted, source) = accepted();
    let projection = accepted.support_projection().unwrap();
    let bytes = serde_json::to_vec(&projection).unwrap();
    let expanded_bytes = serde_json::to_vec(accepted.terrain()).unwrap().len();
    assert!(bytes.len() < expanded_bytes);
    let decoded: CityGroundingProjection = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, projection);
    let mut placements = physical_placements(accepted.layout());
    placements.reverse();
    let rebuilt = decoded.reconstruct(&source, &placements).unwrap();
    assert_eq!(&rebuilt, accepted.terrain());
    assert_eq!(
        rebuilt.foundations()[0].member_building_ids(),
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    let mut triangles: Vec<_> = source.triangles().collect();
    triangles.reverse();
    let reordered = GeographicSurface::from_triangles(triangles).unwrap();
    assert_eq!(
        decoded.reconstruct(&reordered, &placements).unwrap(),
        rebuilt
    );
}

#[test]
fn compact_support_rejects_a_different_source_floor_programme_or_horizontal_pose() {
    let (accepted, source) = accepted();
    let projection = accepted.support_projection().unwrap();
    let placements = physical_placements(accepted.layout());
    let altered = GeographicSurface::from_triangles(
        source
            .triangles()
            .map(|triangle| triangle.map(|point| point + Vec3::Y * 0.01)),
    )
    .unwrap();
    assert!(matches!(
        projection.reconstruct(&altered, &placements),
        Err(CityGroundingProjectionError::SourceMismatch { .. })
    ));
    for field in 0..3 {
        let mut altered = placements.clone();
        match field {
            0 => {
                altered[0].base_elevation_metres =
                    crate::city_layout::grounding::SupportElevation::from_metres(
                        altered[0].base_elevation_metres.metres() + 0.01,
                    )
                    .unwrap()
            }
            1 => {
                altered[0].centre_metres = altered[0]
                    .centre_metres
                    .translated(
                        crate::scene_coordinates::PlanDisplacement::try_from(Vec2::new(0.01, 0.0))
                            .unwrap(),
                    )
                    .unwrap()
            }
            _ => altered[0].program.seed = altered[0].program.seed.wrapping_offset(1),
        }
        assert!(matches!(
            projection.reconstruct(&source, &altered),
            Err(CityGroundingProjectionError::PlacementMismatch { .. })
        ));
    }
}

#[test]
fn malformed_compact_triangle_is_rejected_before_indexing_with_exact_property() {
    let (accepted, _) = accepted();
    let projection = accepted.support_projection().unwrap();
    let mut value = serde_json::to_value(&projection).unwrap();
    value["surfaces"][0]["mesh"]["support_triangles"][0][0] = u32::MAX.into();
    let error = serde_json::from_value::<CityGroundingProjection>(value).unwrap_err();
    assert!(error.to_string().contains("1238"));
    assert!(error.to_string().contains("triangle indices"));
}

#[test]
fn compact_support_cannot_expand_grading_or_relax_its_grade_bound() {
    let (accepted, _) = accepted();
    let original = accepted.support_projection().unwrap();
    let mut value = serde_json::to_value(&original).unwrap();
    let x = value["surfaces"][0]["mesh"]["positions"][0][0]
        .as_f64()
        .unwrap();
    value["surfaces"][0]["mesh"]["positions"][0][0] = (x + 100.0).into();
    let error = serde_json::from_value::<CityGroundingProjection>(value).unwrap_err();
    assert!(error.to_string().contains("1238"));
    assert!(error.to_string().contains("regions or clipping outlines"));
    let mut value = serde_json::to_value(&original).unwrap();
    for point in value["surfaces"][0]["mesh"]["positions"]
        .as_array_mut()
        .unwrap()
    {
        point[1] = (point[0].as_f64().unwrap() * 2.0).into();
    }
    let error = serde_json::from_value::<CityGroundingProjection>(value).unwrap_err();
    assert!(error.to_string().contains("1238"));
    assert!(error.to_string().contains("support grade"));
}

#[test]
fn decoded_projection_rejects_surface_floor_owner_and_order_changes() {
    let (accepted, _) = accepted();
    let projection = accepted.support_projection().unwrap();
    let value = serde_json::to_value(&projection).unwrap();
    let mut shifted = value.clone();
    for point in shifted["surfaces"][0]["mesh"]["positions"]
        .as_array_mut()
        .unwrap()
    {
        point[1] = (point[1].as_f64().unwrap() + 0.01).into();
    }
    assert!(serde_json::from_value::<CityGroundingProjection>(shifted).is_err());
    let mut owner = value.clone();
    owner["surfaces"][0]["mesh"]["property_id"] = 1239.into();
    assert!(serde_json::from_value::<CityGroundingProjection>(owner).is_err());
    let mut members = value.clone();
    members["surfaces"][0]["mesh"]["member_building_ids"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert!(serde_json::from_value::<CityGroundingProjection>(members).is_err());
    let mut compound = accepted.layout().compounds.clone();
    let property = &mut compound[0];
    std::mem::swap(
        &mut property.front_building_id,
        &mut property.rear_building_id,
    );
    assert!(matches!(
        projection.validate_compound_bindings(&compound),
        Err(CityGroundingProjectionError::PropertyMismatch { .. })
    ));
}

#[test]
fn floor_binding_rejects_missing_faces_and_extra_wrong_level_area() {
    let (accepted, source) = accepted();
    let projection = accepted.support_projection().unwrap();
    let value = serde_json::to_value(&projection).unwrap();
    let mesh = projection.surfaces()[0].mesh();
    let floor = &projection.surfaces()[0].floor_bearings()[0];
    let face = mesh
        .support_triangles()
        .iter()
        .position(|triangle| {
            let points = triangle.map(|i| mesh.positions()[i as usize]);
            let centre = (points.iter().copied().sum::<Vec3>() / 3.0).xz().as_dvec2();
            points
                .iter()
                .all(|point| point.y == floor.elevation().metres())
                && (points[1].xz() - points[0].xz())
                    .perp_dot(points[2].xz() - points[0].xz())
                    .abs()
                    > 2.0
                && floor.regions().iter().any(|region| {
                    let outline = region.outline().vertices();
                    (0..outline.len()).all(|i| {
                        let a = outline[i].metres().as_dvec2();
                        let b = outline[(i + 1) % outline.len()].metres().as_dvec2();
                        (b - a).perp_dot(centre - a) >= 0.0
                    })
                })
        })
        .unwrap();
    let mut missing = value.clone();
    missing["surfaces"][0]["mesh"]["support_triangles"]
        .as_array_mut()
        .unwrap()
        .remove(face);
    assert!(serde_json::from_value::<CityGroundingProjection>(missing).is_err());
    let mut raised = value.clone();
    let triangle = mesh.support_triangles()[face];
    let start = mesh.positions().len() as u32;
    for index in triangle {
        let mut point = mesh.positions()[index as usize];
        point.y += 0.01;
        raised["surfaces"][0]["mesh"]["positions"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(point).unwrap());
    }
    raised["surfaces"][0]["mesh"]["support_triangles"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!([start, start + 1, start + 2]));
    assert!(serde_json::from_value::<CityGroundingProjection>(raised).is_err());
    let mut changed_floor = value;
    for point in changed_floor["surfaces"][0]["mesh"]["positions"]
        .as_array_mut()
        .unwrap()
    {
        point[1] = (point[1].as_f64().unwrap() + 0.01).into();
    }
    for bearing in changed_floor["surfaces"][0]["floor_bearings"]
        .as_array_mut()
        .unwrap()
    {
        bearing["elevation"] = (bearing["elevation"].as_f64().unwrap() + 0.01).into();
    }
    let changed: CityGroundingProjection = serde_json::from_value(changed_floor).unwrap();
    assert!(matches!(
        changed.reconstruct(&source, &physical_placements(accepted.layout())),
        Err(CityGroundingProjectionError::FloorMismatch { .. })
    ));
}

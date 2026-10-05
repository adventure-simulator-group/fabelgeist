use super::*;
use bevy::math::Vec3;

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
    let projection = accepted.support_projection();
    let bytes = serde_json::to_vec(&projection).unwrap();
    let expanded_bytes = serde_json::to_vec(accepted.terrain()).unwrap().len();
    assert!(bytes.len() < expanded_bytes);
    let decoded: CityGroundingProjection = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, projection);
    let mut placements = physical_placements(accepted.layout());
    placements.reverse();
    let rebuilt = decoded.reconstruct(&source, &placements).unwrap();
    assert_eq!(&rebuilt, accepted.terrain());
    assert_eq!(rebuilt.foundations[0].member_building_ids, [1238, 17622]);
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
    let projection = accepted.support_projection();
    let placements = physical_placements(accepted.layout());
    let altered = GeographicSurface::from_triangles(
        source
            .triangles()
            .map(|triangle| triangle.map(|point| point + Vec3::Y * 0.01)),
    )
    .unwrap();
    assert!(matches!(
        projection.reconstruct(&altered, &placements),
        Err(CityGroundingProjectionError::SourceMismatch)
    ));
    for field in 0..3 {
        let mut altered = placements.clone();
        match field {
            0 => altered[0].base_elevation_metres += 0.01,
            1 => altered[0].centre_metres.x += 0.01,
            _ => altered[0].program.seed = altered[0].program.seed.wrapping_add(1),
        }
        assert!(matches!(
            projection.reconstruct(&source, &altered),
            Err(CityGroundingProjectionError::PlacementMismatch)
        ));
    }
}

#[test]
fn malformed_compact_triangle_is_rejected_before_indexing_with_exact_property() {
    let (accepted, source) = accepted();
    let projection = accepted.support_projection();
    let mut value = serde_json::to_value(&projection).unwrap();
    value["surfaces"][0]["mesh"]["support_triangles"][0][0] = u32::MAX.into();
    let decoded: CityGroundingProjection = serde_json::from_value(value).unwrap();
    assert!(matches!(
        decoded.reconstruct(&source, &physical_placements(accepted.layout())),
        Err(CityGroundingProjectionError::Surface {
            property: CityPropertyId(1238),
            issue: SupportSurfaceIssue::Topology,
        })
    ));
}

#[test]
fn compact_support_cannot_expand_grading_or_relax_its_grade_bound() {
    let (accepted, source) = accepted();
    let original = accepted.support_projection();
    let placements = physical_placements(accepted.layout());
    let mut value = serde_json::to_value(&original).unwrap();
    let x = value["surfaces"][0]["mesh"]["positions"][0][0]
        .as_f64()
        .unwrap();
    value["surfaces"][0]["mesh"]["positions"][0][0] = (x + 100.0).into();
    let outside: CityGroundingProjection = serde_json::from_value(value).unwrap();
    assert!(matches!(
        outside.reconstruct(&source, &placements),
        Err(CityGroundingProjectionError::Surface {
            property: CityPropertyId(1238),
            issue: SupportSurfaceIssue::Bounds,
        })
    ));
    let mut value = serde_json::to_value(&original).unwrap();
    for point in value["surfaces"][0]["mesh"]["positions"]
        .as_array_mut()
        .unwrap()
    {
        point[1] = (point[0].as_f64().unwrap() * 2.0).into();
    }
    let steep: CityGroundingProjection = serde_json::from_value(value).unwrap();
    assert!(matches!(
        steep.reconstruct(&source, &placements),
        Err(CityGroundingProjectionError::Surface {
            property: CityPropertyId(1238),
            issue: SupportSurfaceIssue::Grade { .. },
        })
    ));
}

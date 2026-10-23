use super::*;

fn leaf() -> DoorSpec<Architectural> {
    DoorSpec {
        opening: OpeningAssemblyId(93),
        source: ResolvedItemId(94),
        closed_centre: Position::from_metres(Vec3::new(-2.0, 1.0, 4.0)).unwrap(),
        hinge_centre: Position::from_metres(Vec3::new(-2.5, 1.0, 4.0)).unwrap(),
        size_metres: LeafDimensions::from_metres(Vec3::new(1.0, 2.0, 0.08)).unwrap(),
        closed_yaw_radians: Radians::ZERO,
        tangent: PlanDirection::from_normalized(Vec2::X).unwrap(),
        outward: PlanDirection::from_normalized(-Vec2::Y).unwrap(),
        open_angle_radians: Radians::new(-1.0).unwrap(),
    }
}

#[test]
fn decoded_leaves_preserve_identity_and_reject_invalid_roles() {
    let source = leaf();
    let restored: DoorSpec<Architectural> =
        postcard::from_bytes(&postcard::to_allocvec(&source).unwrap()).unwrap();
    assert_eq!(restored, source);
    for (field, invalid) in [
        ("size_metres", serde_json::json!([1.0, 0.0, 0.08])),
        ("outward", serde_json::json!([2.0, 0.0])),
    ] {
        let mut wire = serde_json::to_value(source).unwrap();
        wire[field] = invalid;
        let error = serde_json::from_value::<DoorSpec<Architectural>>(wire)
            .unwrap_err()
            .to_string();
        assert!(error.contains("93") && error.contains("94"), "{error}");
    }
}

#[test]
fn sweep_overflow_cannot_be_reduced_into_an_ordinary_radius() {
    let mut source = leaf();
    source.closed_centre = Position::from_metres(Vec3::new(f32::MAX, 1.0, 0.0)).unwrap();
    source.hinge_centre = Position::from_metres(Vec3::new(-f32::MAX, 1.0, 0.0)).unwrap();
    let error = source.horizontal_sweep_radius_metres().unwrap_err();
    assert_eq!(error.opening, source.opening);
    assert_eq!(error.source_id, Some(source.source));
    assert!(matches!(
        error.cause,
        DoorErrorCause::Geometry(GeometryError::NonFinite {
            role: crate::spatial_geometry::GeometryRole::SweepRadius,
            ..
        })
    ));
}

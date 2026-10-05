use super::*;
use bevy::{
    math::{Vec2, Vec3},
    reflect::{PartialReflect, ReflectRef},
};

#[test]
fn admission_and_decoding_enforce_roles_without_repairing_values() {
    let signed = Vec3::new(-3.0, 0.0, 4.0);
    let position = Position::<Architectural>::from_metres(signed).unwrap();
    let displacement = Displacement::<Architectural>::ZERO;
    assert_eq!(position.translated(displacement).unwrap(), position);
    assert_eq!(position.displacement_to(position).unwrap(), displacement);
    assert!(CuboidDimensions::from_metres(Vec3::ZERO).is_ok());
    assert!(LeafDimensions::from_metres(Vec3::ZERO).is_err());
    assert!(PlanDimensions::from_metres(Vec2::new(1.0, 0.0)).is_err());
    assert!(PlanExtents::from_metres(Vec2::ZERO).is_ok());
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let bytes = postcard::to_allocvec(&Vec3::new(0.0, invalid, 0.0)).unwrap();
        assert!(postcard::from_bytes::<Position<Architectural>>(&bytes).is_err());
        assert!(postcard::from_bytes::<Displacement<Architectural>>(&bytes).is_err());
        assert!(postcard::from_bytes::<CuboidDimensions>(&bytes).is_err());
        assert!(postcard::from_bytes::<LeafDimensions>(&bytes).is_err());
        let bytes = postcard::to_allocvec(&invalid).unwrap();
        assert!(postcard::from_bytes::<Elevation<Architectural>>(&bytes).is_err());
        assert!(postcard::from_bytes::<Radians>(&bytes).is_err());
    }
    assert!(serde_json::from_str::<CuboidDimensions>("[-1,2,3]").is_err());
    assert!(serde_json::from_str::<LeafDimensions>("[1,0,3]").is_err());
    assert!(matches!(position.reflect_ref(), ReflectRef::Opaque(_)));
    assert!(matches!(
        CuboidDimensions::ZERO.reflect_ref(),
        ReflectRef::Opaque(_)
    ));
    let mut reflected = position;
    assert!(reflected.try_apply(&Vec3::splat(f32::NAN)).is_err());
    assert_eq!(reflected, position);
    assert_eq!(
        postcard::to_allocvec(&position).unwrap(),
        postcard::to_allocvec(&signed).unwrap()
    );
    assert_eq!(size_of::<Position<Architectural>>(), size_of::<Vec3>());
    assert_eq!(size_of::<CuboidDimensions>(), size_of::<Vec3>());
    assert_eq!(align_of::<Position<Architectural>>(), align_of::<Vec3>());
}

#[test]
fn bounds_distinguish_order_degeneracy_clearance_and_overflow() {
    let flat = SpatialBounds::<Architectural>::from_metres(Vec3::ZERO, Vec3::X).unwrap();
    assert_eq!(flat.extent().unwrap().metres(), Vec3::X);
    assert!(ClearanceVolume::new(flat).is_err());
    let point = SpatialBounds::<Architectural>::at(Position::ORIGIN);
    assert_eq!(point.centre().unwrap(), Position::ORIGIN);
    assert!(
        serde_json::from_str::<SpatialBounds<Architectural>>("{\"min\":[0,2,0],\"max\":[1,1,1]}")
            .is_err()
    );
    assert!(
        serde_json::from_str::<ClearanceVolume<Architectural>>("{\"min\":[0,0,0],\"max\":[1,0,1]}")
            .is_err()
    );
    let huge =
        SpatialBounds::<Architectural>::at(Position::from_metres(Vec3::splat(f32::MAX)).unwrap());
    assert_eq!(
        huge.centre().unwrap_err(),
        GeometryError::NonFinite {
            role: GeometryRole::BoundsCentre,
            axis: CoordinateAxis::X,
        }
    );
    let wide =
        SpatialBounds::<Architectural>::from_metres(Vec3::splat(-f32::MAX), Vec3::splat(f32::MAX))
            .unwrap();
    assert_eq!(
        wide.extent().unwrap_err(),
        GeometryError::NonFinite {
            role: GeometryRole::BoundsExtent,
            axis: CoordinateAxis::X,
        }
    );
}

#[test]
fn normalized_directions_reject_bad_decoding_and_preserve_normalization_arithmetic() {
    let input = Vec2::new(3.0, -7.0);
    let direction = PlanDirection::<Architectural>::from_vector(input).unwrap();
    assert_eq!(direction.vector(), input.normalize_or_zero());
    assert!(matches!(
        PlanDirection::<Architectural>::from_normalized(Vec2::ZERO),
        Err(GeometryError::Direction {
            source: bevy::math::InvalidDirectionError::Zero
        })
    ));
    assert!(serde_json::from_str::<PlanDirection<Architectural>>("[2,0]").is_err());
    let restored: PlanDirection<Architectural> =
        serde_json::from_str(&serde_json::to_string(&direction).unwrap()).unwrap();
    assert_eq!(restored, direction);
    assert!(matches!(direction.reflect_ref(), ReflectRef::Opaque(_)));
}

use super::*;
use bevy::{
    math::{Quat, Vec2, Vec3},
    reflect::{PartialReflect, ReflectRef},
};

#[test]
fn decoded_and_reflected_measurements_cannot_bypass_admission() {
    for input in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let wire = postcard::to_allocvec(&input).unwrap();
        assert!(postcard::from_bytes::<PositiveLength>(&wire).is_err());
    }
    for input in [Vec3::ZERO, Vec3::splat(2.0), Vec3::splat(f32::NAN)] {
        let wire = postcard::to_allocvec(&input).unwrap();
        assert!(postcard::from_bytes::<SpatialDirection<Architectural>>(&wire).is_err());
    }
    for input in [
        Quat::from_xyzw(0.0, 0.0, 0.0, 2.0),
        Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0),
    ] {
        let wire = postcard::to_allocvec(&input).unwrap();
        assert!(postcard::from_bytes::<RigidRotation>(&wire).is_err());
    }
    let rotation = RigidRotation::from_quaternion(Quat::from_rotation_y(0.37)).unwrap();
    let length = PositiveLength::from_metres(0.000_001).unwrap();
    let direction =
        SpatialDirection::<Architectural>::from_vector(Vec3::new(3.0, -2.0, 4.0)).unwrap();
    let bounds = SpatialBounds::<Architectural>::from_metres(Vec3::ZERO, Vec3::ONE).unwrap();
    let clearance = ClearanceVolume::new(bounds).unwrap();
    for value in [
        &rotation as &dyn PartialReflect,
        &length,
        &direction,
        &bounds,
        &clearance,
    ] {
        assert!(matches!(value.reflect_ref(), ReflectRef::Opaque(_)));
    }
    let mut reflected = direction;
    assert!(reflected.try_apply(&Vec3::ZERO).is_err());
    assert_eq!(reflected, direction);
    assert_eq!(
        postcard::to_allocvec(&direction).unwrap(),
        postcard::to_allocvec(&direction.vector()).unwrap()
    );
    assert_eq!(
        postcard::to_allocvec(&rotation).unwrap(),
        postcard::to_allocvec(&rotation.quaternion()).unwrap()
    );
    assert_eq!(
        postcard::to_allocvec(&length).unwrap(),
        postcard::to_allocvec(&length.metres()).unwrap()
    );
    assert_eq!(size_of::<LeafDimensions>(), size_of::<Vec3>());
    assert_eq!(size_of::<PlanDimensions>(), size_of::<Vec2>());
}

#[test]
fn checked_point_arithmetic_retains_zero_and_rejects_overflow() {
    let point = Position::<Architectural>::from_metres(Vec3::splat(f32::MAX)).unwrap();
    let delta = Displacement::<Architectural>::from_metres(Vec3::splat(f32::MAX)).unwrap();
    assert_eq!(point.translated(Displacement::ZERO).unwrap(), point);
    assert!(point.translated(delta).is_err());
    let opposite = Position::from_metres(-point.metres()).unwrap();
    assert!(opposite.displacement_to(point).is_err());
}

#[test]
fn signed_lengths_and_areas_admit_their_distinct_zero_and_negative_contracts() {
    for metres in [-3.5, -0.0, 0.0, 2.0] {
        let signed = SignedLength::from_metres(metres).unwrap();
        let wire = postcard::to_allocvec(&metres).unwrap();
        assert_eq!(postcard::to_allocvec(&signed).unwrap(), wire);
        assert_eq!(postcard::from_bytes::<SignedLength>(&wire).unwrap(), signed);
    }
    for square_metres in [0.0, 0.000_001, 3.5] {
        let area = Area::from_square_metres(square_metres).unwrap();
        let wire = postcard::to_allocvec(&square_metres).unwrap();
        assert_eq!(postcard::to_allocvec(&area).unwrap(), wire);
        assert_eq!(postcard::from_bytes::<Area>(&wire).unwrap(), area);
    }
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let wire = postcard::to_allocvec(&invalid).unwrap();
        assert!(SignedLength::from_metres(invalid).is_err());
        assert!(Area::from_square_metres(invalid).is_err());
        assert!(postcard::from_bytes::<SignedLength>(&wire).is_err());
        assert!(postcard::from_bytes::<Area>(&wire).is_err());
    }
    assert!(Area::from_square_metres(-0.01).is_err());
    let mut area = Area::ZERO;
    let mut signed = SignedLength::ZERO;
    assert!(matches!(area.reflect_ref(), ReflectRef::Opaque(_)));
    assert!(matches!(signed.reflect_ref(), ReflectRef::Opaque(_)));
    assert!(area.try_apply(&-1.0_f32).is_err());
    assert!(signed.try_apply(&f32::NAN).is_err());
    assert_eq!(area, Area::ZERO);
    assert_eq!(signed, SignedLength::ZERO);
    let mut registry = bevy::reflect::TypeRegistry::default();
    registry.register::<Area>();
    registry.register::<SignedLength>();
    assert!(registry.get(std::any::TypeId::of::<Area>()).is_some());
    assert_eq!(size_of::<Area>(), size_of::<f32>());
    assert_eq!(size_of::<SignedLength>(), size_of::<f32>());
}

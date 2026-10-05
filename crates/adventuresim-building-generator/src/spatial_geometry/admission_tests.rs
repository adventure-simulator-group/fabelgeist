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

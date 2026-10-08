use super::spatial::*;
use super::{ArchitecturalPlanProjection, ScenePlanPoint};
use crate::{city_layout::grounding::SupportElevation, scene_input::BuildingOrientation};
use adventuresim_building_generator::{DoorSpec, OpeningAssemblyId, ResolvedItemId};
use adventuresim_building_generator::{
    plan_geometry::ArchitecturalPlanPoint,
    spatial_geometry::{
        Architectural, CuboidDimensions, Displacement, LeafDimensions, PlanDirection, Position,
        Radians,
    },
};
use bevy::math::{Quat, Vec2, Vec3};

#[test]
fn building_orientation_decoding_preserves_canonical_native_values() {
    use bevy::reflect::{PartialReflect, ReflectRef};

    for radians in [-core::f32::consts::PI, -0.0, 0.0, 0.43] {
        let native = postcard::to_allocvec(&radians).unwrap();
        let admitted: BuildingOrientation = postcard::from_bytes(&native).unwrap();
        assert_eq!(admitted.yaw_radians().to_bits(), radians.to_bits());
        assert_eq!(postcard::to_allocvec(&admitted).unwrap(), native);
        assert!(matches!(admitted.reflect_ref(), ReflectRef::Opaque(_)));
    }
    for radians in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        core::f32::consts::PI,
        -core::f32::consts::TAU,
    ] {
        let native = postcard::to_allocvec(&radians).unwrap();
        assert!(postcard::from_bytes::<BuildingOrientation>(&native).is_err());
    }
    for tangent in [Vec2::new(-1.0, 0.0), Vec2::new(-1.0, -0.0)] {
        let orientation = BuildingOrientation::from_frontage_tangent(tangent).unwrap();
        assert!(orientation.is_valid());
        assert_eq!(orientation.yaw_radians(), -core::f32::consts::PI);
        let native = postcard::to_allocvec(&orientation).unwrap();
        assert_eq!(
            postcard::from_bytes::<BuildingOrientation>(&native).unwrap(),
            orientation
        );
    }
}

fn projection() -> ArchitecturalPlanProjection {
    ArchitecturalPlanProjection {
        centre: ScenePlanPoint::from_metres(Vec2::new(-17.0, 31.0)).unwrap(),
        origin: ArchitecturalPlanPoint::from_metres(Vec2::new(2.0, -3.0)).unwrap(),
        orientation: BuildingOrientation::from_radians(0.43).unwrap(),
    }
}
fn leaf<F: adventuresim_building_generator::spatial_geometry::GeometryFrame>(
    hinge: f32,
) -> DoorSpec<F> {
    DoorSpec {
        opening: OpeningAssemblyId(91),
        source: ResolvedItemId(92),
        closed_centre: Position::from_metres(Vec3::new(2.0, 1.1, -3.0)).unwrap(),
        hinge_centre: Position::from_metres(Vec3::new(2.0 + hinge, 1.1, -3.0)).unwrap(),
        size_metres: LeafDimensions::from_metres(Vec3::new(1.4, 2.1, 0.06)).unwrap(),
        closed_yaw_radians: Radians::new(-0.2).unwrap(),
        tangent: PlanDirection::from_normalized(Vec2::X).unwrap(),
        outward: PlanDirection::from_normalized(Vec2::NEG_Y).unwrap(),
        open_angle_radians: Radians::new(hinge.signum() * 1.5).unwrap(),
    }
}
#[test]
fn architectural_floor_and_collision_centre_datums_preserve_native_arithmetic() {
    let plan = projection();
    let origin = Position::<Architectural>::from_metres(Vec3::new(2.0, 4.0, -3.0)).unwrap();
    let datum = ArchitecturalFloorDatum {
        plan,
        floor: SupportElevation::from_metres(11.0).unwrap(),
    }
    .collision_centre(origin)
    .unwrap();
    let transform = datum.native_transform();
    assert_eq!(transform.translation, Vec3::new(-17.0, 15.0, 31.0));
    assert_eq!(
        datum.point(Position::ORIGIN).unwrap().metres(),
        transform.transform_point(-origin.metres())
    );
    let displacement = Displacement::<Architectural>::ZERO;
    assert_eq!(
        datum.displacement(displacement).unwrap().metres(),
        Vec3::ZERO
    );
    let point = Position::<Architectural>::from_metres(Vec3::new(-4.0, 1.5, 8.0)).unwrap();
    let roundtrip = datum
        .architectural_point(datum.point(point).unwrap())
        .unwrap();
    // Rotation's representable inverse introduces ordinary f32 rounding.
    assert!(roundtrip.metres().distance(point.metres()) < 0.00001);
    for hinge in [-0.7, 0.7] {
        let original = leaf::<Architectural>(hinge);
        let pose = datum.door(original).unwrap();
        assert_eq!(pose.leaf().opening, original.opening);
        assert_eq!(pose.leaf().source, original.source);
        assert_eq!(pose.leaf().size_metres, original.size_metres);
        assert_eq!(pose.leaf().open_angle_radians, original.open_angle_radians);
        assert_eq!(
            pose.leaf().closed_centre.metres(),
            transform.transform_point(original.closed_centre.metres() - origin.metres())
        );
        assert_eq!(
            pose.leaf().hinge_centre.metres(),
            transform.transform_point(original.hinge_centre.metres() - origin.metres())
        );
        assert_eq!(
            pose.native_rotation(),
            transform.rotation * Quat::from_rotation_y(original.closed_yaw_radians.radians())
        );
        assert!(
            datum
                .architectural_point(pose.leaf().hinge_centre)
                .unwrap()
                .metres()
                .distance(original.hinge_centre.metres())
                < 0.00001
        );
        assert!(
            (pose.leaf().horizontal_sweep_radius_metres().unwrap()
                - original.horizontal_sweep_radius_metres().unwrap())
            .abs()
                < 0.00001
        );
    }
}
#[test]
fn gate_datum_changes_only_elevation_and_preserves_both_hinges() {
    let datum = GateDatum::from_metres(-5.25).unwrap();
    for hinge in [-0.7, 0.7] {
        let original = leaf::<GateRelative>(hinge);
        let pose = datum.door(original).unwrap();
        assert_eq!(
            pose.leaf().closed_centre.metres(),
            original.closed_centre.metres() + Vec3::Y * -5.25
        );
        assert_eq!(
            pose.leaf().hinge_centre.metres(),
            original.hinge_centre.metres() + Vec3::Y * -5.25
        );
        assert_eq!(pose.leaf().tangent.vector(), original.tangent.vector());
        assert_eq!(pose.leaf().outward.vector(), original.outward.vector());
        for (scene, gate) in [
            (pose.leaf().closed_centre, original.closed_centre),
            (pose.leaf().hinge_centre, original.hinge_centre),
        ] {
            assert!(
                datum
                    .gate_point(scene)
                    .unwrap()
                    .metres()
                    .distance(gate.metres())
                    < 0.00001
            );
        }
        assert_eq!(pose.leaf().size_metres, original.size_metres);
        assert_eq!(pose.leaf().opening, original.opening);
        assert_eq!(pose.leaf().source, original.source);
        assert_eq!(
            pose.leaf().horizontal_sweep_radius_metres().unwrap(),
            original.horizontal_sweep_radius_metres().unwrap()
        );
    }
    assert!(GateDatum::from_metres(f32::NAN).is_err());
    let high = GateDatum::from_metres(f32::MAX).unwrap();
    let point = Position::<GateRelative>::from_metres(Vec3::new(0.0, f32::MAX, 0.0)).unwrap();
    assert!(high.point(point).is_err());
    assert!(LeafDimensions::from_metres(CuboidDimensions::ZERO.metres()).is_err());
}

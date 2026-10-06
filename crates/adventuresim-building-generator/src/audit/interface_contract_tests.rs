use super::*;
use crate::spatial_geometry::{CuboidDimensions, Position, Radians, SpatialBounds};
use crate::{BuildingArchetype, ResolvedItemId};

#[test]
fn bonding_preserves_tolerated_gap_planes_and_zero_overlap() {
    let plan = crate::generate(&crate::BuildingProgram::fixture(
        BuildingArchetype::TownHouse,
        42,
    ))
    .unwrap();
    let mut left = plan.resolved_geometry.solids[0].clone();
    left.centre = Position::ORIGIN;
    left.size = CuboidDimensions::from_metres(Vec3::splat(2.0)).unwrap();
    left.yaw_radians = Radians::ZERO;
    left.crossfall_radians = Radians::ZERO;
    left.longfall_radians = Radians::ZERO;
    let mut right = left.clone();
    right.id = ResolvedItemId(794);
    right.centre = Position::from_metres(Vec3::new(2.02, 0.0, 0.0)).unwrap();
    let contact = bonded_interface_metrics(&left, &right).unwrap().unwrap();
    assert!(contact.contact_min.metres().x > contact.contact_max.metres().x);
    assert_eq!(contact.penetration, SignedLength::ZERO);
    assert_eq!(contact.area.square_metres(), 4.0);
    assert_eq!(
        resolved_plan_overlap_area(&left, &right).unwrap(),
        MeasuredArea::ZERO
    );
    let bond = crate::JunctionBond {
        id: ResolvedItemId(795),
        owners: [left.owner, right.owner],
        bounds: SpatialBounds::from_metres(Vec3::new(0.9, -1.0, -1.0), Vec3::new(1.1, 1.0, 1.0))
            .unwrap(),
        minimum_interface_area_square_metres: 4.0,
        maximum_penetration_metres: 0.0,
    };
    assert!(contact.fits(&bond));
    for bounds in [
        SpatialBounds::from_metres(Vec3::new(0.9, -1.0, -1.0), Vec3::new(0.99, 1.0, 1.0)).unwrap(),
        SpatialBounds::from_metres(Vec3::new(1.04, -1.0, -1.0), Vec3::new(1.1, 1.0, 1.0)).unwrap(),
    ] {
        assert!(
            !contact.fits(&crate::JunctionBond { bounds, ..bond }),
            "a bond must contain both measured gap planes within tolerance: {bounds:?}"
        );
    }
    right.centre = Position::from_metres(Vec3::new(2.03, 0.0, 0.0)).unwrap();
    assert!(bonded_interface_metrics(&left, &right).unwrap().is_none());
    right.centre = Position::from_metres(Vec3::new(1.0, 0.0, 0.0)).unwrap();
    right.size = CuboidDimensions::from_metres(Vec3::new(0.0, 2.0, 2.0)).unwrap();
    let contact = bonded_interface_metrics(&left, &right).unwrap().unwrap();
    assert_eq!(contact.penetration, SignedLength::ZERO);
    assert_eq!(contact.area.square_metres(), 4.0);
}

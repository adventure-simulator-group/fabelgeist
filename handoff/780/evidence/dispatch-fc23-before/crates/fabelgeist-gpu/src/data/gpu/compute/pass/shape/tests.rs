use super::*;
use crate::prelude::DispatchOccupancy;
use serde::Deserialize;

#[derive(Deserialize)]
struct NativeCoveringCase {
    shape: [u32; 3],
    items: u32,
    groups: [u32; 3],
}
#[derive(Deserialize)]
struct NativeOccupancyCase {
    groups: [u32; 3],
    empty: bool,
}
#[derive(Deserialize)]
struct NativeCases {
    coverage: Vec<NativeCoveringCase>,
    occupancy: Vec<NativeOccupancyCase>,
}

#[test]
fn covering_and_empty_grids_match_frozen_native_kernel_behavior() {
    let cases: NativeCases = serde_json::from_str(include_str!("native-cases.json")).unwrap();
    for case in cases.coverage {
        let shape = WorkgroupShape::try_from(case.shape).unwrap();
        assert_eq!(
            shape.covering_x(InvocationCount::from(case.items)),
            WorkgroupGrid::from(case.groups)
        );
    }
    for case in cases.occupancy {
        let expected = if case.empty {
            DispatchOccupancy::Empty
        } else {
            DispatchOccupancy::Populated
        };
        assert_eq!(WorkgroupGrid::from(case.groups).occupancy(), expected);
    }
}

#[test]
fn zero_declarations_retain_each_rejected_native_axis() {
    for axes in [[0, 1, 1], [64, 0, 1], [64, 1, 0], [0, 0, 0]] {
        assert_eq!(
            WorkgroupShape::try_from(axes).unwrap_err(),
            WorkgroupShapeError(axes)
        );
    }
    let error = WorkgroupShape::try_from([64, 0, 1]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "workgroup size [64, 0, 1] has a zero dimension"
    );
}

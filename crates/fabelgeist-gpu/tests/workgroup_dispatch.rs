use fabelgeist_gpu::prelude::*;

// Serialized original-implementation outputs acquire their roles at admission.
#[derive(serde::Deserialize)]
struct CoverageCase {
    shape: [u32; 3],
    items: u32,
    groups: [u32; 3],
}
#[derive(serde::Deserialize)]
struct OccupancyCase {
    groups: [u32; 3],
    empty: bool,
}
#[derive(serde::Deserialize)]
struct OriginalCases {
    coverage: Vec<CoverageCase>,
    occupancy: Vec<OccupancyCase>,
}

#[test]
fn coverage_and_empty_grids_match_original_dispatch_arithmetic() {
    let original: OriginalCases =
        serde_json::from_str(include_str!("fixtures/kernel_dispatch.json")).unwrap();
    for case in original.coverage {
        let shape = WorkgroupShape::try_from(case.shape).unwrap();
        let items = InvocationCount::from(case.items);
        assert_eq!(shape.covering_x(items), WorkgroupGrid::from(case.groups));
    }
    for case in original.occupancy {
        let expected = if case.empty {
            DispatchOccupancy::Empty
        } else {
            DispatchOccupancy::Populated
        };
        assert_eq!(WorkgroupGrid::from(case.groups).occupancy(), expected);
    }
}

#[test]
fn zero_declarations_are_rejected_on_every_axis_without_rejecting_large_words() {
    for axes in [[0, 1, 1], [1, 0, 1], [1, 1, 0], [0, 0, 0]] {
        let cause = WorkgroupShape::try_from(axes).unwrap_err();
        assert_eq!(
            cause.to_string(),
            format!("workgroup size {axes:?} has a zero dimension")
        );
    }
    let shape = WorkgroupShape::try_from([u32::MAX; 3]).unwrap();
    assert_eq!(
        shape.covering_x(InvocationCount::from(u32::MAX)),
        WorkgroupGrid::from([1, 1, 1])
    );
}

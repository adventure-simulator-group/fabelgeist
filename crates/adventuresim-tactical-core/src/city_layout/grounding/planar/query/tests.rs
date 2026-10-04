use super::*;

#[test]
fn broad_phase_preserves_exhaustive_intersections_and_input_order() {
    let bounds: Vec<_> = (0..100)
        .map(|i| {
            let centre = DVec2::new((i % 10) as f64 * 5.0, (i / 10) as f64 * 5.0);
            PlanarBounds::from_points([centre - DVec2::ONE, centre + DVec2::ONE]).unwrap()
        })
        .collect();
    let index = PlanarQueryIndex::from_bounds(bounds.clone());
    for centre in [
        DVec2::ZERO,
        DVec2::splat(20.0),
        DVec2::new(21.0, 16.0),
        DVec2::splat(100.0),
    ] {
        for half in [
            DVec2::ZERO,
            DVec2::splat(0.001),
            DVec2::new(30.0, 0.000001),
            DVec2::splat(50.0),
        ] {
            let query = PlanarBounds::from_points([centre - half, centre + half]).unwrap();
            let expected: Vec<_> = bounds
                .iter()
                .enumerate()
                .filter_map(|(i, b)| b.intersects(query).then_some(i))
                .collect();
            assert_eq!(index.intersections(query), expected);
        }
    }
    let local = PlanarBounds::from_points([DVec2::new(20.0, 20.0)]).unwrap();
    assert_eq!(index.intersections(local).len(), 1);
}

#[test]
fn closed_bounds_keep_touching_regions_and_overlapping_source_references() {
    let first = PlanarBounds::from_points([DVec2::ZERO, DVec2::ONE]).unwrap();
    let second = PlanarBounds::from_points([DVec2::ONE, DVec2::splat(2.0)]).unwrap();
    let index = PlanarQueryIndex::from_bounds(vec![second, first, first]);
    assert_eq!(
        index.intersections(PlanarBounds::from_points([DVec2::ONE]).unwrap()),
        [0, 1, 2]
    );
    assert!(
        PlanarQueryIndex::from_bounds(Vec::new())
            .intersections(first)
            .is_empty()
    );
}

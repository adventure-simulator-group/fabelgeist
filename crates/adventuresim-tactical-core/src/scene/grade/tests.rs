use super::*;
use crate::scene::SceneTerrain;

#[test]
fn diagonal_triangle_grade_is_bounded_without_changing_valid_axis_relief() {
    let mut terrain = SceneTerrain::from_heightmap(
        3,
        3,
        1.0,
        vec![0.0, 0.65, 1.3, 0.65, 1.3, 1.95, 1.3, 1.95, 2.6],
    )
    .unwrap();
    terrain
        .constrain_max_grade(crate::scene::grade::TerrainGradeLimit::from_ratio(0.65).unwrap())
        .unwrap();
    let source = terrain.sampled_geographic_surface().unwrap();
    for triangle in source.triangles() {
        let [a, b, c] = triangle;
        let n = (c - a).cross(b - a).normalize();
        assert!(n.x.hypot(n.z) / n.y <= 0.650_01, "{triangle:?}");
    }
    for grade in [0.0, 0.60, 0.65] {
        let mut terrain = SceneTerrain::new(8, 8, 1.0, |p| p.x * grade).unwrap();
        let before = terrain.clone();
        terrain
            .constrain_max_grade(crate::scene::grade::TerrainGradeLimit::from_ratio(0.65).unwrap())
            .unwrap();
        assert_eq!(terrain, before);
    }
}

#[test]
fn neighboring_triangles_converge_and_repair_is_deterministic() {
    let original = vec![
        0.0, 0.0, 6.0, 6.0, 0.0, 0.0, 6.0, 7.0, 0.0, 0.0, 7.0, 7.0, 0.0, 0.0, 6.0, 6.0,
    ];
    let mut a = original.clone();
    let mut b = original;
    constrain(&mut a, 4, 4, 1.0, 0.65).unwrap();
    constrain(&mut b, 4, 4, 1.0, 0.65).unwrap();
    assert_eq!(a, b);
    let measured = worst(
        &a.iter().map(|h| f64::from(*h)).collect::<Vec<_>>(),
        4,
        4,
        None,
    );
    assert!(measured.rise_metres <= 0.650_01);
    assert!(a.iter().all(|h| *h >= 0.0 && *h <= 7.0));
}

#[test]
fn invalid_grade_request_preserves_samples() {
    let mut heights = vec![0.0, 1.0, 2.0, 3.0];
    let before = heights.clone();
    assert_eq!(
        constrain(&mut heights, 2, 2, 1.0, f32::NAN),
        Err(TerrainGradeError::InvalidInput)
    );
    assert_eq!(heights, before);
}

#[test]
fn dirty_sweeps_preserve_the_complete_unconditional_projection_result() {
    let width = 17;
    let depth = 13;
    let original: Vec<_> = (0..width * depth)
        .map(|i| {
            let x = (i % width) as f64;
            let z = (i / width) as f64;
            if x > 8.0 {
                6.0 + z * 0.2
            } else {
                x * 0.5 - z * 0.4
            }
        })
        .collect();
    let mut incremental = original.clone();
    let mut unconditional = original;
    precondition(&mut incremental, width, depth, 0.65);
    precondition(&mut unconditional, width, depth, 0.65);
    let mut dirty = vec![true; (width - 1) * (depth - 1)];
    for pass in 0..64 {
        repair_sweep(
            &mut incremental,
            &mut dirty,
            [width, depth],
            0.65,
            SweepDirection::for_pass(pass),
        );
        let mut all = vec![true; dirty.len()];
        repair_sweep(
            &mut unconditional,
            &mut all,
            [width, depth],
            0.65,
            SweepDirection::for_pass(pass),
        );
        assert_eq!(
            worst(&incremental, width, depth, None).rise_metres <= 0.650_001,
            worst(&incremental, width, depth, Some(&dirty)).rise_metres <= 0.650_001,
            "convergence decision changed at sweep {pass}"
        );
        assert_eq!(
            incremental, unconditional,
            "physical samples changed at sweep {pass}"
        );
    }
}

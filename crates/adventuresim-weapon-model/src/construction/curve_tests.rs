//! Curve-budget assertions migrated from the browser constructor tests.
use super::*;

#[test]
fn adaptive_curves_honor_chord_and_deviation_budgets_without_redundant_neighbors() {
    let evaluate = |t: f64| [t, 0.18 * (std::f64::consts::TAU * t).sin()];
    let quality = CurveQuality {
        minimum_segments: 2,
        max_chord: 0.045,
        max_deviation: 0.0008,
    };
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let points = adaptive_curve(evaluate, quality, detail);
        assert_eq!(points[0], evaluate(0.0));
        assert_eq!(*points.last().unwrap(), evaluate(1.0));
        for pair in points.windows(2) {
            let chord = (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]);
            assert!(chord > 1e-8 && chord <= detail.error(quality.max_chord) * 1.001);
            let middle = evaluate((pair[0][0] + pair[1][0]) / 2.0);
            assert!(
                point_segment_distance(middle, pair[0], pair[1])
                    <= detail.error(quality.max_deviation) * 1.001
            );
        }
    }
}

#[test]
fn cubic_curves_preserve_corners_and_resolve_the_continuous_curve() {
    for points in [
        [[0.0, 0.0], [0.1, 0.2], [0.3, -0.2], [0.4, 0.0]],
        [[0.0, 0.0], [0.0, 0.0], [0.2, 0.1], [0.2, 0.1]],
    ] {
        let quality = CurveQuality {
            minimum_segments: 3,
            max_chord: 0.015,
            max_deviation: 0.0003,
        };
        let sample = cubic_bezier(points, quality, Detail::Medium);
        assert_eq!(sample[0], points[0]);
        assert_eq!(*sample.last().unwrap(), points[3]);
        for i in 0..=256 {
            let t = i as f64 / 256.0;
            let u = 1.0 - t;
            let p = std::array::from_fn(|a| {
                u * u * u * points[0][a]
                    + 3.0 * u * u * t * points[1][a]
                    + 3.0 * u * t * t * points[2][a]
                    + t * t * t * points[3][a]
            });
            let nearest = sample
                .windows(2)
                .map(|s| point_segment_distance(p, s[0], s[1]))
                .fold(f64::INFINITY, f64::min);
            assert!(nearest <= quality.max_deviation * 1.08);
        }
    }
}

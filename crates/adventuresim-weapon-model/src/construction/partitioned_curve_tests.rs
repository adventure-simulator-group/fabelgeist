use super::*;

#[test]
fn cubic_intersections_include_crossings_and_tangencies_once() {
    // (t-.2)(t-.5)(t-.8), (t-.5)^2, and (t-.5)^3 in Bernstein form.
    for (values, expected) in [
        ([-0.08, 0.14, -0.14, 0.08], vec![0.2, 0.5, 0.8]),
        ([0.25, -1.0 / 12.0, -1.0 / 12.0, 0.25], vec![0.5]),
        ([-0.125, 0.125, -0.125, 0.125], vec![0.5]),
        ([0.0; 4], vec![0.0, 1.0]),
    ] {
        let mut found = roots(values);
        found.sort_by(f64::total_cmp);
        assert_eq!(found.len(), expected.len(), "{values:?}: {found:?}");
        for (a, b) in found.into_iter().zip(expected) {
            assert!((a - b).abs() < 1e-12);
        }
    }
}

#[test]
fn sampled_boundary_keeps_simultaneous_crossings_and_ignores_inactive_cuts() {
    let controls = [
        [-0.04, 0.0],
        [-0.02, 0.1 / 3.0],
        [0.02, 0.2 / 3.0],
        [0.04, 0.1],
    ];
    let quality = CurveQuality {
        minimum_segments: 3,
        max_chord: 0.02,
        max_deviation: 0.00005,
    };
    let inactive = PlanarCut::Transverse {
        start: [0.01, 0.0],
        end: [0.01, 0.01],
    };
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        assert_eq!(
            partitioned_cubic(controls, quality, detail, &[]),
            partitioned_cubic(controls, quality, detail, &[inactive])
        );
        let sampled = partitioned_cubic(
            controls,
            quality,
            detail,
            &[
                PlanarCut::Axial(0.05),
                PlanarCut::Transverse {
                    start: [0.0, 0.0],
                    end: [0.0, 0.1],
                },
            ],
        );
        assert_eq!(sampled.first(), Some(&controls[0]));
        assert_eq!(sampled.last(), Some(&controls[3]));
        assert_eq!(
            sampled
                .iter()
                .filter(|p| p[0].abs() < 1e-14 && (p[1] - 0.05).abs() < 1e-14)
                .count(),
            1
        );
        assert!(sampled.windows(2).all(|pair| pair[1][1] > pair[0][1]));
    }
}

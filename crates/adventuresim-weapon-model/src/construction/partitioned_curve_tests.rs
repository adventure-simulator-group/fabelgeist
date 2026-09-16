use super::*;

#[test]
fn near_terminal_cut_retains_exact_authored_endpoint() {
    for scale in [0.001, 1.0, 1000.0] {
        let controls = [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, scale]];
        let cuts = [PlanarCut::Axial(scale * (1.0 - 96.0 * f64::EPSILON))];
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let sampled = partitioned_cubic(
                controls,
                CurveQuality {
                    minimum_segments: 4,
                    max_chord: scale / 8.0,
                    max_deviation: scale / 1000.0,
                },
                detail,
                &cuts,
            );
            assert_eq!(sampled.first(), Some(&controls[0]));
            assert_eq!(sampled.last(), Some(&controls[3]));
            assert!(sampled.windows(2).all(|pair| pair[0][1] < pair[1][1]));
        }
    }
}

#[test]
fn conditioned_cut_equations_keep_one_physical_junction() {
    let controls = [
        [0.0075, 0.032],
        [0.0072, 0.220],
        [0.0047, 0.430],
        [0.0034, 0.608],
    ];
    let junction = [0.006650813443345097, 0.217];
    let cuts = [
        PlanarCut::Axial(junction[1]),
        PlanarCut::Transverse {
            start: [0.006765505129596703, 0.19999999999999998],
            end: junction,
        },
        PlanarCut::Transverse {
            start: junction,
            end: [0.006021600741484064, 0.3],
        },
    ];
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let sampled = partitioned_cubic(
            controls,
            CurveQuality {
                minimum_segments: 4,
                max_chord: 0.008,
                max_deviation: 0.00015,
            },
            detail,
            &cuts,
        );
        assert_eq!(sampled.first(), Some(&controls[0]));
        assert_eq!(sampled.last(), Some(&controls[3]));
        assert_eq!(
            sampled
                .iter()
                .filter(|p| (p[1] - junction[1]).abs() < 1e-12)
                .count(),
            1
        );
        let retained = *sampled
            .iter()
            .find(|p| (p[1] - junction[1]).abs() < 1e-12)
            .unwrap();
        for cut in cuts {
            assert!(
                cut.distance(retained).abs() <= ROOT_ROUNDOFF_ULPS * f64::EPSILON * junction[1]
            );
        }
        assert!(
            sampled
                .windows(2)
                .all(|pair| pair[1][1] - pair[0][1] > 1e-9)
        );
    }
}

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

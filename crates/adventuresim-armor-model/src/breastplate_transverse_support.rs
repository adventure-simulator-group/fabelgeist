//! Physical ellipse floors converted to main-meridian inequalities using retained a/C.
use crate::breastplate_qp::{LinearConstraint, QpOptions, QpSolution, solve_dense_qp};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TransverseInput {
    pub source_row: usize,
    pub source_origin: &'static str,
    pub xy: [f64; 2],
    pub floor_m: f64,
    pub radius_m: f64,
    pub center_m: f64,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) enum Relation {
    FrontFloor { cosine: f64, required_front_m: f64 },
    SideSatisfied,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct TransverseSupport {
    pub input: TransverseInput,
    pub relation: Relation,
}
#[derive(Debug, Serialize)]
pub(crate) enum TransverseError {
    InvalidInput {
        row: usize,
    },
    OutsideRadius {
        row: usize,
    },
    IncompatibleSide {
        row: usize,
    },
    NonfiniteBound {
        row: usize,
    },
    OutsideMainHeight {
        row: usize,
    },
    InvalidMainDomain,
    IncompatibleFixedEndpoint {
        row: usize,
        required_front_m: f64,
        fixed_front_m: f64,
    },
}
impl std::fmt::Display for TransverseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl TransverseSupport {
    pub(crate) fn new(input: TransverseInput) -> Result<Self, TransverseError> {
        let row = input.source_row;
        if input
            .xy
            .iter()
            .chain([&input.floor_m, &input.radius_m, &input.center_m])
            .any(|v| !v.is_finite())
            || input.radius_m <= 0.
        {
            return Err(TransverseError::InvalidInput { row });
        }
        let x = input.xy[0].abs();
        let a = input.radius_m;
        if x > a {
            return Err(TransverseError::OutsideRadius { row });
        }
        let relation = if x == a {
            if input.floor_m > input.center_m {
                return Err(TransverseError::IncompatibleSide { row });
            }
            Relation::SideSatisfied
        } else {
            // Factored radicand avoids cancellation near the side; no clamping.
            let cosine = ((a - x) * (a + x)).sqrt() / a;
            let required_front_m = input.center_m + (input.floor_m - input.center_m) / cosine;
            if !required_front_m.is_finite() || !cosine.is_finite() || cosine <= 0. {
                return Err(TransverseError::NonfiniteBound { row });
            }
            Relation::FrontFloor {
                cosine,
                required_front_m,
            }
        };
        Ok(Self { input, relation })
    }
    pub(crate) fn constraint(
        &self,
        heights: [f64; 2],
        endpoints: [f64; 2],
    ) -> Result<Option<LinearConstraint>, TransverseError> {
        let row = self.input.source_row;
        let y = self.input.xy[1];
        if heights.iter().chain(&endpoints).any(|v| !v.is_finite()) || heights[0] >= heights[1] {
            return Err(TransverseError::InvalidMainDomain);
        }
        if y < heights[0] || y > heights[1] {
            return Err(TransverseError::OutsideMainHeight { row });
        }
        let Relation::FrontFloor {
            required_front_m, ..
        } = self.relation
        else {
            return Ok(None);
        };
        for i in 0..2 {
            if y == heights[i] && required_front_m > endpoints[i] {
                return Err(TransverseError::IncompatibleFixedEndpoint {
                    row,
                    required_front_m,
                    fixed_front_m: endpoints[i],
                });
            }
        }
        let b = crate::breastplate_whole_front::basis((y - heights[0]) / (heights[1] - heights[0]));
        Ok(Some(LinearConstraint {
            coefficients: vec![b[1], b[2]],
            value: required_front_m - b[0] * endpoints[0] - b[3] * endpoints[1],
        }))
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum GuideSolvePath {
    OriginalOptimumSatisfiesAddedBounds,
    AugmentedSolve,
}
pub(crate) fn solve_augmented(
    h: &[Vec<f64>],
    rhs: &[f64],
    bounds: &mut Vec<LinearConstraint>,
    added: Vec<LinearConstraint>,
) -> Result<(QpSolution, GuideSolvePath), String> {
    let mut original = solve_dense_qp(h, rhs, &[], bounds, QpOptions::default())
        .map_err(|e| format!("Whole front guide QP: {e:?}"))?;
    let inactive = added.iter().all(|r| {
        r.coefficients
            .iter()
            .zip(&original.coefficients)
            .map(|(a, b)| a * b)
            .sum::<f64>()
            >= r.value
    });
    let added_count = added.len();
    bounds.extend(added);
    if inactive {
        // A feasible optimum of the superset feasible region remains optimal
        // after restriction. Its existing KKT witness extends with zero duals.
        original
            .lower_bound_multipliers
            .extend(std::iter::repeat_n(0., added_count));
        Ok((
            original,
            GuideSolvePath::OriginalOptimumSatisfiesAddedBounds,
        ))
    } else {
        let result = solve_dense_qp(h, rhs, &[], bounds, QpOptions::default())
            .map_err(|e| format!("Transverse whole front guide QP: {e:?}"))?;
        Ok((result, GuideSolvePath::AugmentedSolve))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(x: f64, floor: f64) -> TransverseInput {
        TransverseInput {
            source_row: 0,
            source_origin: "synthetic-floor",
            xy: [x, 1.2],
            floor_m: floor,
            radius_m: 0.2,
            center_m: -0.05,
        }
    }
    #[test]
    fn bound_matches_actual_ellipse_at_fixed_radius_and_center() {
        let row = TransverseSupport::new(input(0.16, 0.1)).unwrap();
        let Relation::FrontFloor {
            cosine,
            required_front_m,
        } = row.relation
        else {
            panic!()
        };
        assert!((cosine - 0.6).abs() < 1e-14);
        assert!((-0.05 + (required_front_m + 0.05) * cosine - 0.1).abs() < 1e-14);
        assert!(-0.05 + (required_front_m - 0.001 + 0.05) * cosine < 0.1);
    }
    #[test]
    fn side_and_invalid_inputs_are_classified_without_denominator_clamps() {
        assert!(matches!(
            TransverseSupport::new(input(0.2, -0.05)).unwrap().relation,
            Relation::SideSatisfied
        ));
        assert!(matches!(
            TransverseSupport::new(input(-0.2, -0.04)),
            Err(TransverseError::IncompatibleSide { .. })
        ));
        assert!(matches!(
            TransverseSupport::new(input(0.2000000001, 0.)),
            Err(TransverseError::OutsideRadius { .. })
        ));
        for radius in [0., -1., f64::NAN] {
            let mut x = input(0., 0.);
            x.radius_m = radius;
            assert!(matches!(
                TransverseSupport::new(x),
                Err(TransverseError::InvalidInput { .. })
            ));
        }
        let near = TransverseSupport::new(input(0.2 - 1e-12, 0.1)).unwrap();
        let Relation::FrontFloor {
            required_front_m, ..
        } = near.relation
        else {
            panic!()
        };
        assert!(required_front_m > 1000.);
    }
    #[test]
    fn fixed_endpoints_and_main_domain_are_explicit_contracts() {
        let row = TransverseSupport::new(input(0.16, 0.1)).unwrap();
        for heights in [[1.2, 1.2], [1.4, 1.2], [f64::NAN, 1.4]] {
            assert!(matches!(
                row.constraint(heights, [0.1, 0.1]),
                Err(TransverseError::InvalidMainDomain)
            ));
        }
        assert!(matches!(
            row.constraint([1.2, 1.4], [0.1, 0.1]),
            Err(TransverseError::IncompatibleFixedEndpoint { .. })
        ));
        assert!(matches!(
            row.constraint([1.3, 1.4], [0.1, 0.1]),
            Err(TransverseError::OutsideMainHeight { .. })
        ));
    }
    #[test]
    fn inactive_constraints_preserve_exact_optimum_and_active_bounds_are_enforced() {
        let h = vec![vec![1., 0.], vec![0., 1.]];
        let rhs = [2., 3.];
        let mut bounds = vec![];
        let (solution, path) = solve_augmented(
            &h,
            &rhs,
            &mut bounds,
            vec![LinearConstraint {
                coefficients: vec![1., 0.],
                value: 1.,
            }],
        )
        .unwrap();
        assert!(matches!(
            path,
            GuideSolvePath::OriginalOptimumSatisfiesAddedBounds
        ));
        assert_eq!(solution.coefficients, vec![2., 3.]);
        assert_eq!(solution.lower_bound_multipliers, vec![0.]);
        let (solution, path) = solve_augmented(
            &h,
            &rhs,
            &mut vec![],
            vec![LinearConstraint {
                coefficients: vec![1., 0.],
                value: 4.,
            }],
        )
        .unwrap();
        assert!(matches!(path, GuideSolvePath::AugmentedSolve));
        assert!((solution.coefficients[0] - 4.).abs() < 1e-8);
    }
    #[test]
    fn crown20_boundary_floor_regression_keeps_objective_and_fixed_endpoints() {
        // Recorded failing main-region boundary, not a skin interpolation target.
        let heights = [1.0360580682754517, 1.4295444488525706];
        let endpoints = [0.08154062591450492, -0.01148211536652989];
        let h = vec![
            vec![0.08538076702325272, 0.06403557535441],
            vec![0.06403557535441, 0.08538076702325269],
        ];
        let rhs = [0.011510468011318685, 0.012448914576995426];
        let mut bounds = vec![
            LinearConstraint {
                coefficients: vec![2., -1.],
                value: endpoints[0],
            },
            LinearConstraint {
                coefficients: vec![-1., 2.],
                value: endpoints[1],
            },
        ];
        let original = solve_dense_qp(&h, &rhs, &[], &bounds, QpOptions::default()).unwrap();
        let support = TransverseSupport::new(TransverseInput {
            source_row: 1487,
            source_origin: "new-boundary",
            xy: [-0.18060845136642456, 1.3727585077285767],
            floor_m: -0.04121788218617439,
            radius_m: 0.20351171579495145,
            center_m: -0.0977277109118776,
        })
        .unwrap();
        let row = support.constraint(heights, endpoints).unwrap().unwrap();
        let old_value = row
            .coefficients
            .iter()
            .zip(&original.coefficients)
            .map(|(a, b)| a * b)
            .sum::<f64>();
        assert!(old_value < row.value);
        let (corrected, path) = solve_augmented(&h, &rhs, &mut bounds, vec![row.clone()]).unwrap();
        assert!(matches!(path, GuideSolvePath::AugmentedSolve));
        let value = row
            .coefficients
            .iter()
            .zip(&corrected.coefficients)
            .map(|(a, b)| a * b)
            .sum::<f64>();
        assert!((value - row.value).abs() < 1e-8);
        assert!((corrected.coefficients[0] - original.coefficients[0] - 0.002571513).abs() < 1e-8);
        assert!((corrected.coefficients[1] - original.coefficients[1] - 0.005143027).abs() < 1e-8);
        // Zero change in fixed endpoint coefficients is built into each row.
        assert!(
            support
                .constraint([heights[0], support.input.xy[1]], endpoints)
                .is_err()
        );
    }
}

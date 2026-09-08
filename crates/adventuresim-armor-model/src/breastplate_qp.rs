//! Small, dependency-free, dense convex quadratic fit.
//!
//! Minimizes `0.5 * c^T H c - rhs^T c`, subject to exact equalities and
//! `a^T c >= value` lower bounds. H must be symmetric positive definite.
//! Equalities are rank-reduced first; an explicit orthonormal nullspace keeps
//! them exact. Nonnegative dual coordinate minimization handles the remaining
//! inequalities, including releasing previously active and dependent bounds.
//! Periodic pair and rank-safe block active-set minimizations accelerate
//! correlated rows and release dependent active multipliers. They do not
//! change the objective, constraints, or certificate.
//! No pivot clamping, penalty replacement of equalities, or best-effort export
//! occurs. Iteration exhaustion is an error, not proof of infeasibility.
//!
//! Constraint residual tolerances use rows normalized to Euclidean norm one.
//! Thus scaling a complete constraint by a positive constant does not change
//! the numerical contract. Near-dependent equality classification is subject
//! to `rank_tolerance`; callers should treat a rank conflict as a diagnostic.

#[path = "breastplate_qp_certificate.rs"]
mod certificate;
#[path = "breastplate_qp_dual.rs"]
mod dual;
#[path = "breastplate_qp_preparation.rs"]
mod preparation;

use dual::{polish_dual_block, polish_dual_pairs};

#[derive(Clone, Debug)]
pub struct LinearConstraint {
    pub coefficients: Vec<f64>,
    pub value: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct QpOptions {
    pub max_sweeps: usize,
    pub rank_tolerance: f64,
    pub equality_tolerance: f64,
    pub primal_tolerance: f64,
    pub complementarity_tolerance: f64,
    pub stationarity_tolerance: f64,
}

impl Default for QpOptions {
    fn default() -> Self {
        Self {
            max_sweeps: 10_000,
            rank_tolerance: 1e-11,
            equality_tolerance: 1e-9,
            primal_tolerance: 1e-8,
            complementarity_tolerance: 1e-8,
            stationarity_tolerance: 1e-9,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct QpDiagnostics {
    pub sweeps: usize,
    pub equality_rank: usize,
    pub redundant_equalities: usize,
    pub projected_zero_bounds: usize,
    pub active_lower_bounds: usize,
    pub max_equality_residual: f64,
    pub max_lower_bound_violation: f64,
    pub max_complementarity: f64,
    pub projected_stationarity_residual: f64,
    /// Largest feasible dual-coordinate step, measured in Hessian norm.
    pub max_projected_update: f64,
    pub objective: f64,
    /// Accepted two-coordinate dual minimizations (no certificate relaxation).
    pub dual_pair_steps: usize,
    /// Rank-safe multi-coordinate Newton or nullspace transfer steps.
    pub dual_block_steps: usize,
    pub dual_dependent_transfers: usize,
}

#[derive(Clone, Debug)]
pub struct QpSolution {
    pub coefficients: Vec<f64>,
    /// Multipliers correspond to the ORIGINAL (unnormalized) lower rows.
    pub lower_bound_multipliers: Vec<f64>,
    pub diagnostics: QpDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QpErrorKind {
    InvalidInput,
    NotPositiveDefinite,
    InconsistentEqualities,
    EqualityBoundConflict,
    NumericalFailure,
    NoConvergence,
}

#[derive(Clone, Debug)]
pub struct QpError {
    pub kind: QpErrorKind,
    pub message: String,
    pub diagnostics: QpDiagnostics,
}

fn error(kind: QpErrorKind, message: impl Into<String>, diagnostics: &QpDiagnostics) -> QpError {
    QpError {
        kind,
        message: message.into(),
        diagnostics: diagnostics.clone(),
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn norm(a: &[f64]) -> f64 {
    a.iter().fold(0.0_f64, |n, x| n.hypot(*x))
}
fn max_abs(a: &[f64]) -> f64 {
    a.iter().fold(0.0_f64, |n, x| n.max(x.abs()))
}
fn axpy(target: &mut [f64], factor: f64, source: &[f64]) {
    for (a, b) in target.iter_mut().zip(source) {
        *a += factor * b;
    }
}
fn matvec(matrix: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    matrix.iter().map(|row| dot(row, x)).collect()
}

fn cholesky(matrix: &[Vec<f64>], diagnostics: &QpDiagnostics) -> Result<Vec<Vec<f64>>, QpError> {
    let n = matrix.len();
    let mut l = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let value = matrix[i][j] - dot(&l[i][..j], &l[j][..j]);
            if i == j {
                if !value.is_finite() || value <= 0.0 {
                    return Err(error(
                        QpErrorKind::NotPositiveDefinite,
                        format!(
                            "Hessian factorization has nonpositive/nonfinite pivot {i}: {value}"
                        ),
                        diagnostics,
                    ));
                }
                l[i][j] = value.sqrt();
            } else {
                l[i][j] = value / l[j][j];
                if !l[i][j].is_finite() {
                    return Err(error(
                        QpErrorKind::NumericalFailure,
                        "Nonfinite Cholesky factor",
                        diagnostics,
                    ));
                }
            }
        }
    }
    Ok(l)
}

fn triangular_solve(l: &[Vec<f64>], rhs: &[f64]) -> Vec<f64> {
    let n = rhs.len();
    let mut x = vec![0.0; n];
    for i in 0..n {
        x[i] = (rhs[i] - dot(&l[i][..i], &x[..i])) / l[i][i];
    }
    for i in (0..n).rev() {
        let rest: f64 = (i + 1..n).map(|j| l[j][i] * x[j]).sum();
        x[i] = (x[i] - rest) / l[i][i];
    }
    x
}

#[derive(Clone)]
struct Row {
    a: Vec<f64>,
    b: f64,
    original_norm: f64,
}

fn normalized_rows(
    rows: &[LinearConstraint],
    n: usize,
    diagnostics: &QpDiagnostics,
) -> Result<Vec<Row>, QpError> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            if row.coefficients.len() != n
                || !row.value.is_finite()
                || row.coefficients.iter().any(|x| !x.is_finite())
            {
                return Err(error(
                    QpErrorKind::InvalidInput,
                    format!("Invalid constraint row {i}"),
                    diagnostics,
                ));
            }
            let length = norm(&row.coefficients);
            if !length.is_finite() {
                return Err(error(
                    QpErrorKind::InvalidInput,
                    format!("Constraint norm overflow at row {i}"),
                    diagnostics,
                ));
            }
            let scale = if length == 0.0 { 1.0 } else { length };
            let normalized = Row {
                a: row.coefficients.iter().map(|v| v / scale).collect(),
                b: row.value / scale,
                original_norm: scale,
            };
            if !normalized.b.is_finite() {
                return Err(error(
                    QpErrorKind::InvalidInput,
                    format!("Constraint scaling overflow at row {i}"),
                    diagnostics,
                ));
            }
            Ok(normalized)
        })
        .collect()
}

struct Bound {
    row_index: usize,
    a: Vec<f64>,
    b: f64,
    direction: Vec<f64>,
    denominator: f64,
    lambda: f64,
    /// L^-1 a, so pair curvature is a Gram product without cancellation.
    whitened: Vec<f64>,
}

pub fn solve_dense_qp(
    hessian: &[Vec<f64>],
    rhs: &[f64],
    equalities: &[LinearConstraint],
    lower_bounds: &[LinearConstraint],
    options: QpOptions,
) -> Result<QpSolution, QpError> {
    let mut diagnostics = QpDiagnostics::default();
    preparation::validate_input(hessian, rhs, options, &diagnostics)?;
    let eq = normalized_rows(equalities, rhs.len(), &diagnostics)?;
    let lower = normalized_rows(lower_bounds, rhs.len(), &diagnostics)?;
    let mut reduced =
        preparation::ReducedProblem::new(hessian, rhs, &eq, &lower, options, &mut diagnostics)?;
    let problem = certificate::CertificateProblem {
        hessian,
        rhs,
        equalities: &eq,
        lower_bounds: &lower,
    };
    let mut u = reduced.u0.clone();
    for sweep in 0..=options.max_sweeps {
        diagnostics.sweeps = sweep;
        let c = reduced.certify(&problem, &mut u, &mut diagnostics)?;
        if reduced.meets_tolerances(&u, options, &diagnostics) {
            return Ok(QpSolution {
                coefficients: c,
                lower_bound_multipliers: reduced.original_multipliers(&lower),
                diagnostics,
            });
        }
        if sweep == options.max_sweeps {
            break;
        }
        sweep_dual_coordinates(&mut reduced.bounds, &mut u, &diagnostics)?;
        if (sweep + 1) % 8 == 0 {
            diagnostics.dual_pair_steps += polish_dual_pairs(&mut reduced.bounds, &mut u);
            let (steps, transfers) = polish_dual_block(&mut reduced.bounds, &mut u);
            diagnostics.dual_block_steps += steps;
            diagnostics.dual_dependent_transfers += transfers;
        }
    }
    Err(error(
        QpErrorKind::NoConvergence,
        "KKT tolerances not reached; constraints may be infeasible or ill-conditioned. No fitted coefficients returned.",
        &diagnostics,
    ))
}

fn sweep_dual_coordinates(
    bounds: &mut [Bound],
    u: &mut [f64],
    diagnostics: &QpDiagnostics,
) -> Result<(), QpError> {
    for bound in bounds {
        let violation = bound.b - dot(&bound.a, u);
        let next = (bound.lambda + violation / bound.denominator).max(0.0);
        if !next.is_finite() {
            return Err(error(
                QpErrorKind::NumericalFailure,
                "Nonfinite inequality multiplier",
                diagnostics,
            ));
        }
        axpy(u, next - bound.lambda, &bound.direction);
        bound.lambda = next;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(a: &[f64], b: f64) -> LinearConstraint {
        LinearConstraint {
            coefficients: a.to_vec(),
            value: b,
        }
    }
    fn identity(n: usize) -> Vec<Vec<f64>> {
        (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect()
    }
    fn close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 2e-7, "{actual} != {expected}");
    }
    #[test]
    fn unconstrained_spd_optimum() {
        let result = solve_dense_qp(
            &vec![vec![2.0, 1.0], vec![1.0, 3.0]],
            &[1.0, 2.0],
            &[],
            &[],
            QpOptions::default(),
        )
        .unwrap();
        close(result.coefficients[0], 0.2);
        close(result.coefficients[1], 0.6);
    }
    #[test]
    fn releases_wrong_active_bound() {
        let result = solve_dense_qp(
            &identity(2),
            &[0.0, 0.0],
            &[],
            &[row(&[1.0, 0.0], 1.0), row(&[0.9, 0.1], 0.99)],
            QpOptions::default(),
        )
        .unwrap();
        close(result.coefficients[0], 0.99 * 0.9 / 0.82);
        close(result.coefficients[1], 0.99 * 0.1 / 0.82);
        close(result.lower_bound_multipliers[0], 0.0);
    }
    #[test]
    fn nearly_parallel_bounds_transfer_dual_mass_without_looser_tolerances() {
        let options = QpOptions {
            max_sweeps: 64,
            ..QpOptions::default()
        };
        let result = solve_dense_qp(
            &identity(2),
            &[0.0, 0.0],
            &[],
            &[row(&[1.0, 0.0], 1.0), row(&[1.0, 1e-5], 1.000001)],
            options,
        )
        .unwrap();
        close(result.coefficients[0], 1.000001 / (1.0 + 1e-10));
        close(result.coefficients[1], 1.000001e-5 / (1.0 + 1e-10));
        close(result.lower_bound_multipliers[0], 0.0);
        assert!(result.diagnostics.dual_pair_steps > 0);
        assert!(result.diagnostics.max_complementarity < 1e-8);
    }

    #[test]
    fn parallel_pair_polish_preserves_equalities_and_constraint_scaling() {
        let options = QpOptions {
            max_sweeps: 64,
            ..QpOptions::default()
        };
        let result = solve_dense_qp(
            &identity(3),
            &[0.0; 3],
            &[row(&[0.0, 0.0, 1.0], 0.2)],
            &[
                row(&[1.0, 0.0, 2.0], 1.4),
                row(&[1000.0, 0.01, -100.0], 980.001),
            ],
            options,
        )
        .unwrap();
        close(result.coefficients[0], 1.000001 / (1.0 + 1e-10));
        close(result.coefficients[1], 1.000001e-5 / (1.0 + 1e-10));
        close(result.coefficients[2], 0.2);
        assert!(result.diagnostics.max_equality_residual < 1e-12);
        assert!(result.diagnostics.dual_pair_steps > 0);
    }

    #[test]
    fn block_polish_releases_a_multivariate_dependent_active_row() {
        let root_half = 0.5_f64.sqrt();
        let rows = [
            row(&[1.0, 0.0], 1.0),
            row(&[0.0, 1.0], 1.0),
            row(&[root_half, root_half], 1.5),
        ];
        let mut bounds: Vec<_> = rows
            .iter()
            .enumerate()
            .map(|(index, r)| Bound {
                row_index: index,
                a: r.coefficients.clone(),
                b: r.value,
                direction: r.coefficients.clone(),
                denominator: 1.0,
                lambda: 1.0,
                whitened: r.coefficients.clone(),
            })
            .collect();
        let mut u = vec![1.0 + root_half; 2];
        let (steps, transfers) = polish_dual_block(&mut bounds, &mut u);
        assert!(steps > 0 && transfers > 0);
        for component in &u {
            close(*component, 1.5 * root_half);
        }
        for bound in &bounds {
            assert!(bound.lambda >= 0.0);
            let slack = dot(&bound.a, &u) - bound.b;
            assert!(slack >= -1e-12);
            assert!((bound.lambda * slack).abs() < 1e-12);
        }
    }

    #[test]
    fn coupled_correlated_rows_reach_the_known_multivariate_kkt_point() {
        let expected = [1.0, 0.2, 0.3];
        let coefficients = [
            vec![1.0, 0.0, 0.0],
            vec![1.0, 0.02, 0.0],
            vec![1.0, 0.0, 0.02],
        ];
        let multipliers = [1.0, 2.0, 3.0];
        let mut rhs = expected.to_vec();
        let mut lower = Vec::new();
        for (a, lambda) in coefficients.iter().zip(multipliers) {
            axpy(&mut rhs, -lambda, a);
            lower.push(row(a, dot(a, &expected)));
        }
        let result = solve_dense_qp(
            &identity(3),
            &rhs,
            &[],
            &lower,
            QpOptions {
                max_sweeps: 64,
                ..QpOptions::default()
            },
        )
        .unwrap();
        for (actual, expected) in result.coefficients.iter().zip(expected) {
            close(*actual, expected);
        }
        assert!(result.diagnostics.max_complementarity < 1e-8);
    }

    // Exercise the immutable real-world dump without adding a production JSON
    // dependency. Enable explicitly with BREASTPLATE_QP_FIXTURE=<dump path>.
    fn check_actual_fixture(environment_variable: &str) {
        fn array<'a>(text: &'a str, key: &str) -> &'a str {
            let key_start = text.find(&format!("\"{key}\"")).expect("fixture key");
            let start = key_start + text[key_start..].find('[').expect("fixture array");
            let mut depth = 0;
            for (i, byte) in text.as_bytes()[start..].iter().enumerate() {
                if *byte == b'[' {
                    depth += 1;
                }
                if *byte == b']' {
                    depth -= 1;
                    if depth == 0 {
                        return &text[start..=start + i];
                    }
                }
            }
            panic!("unterminated fixture array");
        }
        fn numbers(text: &str) -> Vec<f64> {
            let mut out = Vec::new();
            let bytes = text.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        i += 1;
                    }
                } else if bytes[i].is_ascii_digit() || bytes[i] == b'-' {
                    let start = i;
                    i += 1;
                    while i < bytes.len()
                        && (bytes[i].is_ascii_digit() || b".eE+-".contains(&bytes[i]))
                    {
                        i += 1;
                    }
                    out.push(text[start..i].parse().expect("fixture number"));
                    continue;
                }
                i += 1;
            }
            out
        }
        fn constraints(text: &str, n: usize) -> Vec<LinearConstraint> {
            let all = numbers(text);
            assert_eq!(all.len() % (n + 1), 0);
            all.chunks_exact(n + 1)
                .map(|chunk| row(&chunk[..n], chunk[n]))
                .collect()
        }
        let path = std::env::var(environment_variable).expect("set fixture environment variable");
        let text = std::fs::read_to_string(path).expect("read actual fixture");
        let rhs = numbers(array(&text, "rhs"));
        let n = rhs.len();
        let hessian: Vec<_> = numbers(array(&text, "hessian"))
            .chunks_exact(n)
            .map(|v| v.to_vec())
            .collect();
        let equalities = constraints(array(&text, "equalities"), n);
        let lower = constraints(array(&text, "lower_bounds"), n);
        let options = QpOptions {
            max_sweeps: 10_000,
            primal_tolerance: 2.5e-5,
            equality_tolerance: 2.5e-6,
            complementarity_tolerance: 2.5e-6,
            ..QpOptions::default()
        };
        let result = solve_dense_qp(&hessian, &rhs, &equalities, &lower, options).unwrap();
        eprintln!(
            "{environment_variable} certificate: {:?}",
            result.diagnostics
        );
        assert!(result.diagnostics.max_lower_bound_violation <= options.primal_tolerance);
        assert!(result.diagnostics.max_equality_residual <= options.equality_tolerance);
        assert!(
            result.diagnostics.max_complementarity
                <= options.complementarity_tolerance * (1.0 + result.diagnostics.objective.abs())
        );
        assert!(result.diagnostics.dual_pair_steps > 0);
        assert!(
            result
                .lower_bound_multipliers
                .iter()
                .all(|x| x.is_finite() && *x >= 0.0)
        );
    }
    #[test]
    #[ignore = "requires BREASTPLATE_QP_FIXTURE pointing at the actual interval-yoke fit dump"]
    fn actual_interval_yoke_dump_is_fully_certified() {
        check_actual_fixture("BREASTPLATE_QP_FIXTURE");
    }
    #[test]
    #[ignore = "requires BREASTPLATE_QP_MATERIAL_FIXTURE pointing at the actual material-fairness fit dump"]
    fn actual_material_fairness_dump_is_fully_certified() {
        check_actual_fixture("BREASTPLATE_QP_MATERIAL_FIXTURE");
    }
    #[test]
    fn dependent_inequalities_can_release_weaker_bound() {
        let result = solve_dense_qp(
            &identity(1),
            &[0.0],
            &[],
            &[row(&[10.0], 10.0), row(&[1.0], 2.0)],
            QpOptions::default(),
        )
        .unwrap();
        close(result.coefficients[0], 2.0);
        close(result.lower_bound_multipliers[0], 0.0);
    }
    #[test]
    fn exact_equality_conflicting_lower_bound_is_explicit() {
        let result = solve_dense_qp(
            &identity(2),
            &[0.0, 0.0],
            &[row(&[1.0, 0.0], 1.0)],
            &[row(&[2.0, 0.0], 4.0)],
            QpOptions::default(),
        );
        assert_eq!(result.unwrap_err().kind, QpErrorKind::EqualityBoundConflict);
    }
    #[test]
    fn redundant_equalities_preserve_exact_solution() {
        let result = solve_dense_qp(
            &identity(2),
            &[0.0, 0.0],
            &[row(&[1.0, 1.0], 3.0), row(&[2.0, 2.0], 6.0)],
            &[row(&[1.0, 0.0], 2.0)],
            QpOptions::default(),
        )
        .unwrap();
        close(result.coefficients[0], 2.0);
        close(result.coefficients[1], 1.0);
        assert_eq!(result.diagnostics.redundant_equalities, 1);
        assert!(result.diagnostics.max_equality_residual < 1e-12);
    }
    #[test]
    fn contradictory_equalities_rejected() {
        let result = solve_dense_qp(
            &identity(1),
            &[0.0],
            &[row(&[1.0], 1.0), row(&[2.0], 3.0)],
            &[],
            QpOptions::default(),
        );
        assert_eq!(
            result.unwrap_err().kind,
            QpErrorKind::InconsistentEqualities
        );
    }
    #[test]
    fn fully_determined_and_zero_rows() {
        let result = solve_dense_qp(
            &identity(1),
            &[5.0],
            &[row(&[1.0], 2.0), row(&[0.0], 0.0)],
            &[row(&[1.0], 1.0), row(&[0.0], -1.0)],
            QpOptions::default(),
        )
        .unwrap();
        close(result.coefficients[0], 2.0);
        assert_eq!(result.diagnostics.projected_zero_bounds, 2);
    }
    #[test]
    fn exhaustion_never_returns_uncertified_mesh() {
        let options = QpOptions {
            max_sweeps: 3,
            ..QpOptions::default()
        };
        let result = solve_dense_qp(
            &identity(1),
            &[0.0],
            &[],
            &[row(&[1.0], 2.0), row(&[-1.0], -1.0)],
            options,
        );
        assert_eq!(result.unwrap_err().kind, QpErrorKind::NoConvergence);
    }
    #[test]
    fn non_spd_and_nonfinite_inputs_rejected() {
        assert_eq!(
            solve_dense_qp(&vec![vec![0.0]], &[0.0], &[], &[], QpOptions::default())
                .unwrap_err()
                .kind,
            QpErrorKind::NotPositiveDefinite
        );
        assert_eq!(
            solve_dense_qp(&identity(1), &[f64::NAN], &[], &[], QpOptions::default())
                .unwrap_err()
                .kind,
            QpErrorKind::InvalidInput
        );
    }
    #[test]
    fn fifty_four_controls_with_many_bounds_match_known_kkt_solution() {
        let n = 54;
        let h: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| if i == j { 1.0 + i as f64 * 0.02 } else { 0.002 })
                    .collect()
            })
            .collect();
        let expected: Vec<f64> = (0..n).map(|i| 0.05 + i as f64 * 0.001).collect();
        let mut rhs = matvec(&h, &expected);
        let equalities: Vec<_> = (0..11)
            .map(|i| {
                let mut a = vec![0.0; n];
                a[i] = 1.0;
                row(&a, expected[i])
            })
            .collect();
        let mut lower = Vec::new();
        // Construct the objective gradient from known nonnegative active
        // multipliers, making expected an independently specified KKT point.
        for i in 11..31 {
            let mut a = vec![0.0; n];
            a[i] = 1.0;
            rhs[i] -= 0.01 + (i - 11) as f64 * 0.001;
            lower.push(row(&a, expected[i]));
        }
        for i in 0..1580 {
            let mut a = vec![0.0; n];
            a[i % n] += 0.6;
            a[(i * 7 + 3) % n] += 0.3;
            a[(i * 13 + 5) % n] += 0.1;
            lower.push(row(&a, dot(&a, &expected) - 0.003));
        }
        let result = solve_dense_qp(&h, &rhs, &equalities, &lower, QpOptions::default()).unwrap();
        let max_error = result
            .coefficients
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        assert!(
            max_error < 2e-7,
            "known KKT error {max_error}, {:?}",
            result.diagnostics
        );
        assert!(result.diagnostics.max_equality_residual < 1e-12);
        assert!(result.diagnostics.max_lower_bound_violation <= 1e-8);
    }
}

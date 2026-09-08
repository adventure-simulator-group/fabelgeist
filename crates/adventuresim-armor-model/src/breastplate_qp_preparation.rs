//! Input validation and exact-equality reduction for the dense QP.

use super::{
    Bound, QpDiagnostics, QpError, QpErrorKind, QpOptions, Row, axpy, cholesky, dot, error, matvec,
    max_abs, norm, triangular_solve,
};

pub(super) struct ReducedProblem {
    pub(super) particular: Vec<f64>,
    pub(super) z: Vec<Vec<f64>>,
    pub(super) reduced_h: Vec<Vec<f64>>,
    pub(super) reduced_rhs: Vec<f64>,
    pub(super) u0: Vec<f64>,
    pub(super) bounds: Vec<Bound>,
}

pub(super) fn validate_input(
    hessian: &[Vec<f64>],
    rhs: &[f64],
    options: QpOptions,
    diagnostics: &QpDiagnostics,
) -> Result<(), QpError> {
    let n = rhs.len();
    let tolerances = [
        options.rank_tolerance,
        options.equality_tolerance,
        options.primal_tolerance,
        options.complementarity_tolerance,
        options.stationarity_tolerance,
    ];
    if n == 0
        || hessian.len() != n
        || hessian.iter().any(|r| r.len() != n)
        || rhs
            .iter()
            .chain(hessian.iter().flatten())
            .any(|x| !x.is_finite())
        || tolerances.iter().any(|x| !x.is_finite() || *x <= 0.0)
        || options.rank_tolerance >= 0.1
    {
        return Err(error(
            QpErrorKind::InvalidInput,
            "Invalid dimensions, values, or tolerances",
            diagnostics,
        ));
    }
    let h_scale = hessian.iter().map(|r| max_abs(r)).fold(0.0_f64, f64::max);
    for i in 0..n {
        for j in 0..i {
            if (hessian[i][j] - hessian[j][i]).abs() > 1e-12 * h_scale.max(f64::MIN_POSITIVE) {
                return Err(error(
                    QpErrorKind::InvalidInput,
                    "Hessian is not symmetric",
                    diagnostics,
                ));
            }
        }
    }
    // Validate H itself, not only its restriction: the public contract is SPD.
    let _ = cholesky(hessian, diagnostics)?;
    Ok(())
}

fn equality_space(
    eq: &[Row],
    n: usize,
    options: QpOptions,
    diagnostics: &mut QpDiagnostics,
) -> Result<(Vec<f64>, Vec<Vec<f64>>), QpError> {
    // Two-pass modified Gram-Schmidt on augmented equality rows. q_j.c=d_j.
    let mut q: Vec<Vec<f64>> = Vec::new();
    let mut d: Vec<f64> = Vec::new();
    for (index, row) in eq.iter().enumerate() {
        let mut a = row.a.clone();
        let mut b = row.b;
        for _ in 0..2 {
            for (basis, value) in q.iter().zip(&d) {
                let projection = dot(&a, basis);
                axpy(&mut a, -projection, basis);
                b -= projection * value;
            }
        }
        let length = norm(&a);
        if length <= options.rank_tolerance {
            if b.abs() > options.equality_tolerance {
                return Err(error(
                    QpErrorKind::InconsistentEqualities,
                    format!("Dependent equality {index} has incompatible normalized residual {b}"),
                    diagnostics,
                ));
            }
            diagnostics.redundant_equalities += 1;
        } else {
            q.push(a.iter().map(|x| x / length).collect());
            d.push(b / length);
            diagnostics.equality_rank += 1;
        }
    }
    let mut particular = vec![0.0; n];
    for (basis, value) in q.iter().zip(&d) {
        axpy(&mut particular, *value, basis);
    }
    // Explicit nullspace avoids subtracting two almost equal inverse-H terms
    // when a lower row lies on (or close to) an exact anchor row.
    let mut z: Vec<Vec<f64>> = Vec::new();
    for column in 0..n {
        let mut basis = vec![0.0; n];
        basis[column] = 1.0;
        for _ in 0..2 {
            for existing in q.iter().chain(&z) {
                let projection = dot(&basis, existing);
                axpy(&mut basis, -projection, existing);
            }
        }
        let length = norm(&basis);
        if length > options.rank_tolerance {
            z.push(basis.iter().map(|x| x / length).collect());
        }
    }
    if z.len() + q.len() != n || particular.iter().any(|x| !x.is_finite()) {
        return Err(error(
            QpErrorKind::NumericalFailure,
            "Equality nullspace construction failed",
            diagnostics,
        ));
    }
    Ok((particular, z))
}

fn project_bounds(
    lower: &[Row],
    z: &[Vec<f64>],
    particular: &[f64],
    factor: &[Vec<f64>],
    options: QpOptions,
    diagnostics: &mut QpDiagnostics,
) -> Result<Vec<Bound>, QpError> {
    let mut bounds = Vec::new();
    for (index, row) in lower.iter().enumerate() {
        let a: Vec<_> = z.iter().map(|basis| dot(basis, &row.a)).collect();
        let b = row.b - dot(&row.a, particular);
        if norm(&a) <= options.rank_tolerance {
            diagnostics.projected_zero_bounds += 1;
            if b > options.primal_tolerance {
                diagnostics.max_lower_bound_violation = b;
                return Err(error(
                    QpErrorKind::EqualityBoundConflict,
                    format!("Lower bound {index} is fixed by exact equalities but violated by {b}"),
                    diagnostics,
                ));
            }
            continue;
        }
        let direction = triangular_solve(factor, &a);
        let mut whitened = vec![0.0; a.len()];
        for i in 0..a.len() {
            whitened[i] = (a[i] - dot(&factor[i][..i], &whitened[..i])) / factor[i][i];
        }
        let denominator = dot(&whitened, &whitened);
        if !denominator.is_finite()
            || denominator <= 0.0
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(error(
                QpErrorKind::NumericalFailure,
                format!("Invalid projected lower bound {index}"),
                diagnostics,
            ));
        }
        bounds.push(Bound {
            row_index: index,
            a,
            b,
            direction,
            denominator,
            lambda: 0.0,
            whitened,
        });
    }
    Ok(bounds)
}

impl ReducedProblem {
    pub(super) fn new(
        hessian: &[Vec<f64>],
        rhs: &[f64],
        eq: &[Row],
        lower: &[Row],
        options: QpOptions,
        diagnostics: &mut QpDiagnostics,
    ) -> Result<Self, QpError> {
        let (particular, z) = equality_space(eq, rhs.len(), options, diagnostics)?;
        let hz: Vec<_> = z.iter().map(|column| matvec(hessian, column)).collect();
        let reduced_h: Vec<Vec<f64>> = z
            .iter()
            .map(|row| hz.iter().map(|column| dot(row, column)).collect())
            .collect();
        let hp = matvec(hessian, &particular);
        let shifted_rhs: Vec<_> = rhs.iter().zip(hp).map(|(a, b)| a - b).collect();
        let reduced_rhs: Vec<_> = z.iter().map(|row| dot(row, &shifted_rhs)).collect();
        let factor = cholesky(&reduced_h, diagnostics)?;
        let u0 = triangular_solve(&factor, &reduced_rhs);
        let bounds = project_bounds(lower, &z, &particular, &factor, options, diagnostics)?;
        Ok(Self {
            particular,
            z,
            reduced_h,
            reduced_rhs,
            u0,
            bounds,
        })
    }
}

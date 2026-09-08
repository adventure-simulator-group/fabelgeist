//! Reconstructed primal iterates and the unchanged global KKT certificate.

use super::preparation::ReducedProblem;
use super::{
    QpDiagnostics, QpError, QpErrorKind, QpOptions, Row, axpy, dot, error, matvec, max_abs,
};

pub(super) struct CertificateProblem<'a> {
    pub(super) hessian: &'a [Vec<f64>],
    pub(super) rhs: &'a [f64],
    pub(super) equalities: &'a [Row],
    pub(super) lower_bounds: &'a [Row],
}

impl ReducedProblem {
    pub(super) fn certify(
        &self,
        problem: &CertificateProblem<'_>,
        u: &mut Vec<f64>,
        diagnostics: &mut QpDiagnostics,
    ) -> Result<Vec<f64>, QpError> {
        let CertificateProblem {
            hessian,
            rhs,
            equalities: eq,
            lower_bounds: lower,
        } = problem;
        // Reconstruct from dual variables before certifying. This makes the
        // stationarity/equality checks independent of accumulated update drift.
        u.clone_from(&self.u0);
        for bound in &self.bounds {
            axpy(u, bound.lambda, &bound.direction);
        }
        let mut c = self.particular.clone();
        for (basis, value) in self.z.iter().zip(u.iter()) {
            axpy(&mut c, *value, basis);
        }
        let hc = matvec(hessian, &c);
        diagnostics.objective = 0.5 * dot(&c, &hc) - dot(rhs, &c);
        diagnostics.max_equality_residual = eq
            .iter()
            .map(|r| (dot(&r.a, &c) - r.b).abs())
            .fold(0.0, f64::max);
        diagnostics.max_lower_bound_violation = lower
            .iter()
            .map(|r| (r.b - dot(&r.a, &c)).max(0.0))
            .fold(0.0, f64::max);
        diagnostics.max_complementarity = 0.0;
        diagnostics.max_projected_update = 0.0;
        diagnostics.active_lower_bounds = 0;
        let mut gradient = matvec(&self.reduced_h, u);
        axpy(&mut gradient, -1.0, &self.reduced_rhs);
        for bound in &self.bounds {
            let slack = dot(&bound.a, u) - bound.b;
            diagnostics.max_complementarity = diagnostics
                .max_complementarity
                .max((bound.lambda * slack).abs());
            let delta = (bound.lambda - slack / bound.denominator).max(0.0) - bound.lambda;
            diagnostics.max_projected_update = diagnostics
                .max_projected_update
                .max(delta.abs() * bound.denominator.sqrt());
            if bound.lambda > 0.0 {
                diagnostics.active_lower_bounds += 1;
            }
            axpy(&mut gradient, -bound.lambda, &bound.a);
        }
        diagnostics.projected_stationarity_residual = max_abs(&gradient);
        let finite_diagnostics = [
            diagnostics.objective,
            diagnostics.max_equality_residual,
            diagnostics.max_lower_bound_violation,
            diagnostics.max_complementarity,
            diagnostics.max_projected_update,
            diagnostics.projected_stationarity_residual,
        ];
        if c.iter()
            .chain(u.iter())
            .chain(&gradient)
            .any(|v| !v.is_finite())
            || finite_diagnostics.iter().any(|v| !v.is_finite())
        {
            return Err(error(
                QpErrorKind::NumericalFailure,
                "Nonfinite iterate or KKT certificate",
                diagnostics,
            ));
        }
        Ok(c)
    }

    pub(super) fn meets_tolerances(
        &self,
        u: &[f64],
        options: QpOptions,
        diagnostics: &QpDiagnostics,
    ) -> bool {
        let stationarity_scale =
            1.0 + max_abs(&self.reduced_rhs) + max_abs(&matvec(&self.reduced_h, u));
        diagnostics.max_equality_residual <= options.equality_tolerance
            && diagnostics.max_lower_bound_violation <= options.primal_tolerance
            && diagnostics.max_complementarity
                <= options.complementarity_tolerance * (1.0 + diagnostics.objective.abs())
            && diagnostics.projected_stationarity_residual
                <= options.stationarity_tolerance * stationarity_scale
    }

    pub(super) fn original_multipliers(&self, lower: &[Row]) -> Vec<f64> {
        let mut multipliers = vec![0.0; lower.len()];
        for bound in &self.bounds {
            multipliers[bound.row_index] = bound.lambda / lower[bound.row_index].original_norm;
        }
        multipliers
    }
}

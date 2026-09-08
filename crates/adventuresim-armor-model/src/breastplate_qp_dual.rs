//! Dual-feasible pair and rank-safe block acceleration.

use super::{Bound, axpy, dot, norm};

/// Minimize the dual objective over pairs of currently positive multipliers.
/// Scalar sweeps can transfer mass between nearly parallel constraints at a
/// rate proportional to their tiny angular separation. A two-variable solve
/// performs that transfer in one step, including setting a redundant bound's
/// multiplier to zero. Every candidate remains dual-feasible and is accepted
/// only if its ACTUAL Gram-quadratic objective change is strictly negative.
/// Singular pairs need no inverse: their two boundary candidates remain valid.
pub(super) fn polish_dual_pairs(bounds: &mut [Bound], u: &mut [f64]) -> usize {
    let mut active: Vec<usize> = bounds
        .iter()
        .enumerate()
        .filter_map(|(i, b)| (b.lambda > 0.0).then_some(i))
        .collect();
    active.sort_by(|&i, &j| {
        (bounds[j].lambda * bounds[j].denominator.sqrt())
            .total_cmp(&(bounds[i].lambda * bounds[i].denominator.sqrt()))
    });
    // This is only an acceleration working set, never a constraint filter.
    // Full scalar sweeps and the global certificate still include every row.
    active.truncate(64);
    let mut accepted = 0;
    for _ in 0..2 {
        for left in 0..active.len() {
            for right in left + 1..active.len() {
                let i = active[left];
                let j = active[right];
                let a = &bounds[i];
                let b = &bounds[j];
                if a.lambda == 0.0 && b.lambda == 0.0 {
                    continue;
                }
                let si = a.denominator.sqrt();
                let sj = b.denominator.sqrt();
                let old_i = a.lambda * si;
                let old_j = b.lambda * sj;
                let gi = (dot(&a.a, u) - a.b) / si;
                let gj = (dot(&b.a, u) - b.b) / sj;
                let rho = (dot(&a.whitened, &b.whitened) / (si * sj)).clamp(-1.0, 1.0);
                // Candidates are NEW normalized multipliers, not increments.
                let mut candidates = vec![
                    (0.0, (old_j - gj + rho * old_i).max(0.0)),
                    ((old_i - gi + rho * old_j).max(0.0), 0.0),
                ];
                let determinant = (1.0 - rho) * (1.0 + rho);
                if determinant > 1e-12 {
                    let ni = old_i + (rho * gj - gi) / determinant;
                    let nj = old_j + (rho * gi - gj) / determinant;
                    if ni >= 0.0 && nj >= 0.0 {
                        candidates.push((ni, nj));
                    }
                }
                let mut best = None;
                let mut best_change = 0.0;
                for (ni, nj) in candidates {
                    if !ni.is_finite() || !nj.is_finite() {
                        continue;
                    }
                    let di = (ni - old_i) / si;
                    let dj = (nj - old_j) / sj;
                    // Direct sum of squares remains accurate when the two
                    // large, opposite multiplier increments almost cancel.
                    let quadratic = a
                        .whitened
                        .iter()
                        .zip(&b.whitened)
                        .map(|(x, y)| (di * x + dj * y).powi(2))
                        .sum::<f64>()
                        * 0.5;
                    let linear = gi * (ni - old_i) + gj * (nj - old_j);
                    let change = linear + quadratic;
                    if change.is_finite()
                        && change < best_change
                        && change < -1e-15 * (1.0 + linear.abs() + quadratic)
                    {
                        best_change = change;
                        best = Some((ni / si, nj / sj));
                    }
                }
                if let Some((new_i, new_j)) = best {
                    axpy(u, new_i - bounds[i].lambda, &bounds[i].direction);
                    axpy(u, new_j - bounds[j].lambda, &bounds[j].direction);
                    bounds[i].lambda = new_i;
                    bounds[j].lambda = new_j;
                    accepted += 1;
                }
            }
        }
    }
    accepted
}

struct ActiveBlock {
    active: Vec<usize>,
    columns: Vec<Vec<f64>>,
    independent: Vec<usize>,
    dependency: Option<(usize, Vec<f64>)>,
}

impl ActiveBlock {
    fn factor(bounds: &[Bound]) -> Self {
        let mut active: Vec<usize> = bounds
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (b.lambda > 0.0).then_some(i))
            .collect();
        active.sort_by(|&i, &j| {
            (bounds[j].lambda * bounds[j].denominator.sqrt())
                .total_cmp(&(bounds[i].lambda * bounds[i].denominator.sqrt()))
        });
        let mut basis: Vec<Vec<f64>> = Vec::new();
        let mut columns: Vec<Vec<f64>> = Vec::new();
        let mut independent: Vec<usize> = Vec::new();
        let mut dependency = None;
        for &index in &active {
            let b = &bounds[index];
            let scale = b.denominator.sqrt();
            let mut v: Vec<_> = b.whitened.iter().map(|x| x / scale).collect();
            let mut column = vec![0.0; basis.len()];
            for _ in 0..2 {
                for (j, q) in basis.iter().enumerate() {
                    let projection = dot(q, &v);
                    column[j] += projection;
                    axpy(&mut v, -projection, q);
                }
            }
            let length = norm(&v);
            if length < 1e-10 {
                // R alpha = Q^T v. columns store the upper triangular R.
                let mut alpha = column;
                for i in (0..alpha.len()).rev() {
                    let rest: f64 = (i + 1..alpha.len()).map(|j| columns[j][i] * alpha[j]).sum();
                    alpha[i] = (alpha[i] - rest) / columns[i][i];
                }
                dependency = Some((index, alpha));
                break;
            }
            column.push(length);
            columns.push(column);
            basis.push(v.iter().map(|x| x / length).collect());
            independent.push(index);
        }
        Self {
            active,
            columns,
            independent,
            dependency,
        }
    }

    fn steps(&self, bounds: &[Bound], u: &[f64]) -> Vec<f64> {
        let Self {
            active,
            columns,
            independent,
            dependency,
        } = self;
        let mut steps = vec![0.0; bounds.len()];
        if let Some((index, alpha)) = dependency {
            steps[*index] = 1.0 / bounds[*index].denominator.sqrt();
            for (&i, a) in independent.iter().zip(alpha) {
                steps[i] = -a / bounds[i].denominator.sqrt();
            }
            let slope: f64 = active
                .iter()
                .map(|&i| steps[i] * (dot(&bounds[i].a, u) - bounds[i].b))
                .sum();
            if slope > 0.0 {
                for step in &mut steps {
                    *step = -*step;
                }
            }
        } else {
            // G = R^T R for the normalized active rows. Solve without forming G.
            let mut delta: Vec<f64> = independent
                .iter()
                .map(|&i| -(dot(&bounds[i].a, u) - bounds[i].b) / bounds[i].denominator.sqrt())
                .collect();
            for i in 0..delta.len() {
                delta[i] = (delta[i] - dot(&columns[i][..i], &delta[..i])) / columns[i][i];
            }
            for i in (0..delta.len()).rev() {
                let rest: f64 = (i + 1..delta.len()).map(|j| columns[j][i] * delta[j]).sum();
                delta[i] = (delta[i] - rest) / columns[i][i];
            }
            for (&i, delta) in independent.iter().zip(delta) {
                steps[i] = delta / bounds[i].denominator.sqrt();
            }
        }
        steps
    }
}

/// Active-set minimization of the dual restricted to its positive support.
/// QR acts on Hessian-whitened rows rather than their squared-condition Gram
/// matrix. Dependent rows admit a nullspace multiplier transfer; independent
/// rows admit a simultaneous Newton step. The first multiplier hitting zero
/// leaves the support. No rows are removed from the QP or its certificate.
pub(super) fn polish_dual_block(bounds: &mut [Bound], u: &mut [f64]) -> (usize, usize) {
    let mut accepted = 0;
    let mut transfers = 0;
    for _ in 0..64 {
        let block = ActiveBlock::factor(bounds);
        let mut steps = block.steps(bounds, u);
        let dependent = block.dependency.is_some();
        let active = &block.active;
        let mut length = if dependent { f64::INFINITY } else { 1.0 };
        let mut blocking = None;
        for &i in active {
            if steps[i] < 0.0 {
                let limit = -bounds[i].lambda / steps[i];
                if limit <= length {
                    length = limit;
                    blocking = Some(i);
                }
            }
        }
        if !length.is_finite() || length <= 0.0 {
            break;
        }
        // Test the ACTUAL proposed update, including rounding at the bound.
        let mut white_update = vec![0.0; u.len()];
        let mut linear = 0.0;
        for &i in active {
            let next = if blocking == Some(i) {
                0.0
            } else {
                (bounds[i].lambda + length * steps[i]).max(0.0)
            };
            steps[i] = next - bounds[i].lambda;
            linear += steps[i] * (dot(&bounds[i].a, u) - bounds[i].b);
            axpy(&mut white_update, steps[i], &bounds[i].whitened);
        }
        let quadratic = 0.5 * dot(&white_update, &white_update);
        if !linear.is_finite()
            || !quadratic.is_finite()
            || linear + quadratic >= -1e-15 * (1.0 + linear.abs() + quadratic)
        {
            break;
        }
        for &i in active {
            bounds[i].lambda += steps[i];
            axpy(u, steps[i], &bounds[i].direction);
        }
        accepted += 1;
        transfers += usize::from(dependent);
    }
    (accepted, transfers)
}

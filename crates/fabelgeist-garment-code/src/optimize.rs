//! Small-scale numerical optimisation -- a stand-in for the
//! `scipy.optimize.minimize` calls in the reference implementation.
//!
//! Two entry points mirror the two ways GarmentCode calls `minimize`:
//! * [`minimize`] -- unconstrained BFGS (scipy's default method), with a
//!   forward-difference gradient and a strong-Wolfe line search.
//! * [`minimize_bounded`] -- box-constrained (scipy dispatches to L-BFGS-B when
//!   `bounds` are given). Implemented as projected BFGS with an active set.
//!
//! All the objectives involved are smooth, low-dimensional least-squares fits,
//! so both routines land on the same minima the reference finds.

/// Forward-difference step, matching scipy's `sqrt(finfo(float).eps)`.
const FD_EPS: f64 = 1.4901161193847656e-08;
/// scipy's default `gtol` for BFGS and `pgtol` for L-BFGS-B.
const GTOL: f64 = 1e-5;
/// scipy's L-BFGS-B `factr` default (1e7) times machine epsilon.
const FTOL: f64 = 1e7 * f64::EPSILON;

#[derive(Debug, Clone)]
pub struct OptResult {
    pub x: Vec<f64>,
    pub fun: f64,
    pub success: bool,
    pub iterations: usize,
}

fn gradient<F: Fn(&[f64]) -> f64>(f: &F, x: &[f64], fx: f64) -> Vec<f64> {
    let mut g = vec![0.0; x.len()];
    let mut probe = x.to_vec();
    for i in 0..x.len() {
        let h = FD_EPS * x[i].abs().max(1.0);
        probe[i] = x[i] + h;
        g[i] = (f(&probe) - fx) / h;
        probe[i] = x[i];
    }
    g
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn norm_inf(a: &[f64]) -> f64 {
    a.iter().fold(0.0_f64, |m, v| m.max(v.abs()))
}

/// Identity-initialised inverse-Hessian approximation.
fn eye(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect()
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter().map(|row| dot(row, v)).collect()
}

/// BFGS inverse-Hessian update (Sherman-Morrison form).
fn bfgs_update(h: &mut [Vec<f64>], s: &[f64], y: &[f64]) {
    let sy = dot(s, y);
    if sy <= 1e-14 {
        return;
    }
    let rho = 1.0 / sy;
    let n = s.len();

    let hy = matvec(h, y);
    let yhy = dot(y, &hy);

    for i in 0..n {
        for j in 0..n {
            h[i][j] += rho * rho * (sy + yhy) * s[i] * s[j] - rho * (hy[i] * s[j] + s[i] * hy[j]);
        }
    }
}

/// Strong-Wolfe line search (Nocedal & Wright, Alg. 3.5/3.6) with
/// `c1 = 1e-4`, `c2 = 0.9` -- scipy's defaults.
///
/// `alpha_max` caps the step, which is how box constraints are honoured: the
/// caller passes the largest step that stays inside the feasible box.
fn line_search<F: Fn(&[f64]) -> f64>(
    f: &F,
    x: &[f64],
    d: &[f64],
    f0: f64,
    g0: &[f64],
    alpha0: f64,
    alpha_max: f64,
) -> Option<(f64, Vec<f64>, f64)> {
    const C1: f64 = 1e-4;
    const C2: f64 = 0.9;

    let phi = |a: f64| -> (Vec<f64>, f64) {
        let xn: Vec<f64> = x.iter().zip(d).map(|(xi, di)| xi + a * di).collect();
        let fn_ = f(&xn);
        (xn, fn_)
    };
    let dphi0 = dot(g0, d);
    if dphi0 >= 0.0 {
        return None;
    }

    let mut a_prev = 0.0;
    let mut f_prev = f0;
    let mut a = alpha0.min(alpha_max);
    if a <= 0.0 {
        return None;
    }

    for i in 0..30 {
        let (xn, fn_) = phi(a);
        if !fn_.is_finite() {
            a *= 0.5;
            continue;
        }
        if fn_ > f0 + C1 * a * dphi0 || (i > 0 && fn_ >= f_prev) {
            return zoom(f, x, d, f0, dphi0, a_prev, f_prev, a, C1, C2);
        }
        let gn = gradient(f, &xn, fn_);
        let dphi = dot(&gn, d);
        if dphi.abs() <= -C2 * dphi0 {
            return Some((a, xn, fn_));
        }
        if dphi >= 0.0 {
            return zoom(f, x, d, f0, dphi0, a, fn_, a_prev, C1, C2);
        }
        if a >= alpha_max {
            // Hit the box boundary along this direction.
            return Some((a, xn, fn_));
        }
        a_prev = a;
        f_prev = fn_;
        a = (a * 2.0).min(alpha_max);
    }

    let (xn, fn_) = phi(a);
    Some((a, xn, fn_))
}

#[allow(clippy::too_many_arguments)]
fn zoom<F: Fn(&[f64]) -> f64>(
    f: &F,
    x: &[f64],
    d: &[f64],
    f0: f64,
    dphi0: f64,
    mut a_lo: f64,
    mut f_lo: f64,
    mut a_hi: f64,
    c1: f64,
    c2: f64,
) -> Option<(f64, Vec<f64>, f64)> {
    for _ in 0..40 {
        let a = 0.5 * (a_lo + a_hi);
        let xn: Vec<f64> = x.iter().zip(d).map(|(xi, di)| xi + a * di).collect();
        let fn_ = f(&xn);

        if !fn_.is_finite() || fn_ > f0 + c1 * a * dphi0 || fn_ >= f_lo {
            a_hi = a;
        } else {
            let gn = gradient(f, &xn, fn_);
            let dphi = dot(&gn, d);
            if dphi.abs() <= -c2 * dphi0 {
                return Some((a, xn, fn_));
            }
            if dphi * (a_hi - a_lo) >= 0.0 {
                a_hi = a_lo;
            }
            a_lo = a;
            f_lo = fn_;
        }
        if (a_hi - a_lo).abs() < 1e-16 {
            break;
        }
    }
    let a = a_lo;
    let xn: Vec<f64> = x.iter().zip(d).map(|(xi, di)| xi + a * di).collect();
    let fn_ = f(&xn);
    Some((a, xn, fn_))
}

/// Unconstrained BFGS -- `scipy.optimize.minimize(f, x0)`.
pub fn minimize<F: Fn(&[f64]) -> f64>(f: F, x0: &[f64]) -> OptResult {
    let n = x0.len();
    let mut x = x0.to_vec();
    let mut fx = f(&x);
    let mut g = gradient(&f, &x, fx);
    let mut h = eye(n);

    let max_iter = 200 * n;
    let mut old_fval = fx + norm2_vec(&g) / 2.0;

    for it in 0..max_iter {
        if norm_inf(&g) <= GTOL {
            return OptResult {
                x,
                fun: fx,
                success: true,
                iterations: it,
            };
        }

        let d: Vec<f64> = matvec(&h, &g).iter().map(|v| -v).collect();
        let dphi0 = dot(&g, &d);
        if dphi0 >= 0.0 {
            // Reset to steepest descent if the direction is not a descent one.
            h = eye(n);
            let d: Vec<f64> = g.iter().map(|v| -v).collect();
            match line_search(&f, &x, &d, fx, &g, 1.0, f64::INFINITY) {
                Some((_, xn, fn_)) => {
                    let gn = gradient(&f, &xn, fn_);
                    x = xn;
                    fx = fn_;
                    g = gn;
                    continue;
                }
                None => break,
            }
        }

        // scipy's initial step guess for BFGS.
        let alpha0 = (1.01 * 2.0 * (fx - old_fval) / dphi0).clamp(1e-12, 1.0);
        let alpha0 = if alpha0.is_finite() && alpha0 > 0.0 {
            alpha0
        } else {
            1.0
        };

        let Some((_, xn, fn_)) = line_search(&f, &x, &d, fx, &g, alpha0, f64::INFINITY) else {
            break;
        };

        let gn = gradient(&f, &xn, fn_);
        let s: Vec<f64> = xn.iter().zip(&x).map(|(a, b)| a - b).collect();
        let y: Vec<f64> = gn.iter().zip(&g).map(|(a, b)| a - b).collect();
        bfgs_update(&mut h, &s, &y);

        old_fval = fx;
        x = xn;
        fx = fn_;
        g = gn;

        if norm_inf(&s) < 1e-14 {
            break;
        }
    }

    let success = norm_inf(&g) <= GTOL;
    OptResult {
        x,
        fun: fx,
        success,
        iterations: max_iter,
    }
}

fn norm2_vec(v: &[f64]) -> f64 {
    dot(v, v).sqrt()
}

/// A box constraint. `None` means unbounded on that side.
pub type Bound = (Option<f64>, Option<f64>);

fn clamp_to_bounds(x: &mut [f64], bounds: &[Bound]) {
    for (i, xi) in x.iter_mut().enumerate() {
        let b = bound_for(bounds, i);
        if let Some(lo) = b.0
            && *xi < lo
        {
            *xi = lo;
        }
        if let Some(hi) = b.1
            && *xi > hi
        {
            *xi = hi;
        }
    }
}

/// scipy broadcasts a single `(lo, hi)` pair across every variable, which the
/// reference relies on (`bounds=[(0, 1)]` for a 2-vector).
fn bound_for(bounds: &[Bound], i: usize) -> Bound {
    if bounds.len() == 1 {
        bounds[0]
    } else {
        bounds[i]
    }
}

/// Largest step along `d` from `x` that stays inside the box.
fn max_feasible_step(x: &[f64], d: &[f64], bounds: &[Bound]) -> f64 {
    let mut alpha = f64::INFINITY;
    for i in 0..x.len() {
        let b = bound_for(bounds, i);
        if d[i] > 0.0 {
            if let Some(hi) = b.1 {
                alpha = alpha.min((hi - x[i]) / d[i]);
            }
        } else if d[i] < 0.0
            && let Some(lo) = b.0
        {
            alpha = alpha.min((lo - x[i]) / d[i]);
        }
    }
    alpha.max(0.0)
}

/// Box-constrained minimisation -- `scipy.optimize.minimize(f, x0, bounds=...)`,
/// which dispatches to L-BFGS-B.
///
/// Projected BFGS with an active set: variables sitting on a bound whose
/// gradient pushes further out are frozen, and the line search is capped at the
/// box boundary. When no bound is active this reduces to the same quasi-Newton
/// iteration as [`minimize`], which is where the accuracy comes from -- the
/// minima these objectives have are interior in every case that matters.
pub fn minimize_bounded<F: Fn(&[f64]) -> f64>(f: F, x0: &[f64], bounds: &[Bound]) -> OptResult {
    let n = x0.len();
    let mut x = x0.to_vec();
    clamp_to_bounds(&mut x, bounds);

    let mut fx = f(&x);
    let mut g = gradient(&f, &x, fx);
    let mut h = eye(n);

    let max_iter = 200 * n + 100;
    let mut stalled = 0usize;
    let mut prev_active: Vec<bool> = vec![false; n];

    for it in 0..max_iter {
        // Which variables are pinned to a bound by their own gradient?
        let active = |x: &[f64], g: &[f64], i: usize| -> bool {
            let b = bound_for(bounds, i);
            let at_lo = b.0.map(|lo| x[i] <= lo + 1e-12).unwrap_or(false);
            let at_hi = b.1.map(|hi| x[i] >= hi - 1e-12).unwrap_or(false);
            (at_lo && g[i] > 0.0) || (at_hi && g[i] < 0.0)
        };

        let active_set: Vec<bool> = (0..n).map(|i| active(&x, &g, i)).collect();
        // The curvature estimate is meaningless across a change of active set.
        if active_set != prev_active {
            h = eye(n);
            prev_active = active_set.clone();
        }

        // Projected gradient: frozen components do not count towards the
        // stopping test.
        let pg: Vec<f64> = (0..n)
            .map(|i| if active_set[i] { 0.0 } else { g[i] })
            .collect();

        if norm_inf(&pg) <= GTOL {
            return OptResult {
                x,
                fun: fx,
                success: true,
                iterations: it,
            };
        }

        // Zero out any component that would immediately leave the box --
        // including ones that are *not* in the active set, whose quasi-Newton
        // direction can still point outwards. Without this the feasible step
        // length collapses to zero and the search stops at a corner.
        let feasible_dir = |d: &mut Vec<f64>, x: &[f64]| {
            for i in 0..n {
                let b = bound_for(bounds, i);
                let at_lo = b.0.map(|lo| x[i] <= lo + 1e-12).unwrap_or(false);
                let at_hi = b.1.map(|hi| x[i] >= hi - 1e-12).unwrap_or(false);
                if active_set[i] || (at_lo && d[i] < 0.0) || (at_hi && d[i] > 0.0) {
                    d[i] = 0.0;
                }
            }
        };

        let mut d: Vec<f64> = matvec(&h, &pg).iter().map(|v| -v).collect();
        feasible_dir(&mut d, &x);

        if norm_inf(&d) == 0.0 || dot(&pg, &d) >= 0.0 {
            // Not a usable direction -- restart from steepest descent.
            h = eye(n);
            d = pg.iter().map(|v| -v).collect();
            feasible_dir(&mut d, &x);
        }
        if norm_inf(&d) == 0.0 {
            // Every descent direction leaves the box: this is a KKT point.
            return OptResult {
                x,
                fun: fx,
                success: true,
                iterations: it,
            };
        }

        let alpha_max = max_feasible_step(&x, &d, bounds);
        if alpha_max <= 0.0 {
            break;
        }

        // NOTE: L-BFGS-B normalises its very first step to unit length. Copying
        // that was tried and made the fits measurably worse on the reference's
        // own objective, so the plain unit step stays.
        let Some((_, mut x_new, f_new)) = line_search(&f, &x, &d, fx, &pg, 1.0, alpha_max) else {
            break;
        };
        clamp_to_bounds(&mut x_new, bounds);
        let f_new = if x_new
            .iter()
            .zip(&x)
            .all(|(a, b)| (a - b).abs() < f64::EPSILON)
        {
            f_new
        } else {
            f(&x_new)
        };

        let g_new = gradient(&f, &x_new, f_new);
        let s: Vec<f64> = x_new.iter().zip(&x).map(|(a, b)| a - b).collect();
        let y: Vec<f64> = g_new.iter().zip(&g).map(|(a, b)| a - b).collect();
        bfgs_update(&mut h, &s, &y);

        // Only give up after the objective has stopped moving several times in
        // a row -- a single tiny step is normal along a flat valley.
        if (fx - f_new).abs() <= FTOL * fx.abs().max(f_new.abs()).max(1.0) {
            stalled += 1;
        } else {
            stalled = 0;
        }

        x = x_new;
        fx = f_new;
        g = g_new;

        if stalled >= 3 || norm_inf(&s) < 1e-15 {
            return OptResult {
                x,
                fun: fx,
                success: true,
                iterations: it,
            };
        }
    }

    let success = {
        let pg: Vec<f64> = (0..n)
            .map(|i| {
                let b = bound_for(bounds, i);
                let at_lo = b.0.map(|lo| x[i] <= lo + 1e-15).unwrap_or(false);
                let at_hi = b.1.map(|hi| x[i] >= hi - 1e-15).unwrap_or(false);
                if (at_lo && g[i] > 0.0) || (at_hi && g[i] < 0.0) {
                    0.0
                } else {
                    g[i]
                }
            })
            .collect();
        norm_inf(&pg) <= GTOL
    };

    OptResult {
        x,
        fun: fx,
        success,
        iterations: max_iter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadratic_bowl() {
        let out = minimize(|x| (x[0] - 0.4).powi(2) + (x[1] + 1.2).powi(2), &[0.0, 0.0]);
        assert!(out.success);
        assert!((out.x[0] - 0.4).abs() < 1e-6, "{:?}", out.x);
        assert!((out.x[1] + 1.2).abs() < 1e-6, "{:?}", out.x);
    }

    #[test]
    fn rosenbrock() {
        let out = minimize(
            |x| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2),
            &[-1.2, 1.0],
        );
        assert!((out.x[0] - 1.0).abs() < 1e-3, "{:?}", out.x);
        assert!((out.x[1] - 1.0).abs() < 1e-3, "{:?}", out.x);
    }

    #[test]
    fn bounded_hits_the_wall() {
        // Unconstrained minimum is at (-1, -1); the box pins it at (0, 0).
        let out = minimize_bounded(
            |x| (x[0] + 1.0).powi(2) + (x[1] + 1.0).powi(2),
            &[0.1, 0.1],
            &[(Some(0.0), Some(1.0))],
        );
        assert!(out.x[0].abs() < 1e-6, "{:?}", out.x);
        assert!(out.x[1].abs() < 1e-6, "{:?}", out.x);
    }

    #[test]
    fn bounded_interior_minimum() {
        let out = minimize_bounded(
            |x| (x[0] - 0.4).powi(2) + (x[1] - 0.3).powi(2),
            &[0.1, 0.1],
            &[(Some(0.0), Some(1.0)), (Some(0.0), Some(1.0))],
        );
        assert!((out.x[0] - 0.4).abs() < 1e-6, "{:?}", out.x);
        assert!((out.x[1] - 0.3).abs() < 1e-6, "{:?}", out.x);
        assert!(out.fun < 1e-12);
    }

    /// A quasi-Newton direction can point out of the box even for a variable
    /// that is *not* in the active set. Clamping the direction (rather than
    /// only the active set) is what keeps the feasible step from collapsing to
    /// zero and stranding the search in a corner.
    #[test]
    fn escapes_a_box_corner() {
        // A coupled residual pair with its root at (0.584, 0.754), started
        // from a corner of the box so the search has to walk off both bounds.
        let f = |x: &[f64]| {
            let a = 3.0 * x[0] + 2.0 * x[1] - 3.26;
            let b = x[0] - x[1] + 0.17;
            a * a + b * b
        };
        for start in [[1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.5, 0.5]] {
            let out = minimize_bounded(f, &start, &[(Some(0.0), Some(1.0))]);
            assert!(
                out.fun < 1e-12,
                "stalled from {start:?} at fun={} x={:?}",
                out.fun,
                out.x
            );
            assert!(
                (out.x[0] - 0.584).abs() < 1e-5,
                "from {start:?}: {:?}",
                out.x
            );
            assert!(
                (out.x[1] - 0.754).abs() < 1e-5,
                "from {start:?}: {:?}",
                out.x
            );
        }
    }

    /// A least-squares fit whose minimum sits right on a bound must still be
    /// found exactly, not merely approached.
    #[test]
    fn converges_tightly_on_a_two_curve_fit() {
        // Mimics `cut_corner`: place two points so their difference matches a
        // target vector.
        let target = [1.7, -4.6];
        let f = move |l: &[f64]| {
            let p1 = [10.0 * l[0], 3.0 * l[0] - 2.0];
            let p2 = [2.0 + 6.0 * l[1], -8.0 * l[1] + 1.0];
            let d = [p2[0] - p1[0], p2[1] - p1[1]];
            (d[0] - target[0]).powi(2) + (d[1] - target[1]).powi(2)
        };
        let out = minimize_bounded(
            f,
            &[0.5, 0.5],
            &[(Some(0.0), Some(1.0)), (Some(0.0), Some(1.0))],
        );
        assert!(out.fun < 1e-12, "fun={} x={:?}", out.fun, out.x);
    }

    /// A single bound pair must broadcast to every variable, as scipy does.
    #[test]
    fn single_bound_broadcasts() {
        let out = minimize_bounded(
            |x| (x[0] + 1.0).powi(2) + (x[1] + 1.0).powi(2),
            &[0.1, 0.1],
            &[(Some(0.0), Some(1.0))],
        );
        assert!(
            out.x[1] >= -1e-12,
            "second variable escaped the bound: {:?}",
            out.x
        );
    }
}

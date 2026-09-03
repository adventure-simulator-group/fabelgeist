//! Polynomial helpers -- the parts of `numpy.poly1d` / `svgpathtools.polytools`
//! that the pattern code relies on.
//!
//! Coefficients are stored in numpy order: `p[0] * x^n + ... + p[n]`.

/// `svgpathtools.misctools.isclose` -- `abs(a - b) < atol + rtol * abs(b)`.
pub fn isclose(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-8 + 1e-5 * b.abs()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }

    fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }

    fn mul(self, o: Self) -> Self {
        Self::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }

    fn div(self, o: Self) -> Self {
        let d = o.re * o.re + o.im * o.im;
        Self::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }

    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
}

/// Strip leading (near-)zero coefficients, as `np.roots` does.
fn trim(p: &[f64]) -> Vec<f64> {
    let first = p.iter().position(|c| *c != 0.0).unwrap_or(p.len());
    p[first..].to_vec()
}

/// Evaluate a polynomial in numpy coefficient order.
pub fn polyval(p: &[f64], x: f64) -> f64 {
    p.iter().fold(0.0, |acc, c| acc * x + c)
}

fn polyval_c(p: &[f64], x: Complex) -> Complex {
    p.iter().fold(Complex::new(0.0, 0.0), |acc, c| {
        acc.mul(x).add(Complex::new(*c, 0.0))
    })
}

/// Derivative of a polynomial (numpy coefficient order).
pub fn polyder(p: &[f64]) -> Vec<f64> {
    let n = p.len();
    if n <= 1 {
        return vec![0.0];
    }
    (0..n - 1).map(|i| p[i] * (n - 1 - i) as f64).collect()
}

/// All complex roots via Durand-Kerner iteration.
///
/// This stands in for `numpy.roots` (a companion-matrix eigensolve). Both are
/// backward-stable for the low degrees used here (<= 4), and the callers only
/// keep roots that pass an `isclose` filter afterwards.
fn roots_complex(p: &[f64]) -> Vec<Complex> {
    let p = trim(p);
    if p.len() <= 1 {
        return vec![];
    }

    // Drop trailing zeros -> roots at the origin.
    let mut trailing = 0;
    while p.len() - trailing > 1 && p[p.len() - 1 - trailing] == 0.0 {
        trailing += 1;
    }
    let core = &p[..p.len() - trailing];
    let mut out: Vec<Complex> = (0..trailing).map(|_| Complex::new(0.0, 0.0)).collect();

    let n = core.len() - 1;
    if n == 0 {
        return out;
    }

    // Monic form.
    let lead = core[0];
    let monic: Vec<f64> = core.iter().map(|c| c / lead).collect();

    // Spread the initial guesses around a circle that encloses all roots.
    let bound = 1.0 + monic[1..].iter().fold(0.0_f64, |m, c| m.max(c.abs()));
    let mut z: Vec<Complex> = (0..n)
        .map(|k| {
            let ang = 2.0 * std::f64::consts::PI * (k as f64) / (n as f64) + 0.4;
            Complex::new(bound * 0.5 * ang.cos(), bound * 0.5 * ang.sin())
        })
        .collect();

    for _ in 0..500 {
        let mut max_step = 0.0_f64;
        for i in 0..n {
            let mut denom = Complex::new(1.0, 0.0);
            for j in 0..n {
                if i != j {
                    denom = denom.mul(z[i].sub(z[j]));
                }
            }
            if denom.abs() < 1e-300 {
                continue;
            }
            let step = polyval_c(&monic, z[i]).div(denom);
            z[i] = z[i].sub(step);
            max_step = max_step.max(step.abs());
        }
        if max_step < 1e-15 {
            break;
        }
    }

    // Polish real-looking roots with Newton on the original polynomial, which
    // tightens the values Durand-Kerner leaves at ~1e-12.
    let der = polyder(&monic);
    for zi in z.iter_mut() {
        if zi.im.abs() < 1e-7 {
            let mut x = zi.re;
            for _ in 0..50 {
                let f = polyval(&monic, x);
                let d = polyval(&der, x);
                if d.abs() < 1e-300 {
                    break;
                }
                let step = f / d;
                x -= step;
                if step.abs() < 1e-16 * (1.0 + x.abs()) {
                    break;
                }
            }
            if x.is_finite() {
                *zi = Complex::new(x, 0.0);
            }
        }
    }

    out.extend(z);
    out
}

/// `svgpathtools.polytools.polyroots(p, realroots=True, condition=...)`.
///
/// Returns the real roots satisfying `condition`, with near-duplicates removed
/// the same way the reference does.
pub fn polyroots_real<F: Fn(f64) -> bool>(p: &[f64], condition: F) -> Vec<f64> {
    let roots: Vec<f64> = roots_complex(p)
        .into_iter()
        .filter(|r| isclose(r.im, 0.0))
        .map(|r| r.re)
        .filter(|r| condition(*r))
        .collect();

    // The reference drops the *first* element of each close pair (it collects
    // `combinations` indices and filters by position in that enumeration).
    let mut duplicate_idx = Vec::new();
    let mut idx = 0usize;
    for i in 0..roots.len() {
        for j in (i + 1)..roots.len() {
            if isclose(roots[i], roots[j]) {
                duplicate_idx.push(idx);
            }
            idx += 1;
        }
    }

    roots
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !duplicate_idx.contains(i))
        .map(|(_, r)| r)
        .collect()
}

/// `svgpathtools.polytools.polyroots01` -- real roots in `[0, 1]`.
pub fn polyroots01(p: &[f64]) -> Vec<f64> {
    polyroots_real(p, |t| (0.0..=1.0).contains(&t))
}

/// Real roots strictly inside `(0, 1)` -- the filter used for curve extrema.
pub fn polyroots_open01(p: &[f64]) -> Vec<f64> {
    polyroots_real(p, |t| 0.0 < t && t < 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadratic_roots() {
        // x^2 - 3x + 2 -> 1, 2
        let mut r = polyroots_real(&[1.0, -3.0, 2.0], |_| true);
        r.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(r.len(), 2);
        assert!((r[0] - 1.0).abs() < 1e-12, "{r:?}");
        assert!((r[1] - 2.0).abs() < 1e-12, "{r:?}");
    }

    #[test]
    fn cubic_roots() {
        // (x - 0.25)(x - 0.5)(x + 3)
        let p = [1.0, 2.25, -2.125, 0.375];
        let mut r = polyroots01(&p);
        r.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(r.len(), 2, "{r:?}");
        assert!((r[0] - 0.25).abs() < 1e-10, "{r:?}");
        assert!((r[1] - 0.5).abs() < 1e-10, "{r:?}");
    }

    #[test]
    fn complex_roots_are_dropped() {
        // x^2 + 1 has no real roots
        assert!(polyroots_real(&[1.0, 0.0, 1.0], |_| true).is_empty());
    }

    #[test]
    fn linear_root() {
        let r = polyroots01(&[2.0, -1.0]);
        assert_eq!(r.len(), 1);
        assert!((r[0] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn derivative() {
        // 3x^2 + 2x + 1 -> 6x + 2
        assert_eq!(polyder(&[3.0, 2.0, 1.0]), vec![6.0, 2.0]);
    }
}

//! Sample cubic boundaries from their exact structural intersections.
use super::{CurveQuality, Detail, PlanarCut, PlanarPoint, adaptive_curve, append_curve};

const ROOT_ROUNDOFF_ULPS: f64 = 64.0;
const ROOT_BISECTION_STEPS: usize = 56;

pub(crate) fn partitioned_cubic(
    controls: [PlanarPoint; 4],
    quality: CurveQuality,
    detail: Detail,
    cuts: &[PlanarCut],
) -> Vec<PlanarPoint> {
    let evaluate = |t| std::array::from_fn(|axis| evaluate_scalar(controls.map(|p| p[axis]), t));
    let mut breaks = vec![(0.0, controls[0]), (1.0, controls[3])];
    for &cut in cuts {
        for t in roots(controls.map(|p| cut.distance(p))) {
            if t <= 0.0 || t >= 1.0 {
                continue;
            }
            let mut point = evaluate(t);
            let (axis, ordinate) = cut.ordinate(point);
            point[axis] = ordinate;
            if cut.active(&[point]) {
                breaks.push((t, point));
            }
        }
    }
    breaks.sort_by(|a, b| a.0.total_cmp(&b.0));
    breaks.dedup_by(|a, b| (a.0 - b.0).abs() <= ROOT_ROUNDOFF_ULPS * f64::EPSILON);
    let mut result = Vec::new();
    for pair in breaks.windows(2) {
        let [(start, a), (end, b)] = [pair[0], pair[1]];
        append_curve(
            &mut result,
            adaptive_curve(
                |u| {
                    if u == 0.0 {
                        a
                    } else if u == 1.0 {
                        b
                    } else {
                        evaluate(start + (end - start) * u)
                    }
                },
                quality,
                detail,
            ),
        );
    }
    result
}

fn evaluate_scalar([a, b, c, d]: [f64; 4], t: f64) -> f64 {
    let lerp = |a, b| a * (1.0 - t) + b * t;
    lerp(lerp(lerp(a, b), lerp(b, c)), lerp(lerp(b, c), lerp(c, d)))
}

fn roots(values: [f64; 4]) -> Vec<f64> {
    let differences = std::array::from_fn::<_, 3, _>(|i| values[i + 1] - values[i]);
    let [a, b, c] = differences;
    let mut monotone = vec![0.0, 1.0];
    monotone.extend(
        quadratic_roots(a - 2.0 * b + c, 2.0 * (b - a), a)
            .into_iter()
            .filter(|t| *t > 0.0 && *t < 1.0),
    );
    monotone.sort_by(f64::total_cmp);
    monotone.dedup_by(|a, b| (*a - *b).abs() <= ROOT_ROUNDOFF_ULPS * f64::EPSILON);
    let roundoff =
        ROOT_ROUNDOFF_ULPS * f64::EPSILON * values.iter().map(|v| v.abs()).fold(0.0, f64::max);
    let mut result: Vec<_> = monotone
        .iter()
        .copied()
        .filter(|&t| evaluate_scalar(values, t).abs() <= roundoff)
        .collect();
    for pair in monotone.windows(2) {
        let [mut lo, mut hi] = [pair[0], pair[1]];
        let left = evaluate_scalar(values, lo);
        let right = evaluate_scalar(values, hi);
        if left.abs() <= roundoff || right.abs() <= roundoff {
            continue;
        }
        if !((left < 0.0 && right > 0.0) || (left > 0.0 && right < 0.0)) {
            continue;
        }
        for _ in 0..ROOT_BISECTION_STEPS {
            let mid = (lo + hi) / 2.0;
            if evaluate_scalar(values, mid).is_sign_negative() == left.is_sign_negative() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        result.push((lo + hi) / 2.0);
    }
    result
}

#[cfg(test)]
#[path = "partitioned_curve_tests.rs"]
mod tests;

fn quadratic_roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    if a == 0.0 {
        return if b == 0.0 { vec![] } else { vec![-c / b] };
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return vec![];
    }
    let q = -0.5 * (b + discriminant.sqrt().copysign(b));
    if q == 0.0 {
        vec![-b / (2.0 * a)]
    } else {
        vec![q / a, c / q]
    }
}

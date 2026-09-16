//! Error bounds for replacing unresolvable groove tails by broad-face fans.
use super::*;

pub(super) const BLADE_SURFACE_ERROR: f64 = 0.00004;
const GROOVE_ADDITIONAL_NORMAL_ERROR: f64 = 0.02;

pub(super) fn envelope_floor(p: &BladeProfile<'_>, detail: Detail) -> Result<f64, String> {
    let Some(f) = p.fuller else {
        return Ok(0.0);
    };
    let strip_width = f.mouth_width.get()
        * f.floor_width_ratio
            .get()
            .min((1.0 - f.floor_width_ratio.get()) / 2.0);
    let minimum = FLOAT32_STRIP_SEPARATION / strip_width;
    if minimum >= 1.0 || f.depth.get() * minimum.powi(2) > detail.error(BLADE_SURFACE_ERROR) / 2.0 {
        return Err("fuller strips cannot be resolved within the surface-error budget".into());
    }
    Ok(minimum)
}

// The broad face has the same body curvature and point-thickness approximation.
// Bound additional error from removing the recess separately from that existing
// chord error, and check both longitudinal and transverse analytic derivatives.
pub(super) fn check(
    interval: [f64; 2],
    side: usize,
    quad: &[Point; 4],
    ring: impl Fn(f64) -> Vec<Point>,
    detail: Detail,
) -> Result<(), String> {
    let flat = match side {
        1..=5 => [1, 6],
        8..=12 => [8, 13],
        _ => return Ok(()),
    };
    let [a, b] = interval;
    let left = ring(a);
    let right = ring(b);
    let broad = unit(cross(
        sub(left[flat[1]], left[flat[0]]),
        sub(right[flat[0]], left[flat[0]]),
    ))?;
    let mut vertices: Vec<_> = quad
        .iter()
        .copied()
        .zip([[a, 0.0], [a, 1.0], [b, 1.0], [b, 0.0]])
        .collect();
    vertices.dedup_by(|a, b| a.0 == b.0);
    if vertices.first().map(|v| v.0) == vertices.last().map(|v| v.0) {
        vertices.pop();
    }
    let check = NormalCheck {
        interval,
        side,
        flat,
        ring,
        broad,
        tolerance: detail.error(GROOVE_ADDITIONAL_NORMAL_ERROR),
    };
    for i in 1..vertices.len().saturating_sub(1) {
        let tri = [vertices[0], vertices[i], vertices[i + 1]];
        let normal = unit(cross(sub(tri[1].0, tri[0].0), sub(tri[2].0, tri[0].0)))?;
        for weights in [
            [1.0 / 3.0; 3],
            [0.6, 0.2, 0.2],
            [0.2, 0.6, 0.2],
            [0.2, 0.2, 0.6],
        ] {
            let param: [f64; 2] =
                std::array::from_fn(|axis| (0..3).map(|j| tri[j].1[axis] * weights[j]).sum());
            check.compare(normal, param)?;
        }
    }
    // Even a strip omitted completely has a bounded analytic slope toward the
    // broad face. This covers the region after the final triangular fan.
    if vertices.len() < 3 {
        for t in [0.25, 0.5, 0.75] {
            check.compare(broad, [a + (b - a) * t, 0.5])?;
        }
    }
    Ok(())
}

struct NormalCheck<F> {
    interval: [f64; 2],
    side: usize,
    flat: [usize; 2],
    ring: F,
    broad: Point,
    tolerance: f64,
}
impl<F: Fn(f64) -> Vec<Point>> NormalCheck<F> {
    fn compare(&self, normal: Point, param: [f64; 2]) -> Result<(), String> {
        let Self {
            interval,
            side,
            flat,
            ring,
            broad,
            tolerance,
        } = self;
        let [y, u] = param;
        let h = (interval[1] - interval[0]) * 0.0001;
        let at = ring(y);
        let before = ring(y - h);
        let after = ring(y + h);
        let sampled_normal = |indices: [usize; 2]| {
            let sample = |r: &[Point]| add(mul(r[indices[0]], 1.0 - u), mul(r[indices[1]], u));
            unit(cross(
                sub(at[indices[1]], at[indices[0]]),
                sub(sample(&after), sample(&before)),
            ))
        };
        let reference = sampled_normal(*flat)?;
        let next = (side + 1) % at.len();
        let analytic = if at[*side] == at[next] {
            reference
        } else {
            sampled_normal([*side, next])?
        };
        let base_error = angle(*broad, reference);
        if angle(normal, analytic) > base_error + tolerance
            || angle(analytic, reference) > *tolerance
        {
            return Err("fuller tail cannot be simplified within the normal-error budget".into());
        }
        Ok(())
    }
}
fn unit(v: Point) -> Result<Point, String> {
    let length = magnitude(v);
    if !length.is_finite() || length == 0.0 {
        return Err("unresolved blade surface normal".into());
    }
    Ok(mul(v, 1.0 / length))
}
fn angle(a: Point, b: Point) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}

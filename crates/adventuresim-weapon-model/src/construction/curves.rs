//! Geometric error budgets for curve and section sampling.
use super::{Detail, PlanarPoint};

#[cfg(test)]
#[path = "curve_tests.rs"]
mod tests;

#[derive(Clone, Copy)]
pub(crate) struct CurveQuality {
    pub(crate) minimum_segments: usize,
    pub(crate) max_chord: f64,
    pub(crate) max_deviation: f64,
}

impl Default for CurveQuality {
    fn default() -> Self {
        Self {
            minimum_segments: 8,
            max_chord: f64::INFINITY,
            max_deviation: f64::INFINITY,
        }
    }
}

fn point_segment_distance(p: PlanarPoint, a: PlanarPoint, b: PlanarPoint) -> f64 {
    let delta = [b[0] - a[0], b[1] - a[1]];
    let denominator = delta[0] * delta[0] + delta[1] * delta[1];
    let t = if denominator == 0.0 {
        0.0
    } else {
        ((p[0] - a[0]) * delta[0] + (p[1] - a[1]) * delta[1]) / denominator
    }
    .clamp(0.0, 1.0);
    (p[0] - a[0] - delta[0] * t).hypot(p[1] - a[1] - delta[1] * t)
}

fn refine(
    evaluate: &impl Fn(f64) -> PlanarPoint,
    quality: CurveQuality,
    samples: &mut Vec<PlanarPoint>,
    span: [f64; 2],
    ends: [PlanarPoint; 2],
    depth: usize,
) {
    let [t0, t1] = span;
    let [p0, p1] = ends;
    let midpoint_t = (t0 + t1) / 2.0;
    let midpoint = evaluate(midpoint_t);
    let chord = (p1[0] - p0[0]).hypot(p1[1] - p0[1]);
    let deviation = [
        evaluate(t0 * 0.75 + t1 * 0.25),
        midpoint,
        evaluate(t0 * 0.25 + t1 * 0.75),
    ]
    .into_iter()
    .map(|p| point_segment_distance(p, p0, p1))
    .fold(0.0, f64::max);
    if depth < 8 && (chord > quality.max_chord || deviation > quality.max_deviation) {
        refine(
            evaluate,
            quality,
            samples,
            [t0, midpoint_t],
            [p0, midpoint],
            depth + 1,
        );
        refine(
            evaluate,
            quality,
            samples,
            [midpoint_t, t1],
            [midpoint, p1],
            depth + 1,
        );
    } else {
        samples.push(p1);
    }
}

pub(crate) fn adaptive_curve(
    evaluate: impl Fn(f64) -> PlanarPoint,
    mut quality: CurveQuality,
    detail: Detail,
) -> Vec<PlanarPoint> {
    quality.minimum_segments = detail.samples(quality.minimum_segments, 2);
    quality.max_chord = detail.error(quality.max_chord);
    quality.max_deviation = detail.error(quality.max_deviation);
    let mut samples = vec![evaluate(0.0)];
    for i in 0..quality.minimum_segments {
        let t0 = i as f64 / quality.minimum_segments as f64;
        let t1 = (i + 1) as f64 / quality.minimum_segments as f64;
        let p0 = *samples.last().unwrap();
        refine(
            &evaluate,
            quality,
            &mut samples,
            [t0, t1],
            [p0, evaluate(t1)],
            0,
        );
    }
    let minimum_spacing = 1e-10_f64.max(
        [quality.max_chord, quality.max_deviation]
            .into_iter()
            .filter(|v| v.is_finite())
            .reduce(f64::min)
            .unwrap_or(1.0)
            * 1e-7,
    );
    let mut compact = vec![samples[0]];
    for &point in &samples[1..samples.len() - 1] {
        let previous = *compact.last().unwrap();
        if (point[0] - previous[0]).hypot(point[1] - previous[1]) > minimum_spacing {
            compact.push(point);
        }
    }
    let endpoint = *samples.last().unwrap();
    let previous = *compact.last().unwrap();
    if (endpoint[0] - previous[0]).hypot(endpoint[1] - previous[1]) <= minimum_spacing
        && compact.len() > 1
    {
        *compact.last_mut().unwrap() = endpoint;
    } else {
        compact.push(endpoint);
    }
    compact
}

pub(crate) fn cubic_bezier(
    [p0, p1, p2, p3]: [PlanarPoint; 4],
    quality: CurveQuality,
    detail: Detail,
) -> Vec<PlanarPoint> {
    adaptive_curve(
        |t| {
            let u = 1.0 - t;
            std::array::from_fn(|i| {
                u.powi(3) * p0[i]
                    + 3.0 * u * u * t * p1[i]
                    + 3.0 * u * t * t * p2[i]
                    + t.powi(3) * p3[i]
            })
        },
        quality,
        detail,
    )
}

pub(crate) fn append_curve(target: &mut Vec<PlanarPoint>, mut points: Vec<PlanarPoint>) {
    if !target.is_empty() && !points.is_empty() {
        points.remove(0);
    }
    target.extend(points);
}

pub(crate) fn subdivide<const N: usize>(points: &[[f64; N]], maximum_chord: f64) -> Vec<[f64; N]> {
    let mut result = vec![points[0]];
    for pair in points.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        let length = (0..N).map(|i| (b[i] - a[i]).powi(2)).sum::<f64>().sqrt();
        let count = (length / maximum_chord).ceil().max(1.0) as usize;
        for i in 1..=count {
            let t = i as f64 / count as f64;
            result.push(std::array::from_fn(|axis| {
                a[axis] + (b[axis] - a[axis]) * t
            }));
        }
    }
    result
}

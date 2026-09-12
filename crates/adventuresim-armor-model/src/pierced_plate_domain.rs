//! Closed-loop piercing in an authored plate chart, independent of its carrier.
use crate::{DesignError, GenerateError};
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
const SAMPLE_SPACING_MM: f64 = 5.0;
const GRID_EDGE_CLEARANCE_MM: f64 = 1.0;
pub(crate) const MINIMUM_WEB_MM: f64 = 3.0;
const CORNER_STEPS: usize = 4;

pub(crate) struct PiercedDomain {
    pub points: Vec<[f64; 2]>,
    pub indices: Vec<u32>,
}
impl PiercedDomain {
    pub fn new(
        outer: &[[f64; 2]],
        holes: &[Vec<[f64; 2]>],
        interior: impl IntoIterator<Item = [f64; 2]>,
    ) -> Result<Self, GenerateError> {
        for hole in holes {
            if hole
                .iter()
                .any(|p| !inside(*p, outer) || edge_distance(*p, outer) < MINIMUM_WEB_MM)
            {
                return Err(DesignError::VisorOpeningSpacing.into());
            }
        }
        if holes
            .iter()
            .enumerate()
            .any(|(i, a)| holes[i + 1..].iter().any(|b| openings_conflict(a, b)))
        {
            return Err(DesignError::VisorOpeningSpacing.into());
        }
        let (samples, constraints) = sample_domain(outer, holes, interior);
        let cdt =
            ConstrainedDelaunayTriangulation::<Point2<f64>>::bulk_load_cdt(samples, constraints)
                .map_err(|_| GenerateError::InvalidSurface)?;
        let points: Vec<_> = cdt
            .vertices()
            .map(|v| [v.position().x, v.position().y])
            .collect();
        let mut indices = Vec::new();
        for face in cdt.inner_faces() {
            let ids = face.vertices().map(|v| v.fix().index());
            let [a, b, c] = ids.map(|i| points[i]);
            // Interpolated boundary samples can be nearly collinear in f64.
            // Such a CDT sliver has no material area in the authored domain.
            const MINIMUM_DOMAIN_DOUBLE_AREA_MM2: f64 = 1e-7;
            let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            if area.abs() < MINIMUM_DOMAIN_DOUBLE_AREA_MM2 {
                continue;
            }
            let center =
                std::array::from_fn(|axis| ids.iter().map(|i| points[*i][axis]).sum::<f64>() / 3.0);
            if inside(center, outer) && holes.iter().all(|h| !inside(center, h)) {
                // Surface coordinates run down the face; reverse the CDT's CCW faces.
                indices.extend([ids[0] as u32, ids[2] as u32, ids[1] as u32]);
            }
        }
        Ok(Self { points, indices })
    }
}

fn sample_domain(
    outer: &[[f64; 2]],
    holes: &[Vec<[f64; 2]>],
    interior: impl IntoIterator<Item = [f64; 2]>,
) -> (Vec<Point2<f64>>, Vec<[usize; 2]>) {
    let mut points = Vec::new();
    let mut constraints = Vec::new();
    for boundary in std::iter::once(outer).chain(holes.iter().map(Vec::as_slice)) {
        let start = points.len();
        for (a, b) in edges(boundary) {
            let steps = (distance(*a, *b) / SAMPLE_SPACING_MM).ceil() as usize;
            for i in 0..steps.max(1) {
                let t = i as f64 / steps.max(1) as f64;
                points.push(Point2::new(
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                ));
            }
        }
        let end = points.len();
        constraints.extend((start..end).map(|i| [i, if i + 1 == end { start } else { i + 1 }]));
    }
    for p in interior {
        if inside(p, outer)
            && edge_distance(p, outer) > GRID_EDGE_CLEARANCE_MM
            && holes
                .iter()
                .all(|h| !inside(p, h) && edge_distance(p, h) > GRID_EDGE_CLEARANCE_MM)
        {
            points.push(Point2::new(p[0], p[1]));
        }
    }
    (points, constraints)
}

pub(crate) fn rounded_slot(
    center: [f64; 2],
    width: f64,
    height: f64,
    angle: f64,
    rounding: f64,
) -> Vec<[f64; 2]> {
    let radius = width.min(height) * 0.5 * rounding;
    let mut points = Vec::new();
    for (corner, sign) in [[1.0, 1.0], [-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0]]
        .iter()
        .enumerate()
    {
        let steps = if radius == 0.0 { 0 } else { CORNER_STEPS };
        for step in 0..=steps {
            let theta =
                (corner as f64 + step as f64 / CORNER_STEPS as f64) * std::f64::consts::FRAC_PI_2;
            let x = sign[0] * (width * 0.5 - radius) + radius * theta.cos();
            let y = sign[1] * (height * 0.5 - radius) + radius * theta.sin();
            let p = [
                center[0] + x * angle.cos() - y * angle.sin(),
                center[1] + x * angle.sin() + y * angle.cos(),
            ];
            if points.last().is_none_or(|last| distance(*last, p) > 1e-8) {
                points.push(p);
            }
        }
    }
    if distance(points[0], *points.last().unwrap()) < 1e-8 {
        points.pop();
    }
    points
}

fn edges(polygon: &[[f64; 2]]) -> impl Iterator<Item = (&[f64; 2], &[f64; 2])> {
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
}

fn inside(p: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let mut result = false;
    for (a, b) in edges(polygon) {
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            result = !result;
        }
    }
    result
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

pub(crate) fn edge_distance(p: [f64; 2], polygon: &[[f64; 2]]) -> f64 {
    edges(polygon)
        .map(|(a, b)| {
            let v = [b[0] - a[0], b[1] - a[1]];
            let t = (((p[0] - a[0]) * v[0] + (p[1] - a[1]) * v[1]) / (v[0] * v[0] + v[1] * v[1]))
                .clamp(0.0, 1.0);
            distance(p, [a[0] + t * v[0], a[1] + t * v[1]])
        })
        .fold(f64::INFINITY, f64::min)
}

/// Test containment, boundary crossing and the remaining metal between holes.
fn openings_conflict(a: &[[f64; 2]], b: &[[f64; 2]]) -> bool {
    if a.iter()
        .any(|p| inside(*p, b) || edge_distance(*p, b) < MINIMUM_WEB_MM)
        || b.iter()
            .any(|p| inside(*p, a) || edge_distance(*p, a) < MINIMUM_WEB_MM)
    {
        return true;
    }
    fn side(a: [f64; 2], b: [f64; 2], p: [f64; 2]) -> f64 {
        (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
    }
    edges(a).any(|(p, q)| {
        edges(b).any(|(r, s)| {
            side(*p, *q, *r) * side(*p, *q, *s) < 0.0 && side(*r, *s, *p) * side(*r, *s, *q) < 0.0
        })
    })
}

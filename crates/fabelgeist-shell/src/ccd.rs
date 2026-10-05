//! Conservative advancement for linearly moving thick cloth primitives.
//!
//! Bound distance decrease by the maximum relative vertex displacement.
//! Each advance is shorter than (distance - clearance) / that bound, so it
//! cannot step over contact. Exhausting the iteration budget returns the last
//! safe time; it must never be interpreted as "no collision".
//! See https://ipc-sim.github.io/C-IPC/ (additive conservative advancement).
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::ConstraintGradient;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Pair {
    VertexTriangle,
    EdgeEdge,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Contact {
    /// A conservative lower bound for the first contact, in [0, 1].
    pub time: f64,
    pub weights: [ConstraintGradient; 4],
    pub normal: Vec3,
    pub distance: f64,
}

type V = [f64; 3];
fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] - b[i])
}
fn mul(a: V, s: f64) -> V {
    a.map(|x| x * s)
}
fn dot(a: V, b: V) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn norm(a: V) -> f64 {
    dot(a, a).sqrt()
}
fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn convert(p: Vec3) -> V {
    [p.x as f64, p.y as f64, p.z as f64]
}

fn segment(p: V, a: V, b: V) -> (V, f64) {
    let edge = sub(b, a);
    let length = dot(edge, edge);
    let t = if length > 0.0 {
        (dot(sub(p, a), edge) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (add(a, mul(edge, t)), t)
}

/// Return the closest point with barycentric coordinates. Edge tests also
/// handle degenerate triangles; no division by their area is required.
fn triangle(p: V, a: V, b: V, c: V) -> (V, [f64; 3]) {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let normal = cross(ab, ac);
    let area2 = dot(normal, normal);
    if area2 > 0.0 {
        let projected = sub(p, mul(normal, dot(sub(p, a), normal) / area2));
        let ap = sub(projected, a);
        let v = dot(cross(ap, ac), normal) / area2;
        let w = dot(cross(ab, ap), normal) / area2;
        let u = 1.0 - v - w;
        if u >= 0.0 && v >= 0.0 && w >= 0.0 {
            return (projected, [u, v, w]);
        }
    }
    let (q, t) = segment(p, a, b);
    let mut best = (q, [1.0 - t, t, 0.0]);
    for (q, bary) in [
        {
            let (q, t) = segment(p, b, c);
            (q, [0.0, 1.0 - t, t])
        },
        {
            let (q, t) = segment(p, c, a);
            (q, [t, 0.0, 1.0 - t])
        },
    ] {
        if norm(sub(p, q)) < norm(sub(p, best.0)) {
            best = (q, bary);
        }
    }
    best
}

fn edges(a: V, b: V, c: V, d: V) -> (f64, f64) {
    let (u, v, r) = (sub(b, a), sub(d, c), sub(a, c));
    let (aa, bb, cc, dd, ee) = (dot(u, u), dot(u, v), dot(v, v), dot(u, r), dot(v, r));
    if aa == 0.0 && cc == 0.0 {
        return (0.0, 0.0);
    }
    if aa == 0.0 {
        return (0.0, (ee / cc).clamp(0.0, 1.0));
    }
    if cc == 0.0 {
        return ((-dd / aa).clamp(0.0, 1.0), 0.0);
    }
    let determinant = norm(cross(u, v)).powi(2);
    let mut s = if determinant > 1e-28 * aa * cc {
        ((bb * ee - cc * dd) / determinant).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = (bb * s + ee) / cc;
    if t < 0.0 {
        t = 0.0;
        s = (-dd / aa).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((bb - dd) / aa).clamp(0.0, 1.0);
    }
    (s, t)
}

fn sample(pair: Pair, p: [V; 4], time: f64, fallback: V) -> Contact {
    let weights = match pair {
        Pair::VertexTriangle => {
            let (_, bary) = triangle(p[0], p[1], p[2], p[3]);
            [1.0, -bary[0], -bary[1], -bary[2]]
        }
        Pair::EdgeEdge => {
            let (s, t) = edges(p[0], p[1], p[2], p[3]);
            [1.0 - s, s, -(1.0 - t), -t]
        }
    };
    let delta = (0..4).fold([0.0; 3], |sum, i| add(sum, mul(p[i], weights[i])));
    let distance = norm(delta);
    let direction = if distance > 1e-15 {
        mul(delta, 1.0 / distance)
    } else if norm(fallback) > 1e-15 {
        mul(fallback, 1.0 / norm(fallback))
    } else {
        let n = match pair {
            Pair::VertexTriangle => cross(sub(p[2], p[1]), sub(p[3], p[1])),
            Pair::EdgeEdge => cross(sub(p[1], p[0]), sub(p[3], p[2])),
        };
        if norm(n) > 1e-15 {
            mul(n, 1.0 / norm(n))
        } else {
            [1.0, 0.0, 0.0]
        }
    };
    Contact {
        time,
        weights: weights.map(ConstraintGradient::from),
        normal: Vec3::new(
            direction[0] as f32,
            direction[1] as f32,
            direction[2] as f32,
        ),
        distance,
    }
}

/// Closest contact at the end of an interval, including collapsed primitives.
pub(crate) fn proximity(pair: Pair, end: [Vec3; 4], start: [Vec3; 4]) -> Contact {
    let previous = sample(pair, start.map(convert), 0.0, [0.0; 3]);
    sample(pair, end.map(convert), 1.0, convert(previous.normal))
}

/// Detect first contact over the whole interval, not just at its endpoints.
/// Linear vertex paths include translating, rotating and deforming triangles.
/// Geometry is evaluated in f64 even though storage/rendering use f32.
pub(crate) fn sweep(
    pair: Pair,
    start: [Vec3; 4],
    end: [Vec3; 4],
    clearance: f32,
) -> Option<Contact> {
    sweep_with_budget(pair, start, end, clearance, 256)
}

fn sweep_with_budget(
    pair: Pair,
    start: [Vec3; 4],
    end: [Vec3; 4],
    clearance: f32,
    budget: usize,
) -> Option<Contact> {
    let start = start.map(convert);
    let end = end.map(convert);
    let velocity = std::array::from_fn::<_, 4, _>(|i| sub(end[i], start[i]));
    let speed = match pair {
        Pair::VertexTriangle => (1..4)
            .map(|i| norm(sub(velocity[0], velocity[i])))
            .fold(0.0, f64::max),
        Pair::EdgeEdge => (0..2)
            .flat_map(|i| (2..4).map(move |j| norm(sub(velocity[i], velocity[j]))))
            .fold(0.0, f64::max),
    };
    let mut contact = sample(pair, start, 0.0, [0.0; 3]);
    let initial = contact.distance;
    let clearance = clearance as f64;
    if initial <= clearance {
        return Some(contact);
    }
    if speed <= 1e-15 {
        return None;
    }
    // Keep a small fraction of the initial gap. This produces a safe lower
    // bound and avoids asymptotically taking infinitely many steps at impact.
    let target = clearance + (initial - clearance) * 0.01;
    let mut time = 0.0;
    for _ in 0..budget {
        let distance = contact.distance;
        if distance <= target {
            return Some(contact);
        }
        let safe_step = 0.9 * (distance - clearance) / speed;
        if safe_step >= 1.0 - time {
            return None;
        }
        let next = time + safe_step;
        if next <= time {
            return Some(contact);
        }
        time = next;
        let p = std::array::from_fn(|i| add(start[i], mul(velocity[i], time)));
        contact = sample(pair, p, time, convert(contact.normal));
    }
    Some(contact)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new(x, y, z)
    }
    #[test]
    fn crossing_and_exiting_triangle_is_detected_between_clear_endpoints() {
        let start = [
            p(-1.0, 0.0, 1.0),
            p(-1.0, -1.0, 0.0),
            p(1.0, -1.0, 0.0),
            p(0.0, 1.0, 0.0),
        ];
        let mut end = start;
        end[0] = p(1.2, 0.0, -1.0);
        let hit = sweep(Pair::VertexTriangle, start, end, 0.01).unwrap();
        assert!(hit.time > 0.0 && hit.time < 0.5);
    }
    #[test]
    fn moving_triangle_hits_a_stationary_vertex() {
        let start = [
            p(0.0, 0.0, 0.0),
            p(-1.0, -1.0, -1.0),
            p(1.0, -1.0, -1.0),
            p(0.0, 1.0, -1.0),
        ];
        let mut end = start;
        for p in &mut end[1..] {
            p.z = 1.0;
        }
        let hit = sweep(Pair::VertexTriangle, start, end, 0.01).unwrap();
        assert!(hit.time > 0.4 && hit.time < 0.5);
    }
    #[test]
    fn deforming_triangle_is_checked_throughout_the_interval() {
        let start = [
            p(0.0, 0.0, 0.0),
            p(-1.0, -1.0, -1.0),
            p(1.0, -1.0, -1.0),
            p(0.0, 1.0, -0.5),
        ];
        let mut end = start;
        end[1].z = 0.5;
        end[2].z = 1.5;
        end[3].z = 1.0;
        assert!(sweep(Pair::VertexTriangle, start, end, 0.001).is_some());
    }
    #[test]
    fn fast_edges_cross_with_no_close_endpoints() {
        let start = [
            p(-1.0, 0.0, 0.0),
            p(1.0, 0.0, 0.0),
            p(0.0, -1.0, 10.0),
            p(0.0, 1.0, 10.0),
        ];
        let mut end = start;
        end[2].z = -10.0;
        end[3].z = -10.0;
        let hit = sweep(Pair::EdgeEdge, start, end, 0.001).unwrap();
        assert!(hit.time > 0.49 && hit.time < 0.5);
    }
    #[test]
    fn coplanar_and_degenerate_edge_crossings_are_detected() {
        let start = [
            p(-1.0, 0.0, 0.0),
            p(1.0, 0.0, 0.0),
            p(0.0, 2.0, 0.0),
            p(0.0, 2.0, 0.0),
        ];
        let mut end = start;
        end[2].y = -2.0;
        end[3].y = -2.0;
        assert!(sweep(Pair::EdgeEdge, start, end, 0.001).is_some());
    }
    #[test]
    fn parallel_near_miss_and_common_translation_do_not_collide() {
        let start = [
            p(-1.0, 0.0, 0.0),
            p(1.0, 0.0, 0.0),
            p(-1.0, 0.1, 0.0),
            p(1.0, 0.1, 0.0),
        ];
        let end = start.map(|v| v + p(100.0, 0.0, 0.0));
        assert!(sweep(Pair::EdgeEdge, start, end, 0.01).is_none());
    }
    #[test]
    fn exhausted_budget_returns_a_safe_time_not_a_missed_collision() {
        let start = [
            p(-1.0, 0.0, 0.0),
            p(1.0, 0.0, 0.0),
            p(0.0, -1.0, 1.0),
            p(0.0, 1.0, 1.0),
        ];
        let mut end = start;
        end[2].z = -1.0;
        end[3].z = -1.0;
        let hit = sweep_with_budget(Pair::EdgeEdge, start, end, 0.001, 0).unwrap();
        assert_eq!(hit.time, 0.0);
    }

    #[test]
    fn rotating_triangle_crossing_is_detected_even_when_it_collapses_midstep() {
        let start = [
            p(0.0, 0.0, 0.0),
            p(-1.0, -1.0, -0.5),
            p(1.0, -1.0, -0.5),
            p(0.0, 1.0, -0.5),
        ];
        let mut end = start;
        for vertex in &mut end[1..] {
            vertex.x = -vertex.x;
            vertex.z = -vertex.z;
        }
        let hit = sweep(Pair::VertexTriangle, start, end, 0.001).unwrap();
        assert!(hit.time < 0.5);
        assert!(hit.normal.is_finite());
    }

    #[test]
    fn conservative_advancement_never_misses_sampled_moving_contacts() {
        let mut seed = 0x31415926u32;
        let mut random = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32
        };
        let mut checked = 0;
        for pair in [Pair::VertexTriangle, Pair::EdgeEdge] {
            for _ in 0..128 {
                let start = std::array::from_fn(|_| p(random(), random(), random()));
                let end = std::array::from_fn(|_| p(random(), random(), random()));
                let clearance = 0.02;
                let sampled = (0..=256).any(|step| {
                    let t = step as f64 / 256.0;
                    let at = std::array::from_fn(|i| {
                        add(
                            convert(start[i]),
                            mul(sub(convert(end[i]), convert(start[i])), t),
                        )
                    });
                    sample(pair, at, t, [0.0; 3]).distance < clearance as f64
                });
                if sampled {
                    checked += 1;
                    assert!(
                        sweep(pair, start, end, clearance).is_some(),
                        "missed {pair:?}: {start:?} -> {end:?}"
                    );
                }
            }
        }
        assert!(
            checked > 20,
            "test must exercise enough actual contacts: {checked}"
        );
    }
}

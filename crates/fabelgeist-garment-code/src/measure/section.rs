//! Slicing a closed mesh with a plane.
//!
//! Every measurement in this module is a plane section of the body: a girth is
//! the section's perimeter, a back width is part of it, a level is found by
//! scanning sections up the body until one is narrowest or widest.
//!
//! The sections are returned as **loops**, not as a bag of segments, and that
//! is what makes the rest of the module possible. A horizontal cut across the
//! chest of a body standing with its arms away from it produces three loops --
//! the torso and one for each arm -- and a measurement wants exactly one of
//! them. The same fact is used the other way round to find levels: the armpit
//! is the highest cut at which the arms are still their own loops, and the
//! crotch is the highest cut at which the legs are.

use std::collections::HashMap;

/// A triangle mesh being measured. Nothing is copied; the measurements read
/// the caller's arrays directly.
#[derive(Clone, Copy)]
pub struct BodyMesh<'a> {
    pub vertices: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
}

/// Where a point lands when a section is flattened.
///
/// Sections are cut on three kinds of plane and each has its own natural pair
/// of axes, so the loops are kept in 3D and projected on demand rather than
/// being flattened once into a basis that only one caller wants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axes {
    /// A horizontal cut, seen from above: `(x, z)`.
    Horizontal,
    /// A cut down the middle of the body, seen from the side: `(z, y)`.
    Sagittal,
}

/// Points are matched to a tenth of a millimetre when loops are assembled.
///
/// Two triangles sharing an edge each compute the crossing on that edge from
/// the same two vertices, but from opposite ends, so the two answers differ in
/// the last bits. Rounding to a grid is what lets them be recognised as one
/// point; a tenth of a millimetre is far below any feature of a body and far
/// above that error.
const GRID: f32 = 10.0;

fn key(point: [f32; 3]) -> [i32; 3] {
    [
        (point[0] * GRID).round() as i32,
        (point[1] * GRID).round() as i32,
        (point[2] * GRID).round() as i32,
    ]
}

/// One closed ring of the surface, in the order it goes round.
pub type Ring = Vec<[f32; 3]>;

/// Cut the mesh with the plane through `origin` with the given `normal`, and
/// return the rings the cut makes.
///
/// A vertex exactly on the plane is treated as being just above it. Deciding
/// it either way is arbitrary; deciding it *consistently* is not, because a
/// triangle whose vertices disagree about which side they are on produces a
/// segment out of nothing and breaks the ring it lands in.
pub fn cross_section(mesh: BodyMesh<'_>, origin: [f32; 3], normal: [f32; 3]) -> Vec<Ring> {
    let side = |vertex: [f32; 3]| {
        let distance = (vertex[0] - origin[0]) * normal[0]
            + (vertex[1] - origin[1]) * normal[1]
            + (vertex[2] - origin[2]) * normal[2];
        if distance == 0.0 {
            f32::MIN_POSITIVE
        } else {
            distance
        }
    };

    let mut segments: Vec<([f32; 3], [f32; 3])> = Vec::new();
    for face in mesh.faces {
        let points = [
            mesh.vertices[face[0] as usize],
            mesh.vertices[face[1] as usize],
            mesh.vertices[face[2] as usize],
        ];
        let distances = [side(points[0]), side(points[1]), side(points[2])];
        if distances.iter().all(|d| *d > 0.0) || distances.iter().all(|d| *d < 0.0) {
            continue;
        }

        let mut crossings = Vec::with_capacity(2);
        for edge in 0..3 {
            let next = (edge + 1) % 3;
            let (from, to) = (distances[edge], distances[next]);
            if (from > 0.0) == (to > 0.0) {
                continue;
            }
            let t = from / (from - to);
            let (a, b) = (points[edge], points[next]);
            crossings.push([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ]);
        }
        if let [a, b] = crossings[..]
            && key(a) != key(b)
        {
            segments.push((a, b));
        }
    }

    assemble(segments)
}

/// Chain the segments into closed rings.
///
/// A segment that cannot be continued ends its ring; the result is kept
/// anyway, because a body mesh with a hole in it should still measure rather
/// than silently drop the piece the hole is in.
fn assemble(segments: Vec<([f32; 3], [f32; 3])>) -> Vec<Ring> {
    let mut at: HashMap<[i32; 3], Vec<usize>> = HashMap::new();
    for (index, (a, b)) in segments.iter().enumerate() {
        at.entry(key(*a)).or_default().push(index);
        at.entry(key(*b)).or_default().push(index);
    }

    let mut used = vec![false; segments.len()];
    let mut rings = Vec::new();

    for start in 0..segments.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let (first, mut head) = segments[start];
        let mut ring = vec![first, head];

        loop {
            let candidates = at.get(&key(head));
            let next = candidates.and_then(|list| {
                list.iter()
                    .copied()
                    .find(|index| !used[*index] && matches(segments[*index], head))
            });
            let Some(index) = next else { break };
            used[index] = true;
            let (a, b) = segments[index];
            head = if key(a) == key(head) { b } else { a };
            if key(head) == key(ring[0]) {
                break;
            }
            ring.push(head);
        }

        if ring.len() >= 3 {
            rings.push(ring);
        }
    }

    rings
}

fn matches(segment: ([f32; 3], [f32; 3]), point: [f32; 3]) -> bool {
    key(segment.0) == key(point) || key(segment.1) == key(point)
}

/// Flatten a ring onto a pair of axes.
pub fn flatten(ring: &[[f32; 3]], axes: Axes) -> Vec<[f32; 2]> {
    ring.iter()
        .map(|point| match axes {
            Axes::Horizontal => [point[0], point[2]],
            Axes::Sagittal => [point[2], point[1]],
        })
        .collect()
}

/// Flatten a ring onto the plane it was cut on, whatever that plane is.
///
/// Only the perimeter is ever wanted from a limb section, and a perimeter does
/// not care which way the axes point, so any orthonormal pair perpendicular to
/// the normal will do.
pub fn flatten_on_plane(ring: &[[f32; 3]], normal: [f32; 3]) -> Vec<[f32; 2]> {
    let helper = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let dot = helper[0] * normal[0] + helper[1] * normal[1] + helper[2] * normal[2];
    let mut u = [
        helper[0] - normal[0] * dot,
        helper[1] - normal[1] * dot,
        helper[2] - normal[2] * dot,
    ];
    let length = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    u = [u[0] / length, u[1] / length, u[2] / length];
    let v = [
        u[1] * normal[2] - u[2] * normal[1],
        u[2] * normal[0] - u[0] * normal[2],
        u[0] * normal[1] - u[1] * normal[0],
    ];

    ring.iter()
        .map(|point| {
            [
                point[0] * u[0] + point[1] * u[1] + point[2] * u[2],
                point[0] * v[0] + point[1] * v[1] + point[2] * v[2],
            ]
        })
        .collect()
}

/// The convex hull of a flattened ring, anticlockwise, by Andrew's monotone
/// chain.
///
/// Girths are hulls rather than the ring itself because that is what a tape
/// measure does: it bridges a hollow -- the small of the back, the dip beside
/// a hip -- instead of following it in and out again.
pub fn hull(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    sorted.dedup_by(|a, b| a[0] == b[0] && a[1] == b[1]);
    if sorted.len() < 3 {
        return sorted;
    }

    let turn = |o: [f32; 2], a: [f32; 2], b: [f32; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };

    let mut chain: Vec<[f32; 2]> = Vec::with_capacity(sorted.len() * 2);
    for pass in 0..2 {
        let start = chain.len();
        let ordered: Box<dyn Iterator<Item = &[f32; 2]>> = if pass == 0 {
            Box::new(sorted.iter())
        } else {
            Box::new(sorted.iter().rev())
        };
        for point in ordered {
            while chain.len() >= start + 2
                && turn(chain[chain.len() - 2], chain[chain.len() - 1], *point) <= 0.0
            {
                chain.pop();
            }
            chain.push(*point);
        }
        chain.pop();
    }
    chain
}

pub fn perimeter(polygon: &[[f32; 2]]) -> f32 {
    if polygon.len() < 3 {
        return 0.0;
    }
    let mut total = 0.0;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        total += ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
    }
    total
}

pub fn centroid(points: &[[f32; 2]]) -> [f32; 2] {
    let count = points.len().max(1) as f32;
    let sum = points
        .iter()
        .fold([0.0f32, 0.0], |acc, p| [acc[0] + p[0], acc[1] + p[1]]);
    [sum[0] / count, sum[1] / count]
}

/// Whether a flattened ring encloses a point, by the crossing-number rule.
pub fn encloses(polygon: &[[f32; 2]], point: [f32; 2]) -> bool {
    let mut inside = false;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        if (a[1] > point[1]) != (b[1] > point[1]) {
            let crossing = a[0] + (point[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
            if crossing > point[0] {
                inside = !inside;
            }
        }
    }
    inside
}

/// The ring of a section that a point is in, or the one nearest it.
///
/// "Nearest" is the fallback rather than the rule: a cut through the chest
/// gives the torso and both arms, and the torso is the one that encloses the
/// spine. Only when nothing encloses the reference -- a cut just past the top
/// of the head, say -- does the closest ring win.
pub fn ring_at(rings: &[Ring], axes: Axes, reference: [f32; 2]) -> Option<&Ring> {
    let mut nearest: Option<(f32, &Ring)> = None;
    for ring in rings {
        let flat = flatten(ring, axes);
        if encloses(&flat, reference) {
            return Some(ring);
        }
        let middle = centroid(&flat);
        let distance = (middle[0] - reference[0]).powi(2) + (middle[1] - reference[1]).powi(2);
        if nearest.is_none_or(|(best, _)| distance < best) {
            nearest = Some((distance, ring));
        }
    }
    nearest.map(|(_, ring)| ring)
}

/// The length of the hull's back half: from the point furthest to one side of
/// the body to the point furthest to the other, going round the back.
///
/// This is GarmentCode's "section" family of measurements -- `back_width`,
/// `waist_back_width`, `hip_back_width`, `neck_w`. The two side points are its
/// balance lines: the vertical lines down the sides of the body.
pub fn back_width(hull: &[[f32; 2]]) -> f32 {
    if hull.len() < 3 {
        return 0.0;
    }
    let index_of = |pick: fn(f32, f32) -> bool| {
        let mut best = 0;
        for index in 1..hull.len() {
            if pick(hull[index][0], hull[best][0]) {
                best = index;
            }
        }
        best
    };
    let left = index_of(|a, b| a > b);
    let right = index_of(|a, b| a < b);

    // Both ways round from one side point to the other; the back is the way
    // whose points sit behind the section's middle.
    let walk = |from: usize, to: usize| {
        let mut length = 0.0;
        let mut depth = 0.0;
        let mut count = 0.0f32;
        let mut index = from;
        while index != to {
            let next = (index + 1) % hull.len();
            let (a, b) = (hull[index], hull[next]);
            length += ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
            depth += b[1];
            count += 1.0;
            index = next;
        }
        (length, depth / count.max(1.0))
    };

    let (forward, forward_depth) = walk(left, right);
    let (backward, backward_depth) = walk(right, left);
    if forward_depth < backward_depth {
        forward
    } else {
        backward
    }
}

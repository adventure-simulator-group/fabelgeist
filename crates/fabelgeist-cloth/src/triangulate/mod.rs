//! Turning a flat outline into a simulable mesh.
//!
//! A sewing pattern is a set of closed 2D outlines. A cloth solver needs
//! triangles, and not just any triangles: the mesh has to be roughly uniform,
//! because the stretch constraints are its edges and a sliver produces a
//! constraint that is far stiffer than its neighbours. Ear clipping alone
//! would give exactly that -- a fan of slivers with no interior vertices at
//! all -- so this scatters interior points first and triangulates the whole
//! set.
//!
//! 1. Resample the outline so that boundary segments are near the target edge
//!    length, keeping track of which input edge each boundary vertex came from
//!    (the seams need that later).
//! 2. Lay a triangular lattice over the interior and keep the points that are
//!    comfortably inside.
//! 3. Delaunay-triangulate the lot, then throw away the triangles whose
//!    centres fall outside the outline -- which is what makes a concave panel,
//!    an armhole or a neckline come out right.

use fabelgeist_math::Vec2;

/// The mesh of one panel.
#[derive(Clone, Debug, Default)]
pub struct PanelMesh {
    pub vertices: Vec<Vec2>,
    pub triangles: Vec<[u32; 3]>,
    /// For each edge of the input outline, the boundary vertices along it, in
    /// order, including both endpoints. Consecutive edges share a vertex.
    pub edge_chains: Vec<Vec<u32>>,
}

impl PanelMesh {
    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    /// Vertices that lie on the outline. Interior vertices come after them, so
    /// this is a prefix.
    pub fn boundary_count(&self) -> usize {
        self.edge_chains
            .iter()
            .flatten()
            .copied()
            .max()
            .map(|m| m as usize + 1)
            .unwrap_or(0)
    }
}

/// Twice the signed area of the polygon. Positive is counter-clockwise.
pub fn signed_area(outline: &[Vec2]) -> f32 {
    let mut total = 0.0;
    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[(i + 1) % outline.len()];
        total += a.x * b.y - b.x * a.y;
    }
    total * 0.5
}

/// Ray casting: count crossings of a ray going in +x.
pub fn contains(outline: &[Vec2], point: Vec2) -> bool {
    let mut inside = false;
    let mut j = outline.len() - 1;
    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[j];
        // `>` on one side and `<=` on the other, so a vertex exactly at the
        // ray's height is counted once rather than twice or not at all.
        if (a.y > point.y) != (b.y > point.y) {
            let t = (point.y - a.y) / (b.y - a.y);
            if point.x < a.x + t * (b.x - a.x) {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Distance from a point to the outline, unsigned.
pub fn distance_to_outline(outline: &[Vec2], point: Vec2) -> f32 {
    let mut best = f32::INFINITY;
    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[(i + 1) % outline.len()];
        best = best.min(distance_to_segment(point, a, b));
    }
    best
}

fn distance_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let length_squared = ab.length_sq();
    let t = if length_squared > 1e-20 {
        ((point - a).x * ab.x + (point - a).y * ab.y) / length_squared
    } else {
        0.0
    };
    let closest = a + ab * t.clamp(0.0, 1.0);
    (point - closest).length()
}

/// Mesh a closed outline.
///
/// `outline` is a simple polygon: no repeated final point, no self-crossings.
/// `target_edge` is the edge length to aim for -- the resolution knob for the
/// whole simulation, since it sets both the particle count and how fine a fold
/// the fabric can make.
pub fn triangulate(outline: &[Vec2], target_edge: f32) -> PanelMesh {
    if outline.len() < 3 || !(target_edge > 0.0) {
        return PanelMesh::default();
    }

    // The outline is used exactly as given, whichever way it winds. Reversing
    // a clockwise one would renumber the edges, and `edge_chains` is what a
    // seam declared between edge 3 of one panel and edge 5 of another resolves
    // through -- so a reversal here would mis-wire every seam in the garment
    // and still produce a plausible-looking mesh. Nothing downstream needs the
    // winding anyway: `contains` counts ray crossings, which is
    // winding-agnostic, and `orient` makes every output triangle
    // counter-clockwise regardless.
    let (mut vertices, edge_chains) = resample_boundary(outline, target_edge);
    let boundary_count = vertices.len();
    if boundary_count < 3 {
        return PanelMesh::default();
    }

    vertices.extend(interior_points(outline, target_edge));

    let triangles = delaunay(&vertices)
        .into_iter()
        .filter(|triangle| {
            let a = vertices[triangle[0] as usize];
            let b = vertices[triangle[1] as usize];
            let c = vertices[triangle[2] as usize];
            let centroid = (a + b + c) * (1.0 / 3.0);
            // The centroid decides. For a convex panel every triangle is
            // inside anyway; for a neckline or an armhole this is what removes
            // the ones spanning the hole.
            contains(outline, centroid)
        })
        .collect();

    PanelMesh {
        vertices,
        triangles,
        edge_chains,
    }
}

/// Split each outline edge into segments no longer than `target_edge`.
///
/// The chains record which vertices came from which input edge, which is how a
/// seam declared between two *edges* becomes constraints between two lists of
/// *particles*.
fn resample_boundary(outline: &[Vec2], target_edge: f32) -> (Vec<Vec2>, Vec<Vec<u32>>) {
    let mut vertices: Vec<Vec2> = Vec::new();
    let mut chains: Vec<Vec<u32>> = Vec::new();

    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[(i + 1) % outline.len()];
        let length = (b - a).length();
        let segments = ((length / target_edge).round() as usize).max(1);

        let mut chain = Vec::with_capacity(segments + 1);
        // The first vertex of this edge is the last of the previous one; only
        // the very first edge has to create it.
        let start = if i == 0 {
            vertices.push(a);
            0u32
        } else {
            *chains[i - 1].last().unwrap()
        };
        chain.push(start);

        for step in 1..segments {
            let t = step as f32 / segments as f32;
            vertices.push(a + (b - a) * t);
            chain.push(vertices.len() as u32 - 1);
        }

        // The last edge closes the loop back onto vertex 0 rather than adding
        // a duplicate of it.
        if i + 1 == outline.len() {
            chain.push(0);
        } else {
            vertices.push(b);
            chain.push(vertices.len() as u32 - 1);
        }

        chains.push(chain);
    }

    (vertices, chains)
}

/// A triangular lattice clipped to the interior.
///
/// Rows are offset by half a spacing and spaced by `sqrt(3)/2` of it, which is
/// the packing that makes every lattice triangle equilateral -- the best
/// starting point for a uniform mesh.
fn interior_points(outline: &[Vec2], target_edge: f32) -> Vec<Vec2> {
    let mut minimum = outline[0];
    let mut maximum = outline[0];
    for point in outline {
        minimum = Vec2::new(minimum.x.min(point.x), minimum.y.min(point.y));
        maximum = Vec2::new(maximum.x.max(point.x), maximum.y.max(point.y));
    }

    let row_height = target_edge * 0.866_025_4;
    let rows = (((maximum.y - minimum.y) / row_height).ceil() as usize).max(1);
    let columns = (((maximum.x - minimum.x) / target_edge).ceil() as usize).max(1);

    // A lattice point closer to the boundary than this would sit almost on top
    // of a resampled boundary vertex and produce a sliver.
    let clearance = target_edge * 0.65;

    let mut points = Vec::new();
    for row in 0..=rows {
        let y = minimum.y + row as f32 * row_height;
        let offset = if row % 2 == 1 { target_edge * 0.5 } else { 0.0 };
        for column in 0..=columns {
            let lattice = Vec2::new(minimum.x + offset + column as f32 * target_edge, y);
            // A perfect lattice puts four points on a common circle over and
            // over, and the in-circle test cannot break those ties
            // consistently -- which leaves the Delaunay hole non-star-shaped
            // and the retriangulation overlapping itself. A small deterministic
            // nudge removes the ties. It also makes a slightly better cloth:
            // a perfectly regular mesh has its stretch constraints aligned to
            // two axes, and folds along them in preference to any other
            // direction.
            let point = lattice + jitter(row, column) * (target_edge * 0.12);
            if contains(outline, point) && distance_to_outline(outline, point) > clearance {
                points.push(point);
            }
        }
    }
    points
}

/// A repeatable offset in `[-1, 1]^2` from a lattice coordinate. Deterministic,
/// so the same panel always meshes the same way.
fn jitter(row: usize, column: usize) -> Vec2 {
    let mut hash =
        (row as u32).wrapping_mul(0x9E37_79B9) ^ (column as u32).wrapping_mul(0x85EB_CA6B);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0xC2B2_AE35);
    hash ^= hash >> 13;
    let x = (hash & 0xFFFF) as f32 / 32768.0 - 1.0;
    let y = ((hash >> 16) & 0xFFFF) as f32 / 32768.0 - 1.0;
    Vec2::new(x, y)
}

/// Bowyer-Watson Delaunay triangulation.
///
/// Quadratic in the point count, which is fine here: a panel is thousands of
/// points at most, and this runs once when the garment is built rather than
/// per frame.
pub fn delaunay(points: &[Vec2]) -> Vec<[u32; 3]> {
    if points.len() < 3 {
        return Vec::new();
    }

    // A triangle big enough to contain everything, whose vertices are appended
    // past the real points and removed at the end.
    let mut minimum = points[0];
    let mut maximum = points[0];
    for point in points {
        minimum = Vec2::new(minimum.x.min(point.x), minimum.y.min(point.y));
        maximum = Vec2::new(maximum.x.max(point.x), maximum.y.max(point.y));
    }
    let center = (minimum + maximum) * 0.5;
    let span = (maximum - minimum).length().max(1e-6);
    let far = span * 10.0;

    let mut working: Vec<Vec2> = points.to_vec();
    let base = working.len() as u32;
    working.push(center + Vec2::new(-far, -far));
    working.push(center + Vec2::new(far, -far));
    working.push(center + Vec2::new(0.0, far));

    let mut triangles: Vec<[u32; 3]> = vec![[base, base + 1, base + 2]];

    for index in 0..points.len() as u32 {
        let point = working[index as usize];

        // Every triangle whose circumcircle contains the point loses its claim
        // to the region; together they form a star-shaped hole.
        let mut bad = Vec::new();
        let mut kept = Vec::with_capacity(triangles.len());
        for triangle in triangles.drain(..) {
            if in_circumcircle(&working, triangle, point) {
                bad.push(triangle);
            } else {
                kept.push(triangle);
            }
        }
        triangles = kept;

        // The hole's boundary is the edges belonging to exactly one bad
        // triangle; the shared ones are interior and go away with it.
        let mut edges: Vec<([u32; 2], bool)> = Vec::new();
        for triangle in &bad {
            for pair in [
                [triangle[0], triangle[1]],
                [triangle[1], triangle[2]],
                [triangle[2], triangle[0]],
            ] {
                let key = if pair[0] < pair[1] {
                    [pair[0], pair[1]]
                } else {
                    [pair[1], pair[0]]
                };
                if let Some(entry) = edges.iter_mut().find(|(e, _)| *e == key) {
                    entry.1 = false;
                } else {
                    edges.push((key, true));
                }
            }
        }

        for (edge, unique) in edges {
            if unique {
                triangles.push([edge[0], edge[1], index]);
            }
        }
    }

    triangles
        .into_iter()
        .filter(|triangle| triangle.iter().all(|&v| v < base))
        .map(|triangle| orient(&working, triangle))
        .collect()
}

/// Counter-clockwise, so that every triangle in the output winds the same way.
fn orient(points: &[Vec2], triangle: [u32; 3]) -> [u32; 3] {
    let a = points[triangle[0] as usize];
    let b = points[triangle[1] as usize];
    let c = points[triangle[2] as usize];
    let cross = (b - a).x * (c - a).y - (b - a).y * (c - a).x;
    if cross < 0.0 {
        [triangle[0], triangle[2], triangle[1]]
    } else {
        triangle
    }
}

fn in_circumcircle(points: &[Vec2], triangle: [u32; 3], point: Vec2) -> bool {
    let a = points[triangle[0] as usize];
    let b = points[triangle[1] as usize];
    let c = points[triangle[2] as usize];

    // The standard determinant test, which needs a known orientation; the
    // sign of the triangle's own area supplies it.
    let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    if area.abs() < 1e-20 {
        // Degenerate triangle: no circumcircle to be inside of.
        return false;
    }

    let ax = a.x - point.x;
    let ay = a.y - point.y;
    let bx = b.x - point.x;
    let by = b.y - point.y;
    let cx = c.x - point.x;
    let cy = c.y - point.y;

    let determinant = (ax * ax + ay * ay) * (bx * cy - by * cx)
        - (bx * bx + by * by) * (ax * cy - ay * cx)
        + (cx * cx + cy * cy) * (ax * by - ay * bx);

    // A point exactly on the circle is a tie, and answering it inconsistently
    // for the several triangles that share it is what breaks the star shape of
    // the hole. Scale the tolerance by the triangle so it means the same thing
    // at any size, and always answer "outside" -- that keeps the existing
    // triangle rather than opening a hole nothing closes.
    let scale = (ax * ax + ay * ay)
        .max(bx * bx + by * by)
        .max(cx * cx + cy * cy);
    let epsilon = 1e-6 * scale * area.abs();
    if determinant.abs() <= epsilon {
        return false;
    }

    if area > 0.0 {
        determinant > 0.0
    } else {
        determinant < 0.0
    }
}

#[cfg(test)]
mod tests;

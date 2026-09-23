//! Conforming cloth to its wearer after draping or transfer: holding cloth
//! together where it was pulled apart, drawing it onto the body it dresses,
//! pushing it clear of the wearer and inner garments, and covering body
//! features smaller than its triangles.
use super::dressing::Dressing;
use super::*;

/// Longest a transferred cloth edge may become, relative to its settled
/// length. A body that grows stretches its cloth evenly and stays under this;
/// a hem bound to both thighs of a body with wider hips would tear far beyond.
pub(super) const TRANSFER_STRETCH: f32 = 1.25;
/// Cloth drawn onto the body may not stretch at all.
const DRAWN_STRETCH: f32 = 1.0;
/// Jacobi passes pulling overstretched edges back to their limit.
const STRETCH_PASSES: usize = 64;
/// Rounds of pushing cloth out of what it must clear, each spreading the push
/// to neighbouring cloth so a single vertex never spikes out.
const CLEARING_ROUNDS: usize = 4;
/// How far inside a surface a cloth vertex is still found and pushed out, in
/// metres. Offsets follow the surface, so crossings stay shallow.
const PENETRATION_REACH_M: f32 = 0.03;
/// Share of the neighbours' average push a vertex takes when it was not
/// pushed itself.
const PUSH_SPREAD: f32 = 0.5;
/// Rounds of drawing cloth in, each followed by the stretch limit and clearance.
const DRAWING_ROUNDS: usize = 12;
/// Cloth further than this from the body hangs free and is not drawn in, m.
const DRAWING_REACH_M: f32 = 0.15;
/// Rounds of lifting cloth over body features that pass through it.
const COVERING_ROUNDS: usize = 6;
/// Rounds of clearing cloth and covering features. Covering a feature can tip
/// a neighbouring vertex back in, which the next round settles.
const KEEP_OUT_ROUNDS: usize = 2;
/// Least alignment between the cloth's and the surface's outward normals for
/// a feature to lift the cloth. Cloth standing edge-on to the surface, such
/// as a hem folded against it, is not lifted along the surface normal.
const MINIMUM_FACING: f32 = 0.3;
/// Barycentric margin inside a cloth triangle for a body feature to count as
/// passing through it rather than beside its edge.
const INTERIOR_MARGIN: f32 = 1e-3;

/// The cloth as sewn: one position per sewn vertex, so seam copies move
/// together, with its edges and each sewn vertex's neighbours.
pub(super) struct SewnCloth {
    surface: SewnSurface,
    neighbours: Vec<Vec<usize>>,
    edges: Vec<[usize; 2]>,
}

impl SewnCloth {
    pub(super) fn new(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Self {
        let surface = SewnSurface::from_positions(positions, faces);
        let mut edges = std::collections::BTreeSet::new();
        for face in &surface.faces {
            for k in 0..3 {
                let (a, b) = (face[k] as usize, face[(k + 1) % 3] as usize);
                edges.insert([a.min(b), a.max(b)]);
            }
        }
        let mut neighbours = vec![Vec::new(); surface.groups.len()];
        for &[a, b] in &edges {
            neighbours[a].push(b);
            neighbours[b].push(a);
        }
        Self {
            surface,
            neighbours,
            edges: edges.into_iter().collect(),
        }
    }

    fn sewn(&self, positions: &[[f32; 3]]) -> Vec<Vec3> {
        self.surface
            .positions(positions)
            .into_iter()
            .map(vector)
            .collect()
    }

    fn expand(&self, sewn: Vec<Vec3>) -> Vec<[f32; 3]> {
        self.surface
            .expand(&sewn.into_iter().map(array).collect::<Vec<_>>())
    }

    /// Edge length limits: `stretch` times each edge's length in `rest`.
    fn limits(&self, rest: &[Vec3], stretch: f32) -> Vec<f32> {
        self.edges
            .iter()
            .map(|&[a, b]| (rest[b] - rest[a]).length() * stretch)
            .collect()
    }

    /// Pull edges stretched past `stretch` times their length in `rest` back
    /// towards it; the rest of the cloth is kept as it is.
    pub(super) fn limit_stretch(
        &self,
        rest: &[[f32; 3]],
        positions: &[[f32; 3]],
        stretch: f32,
    ) -> Vec<[f32; 3]> {
        let limits = self.limits(&self.sewn(rest), stretch);
        let mut points = self.sewn(positions);
        self.hold(&mut points, &limits);
        self.expand(points)
    }

    fn hold(&self, points: &mut [Vec3], limits: &[f32]) {
        for _ in 0..STRETCH_PASSES {
            let mut pulls = vec![(Vec3::default(), 0u32); points.len()];
            for (&[a, b], &limit) in self.edges.iter().zip(limits) {
                let edge = points[b] - points[a];
                let length = edge.length();
                if length <= limit {
                    continue;
                }
                let pull = edge * ((length - limit) / length * 0.5);
                pulls[a] = (pulls[a].0 + pull, pulls[a].1 + 1);
                pulls[b] = (pulls[b].0 - pull, pulls[b].1 + 1);
            }
            if pulls.iter().all(|(_, count)| *count == 0) {
                break;
            }
            for (point, (pull, count)) in points.iter_mut().zip(pulls) {
                if count > 0 {
                    *point += pull / count as f32;
                }
            }
        }
    }

    /// Push cloth out to `clearance` from the collision surface: the wearer
    /// and inner garments, whose triangles face outward.
    fn clear(
        &self,
        positions: &[[f32; 3]],
        collision: &fabelgeist_bvh::TriangleBvh,
        clearance: f32,
    ) -> Vec<[f32; 3]> {
        let mut sewn = self.sewn(positions);
        self.push_clear(&mut sewn, collision, clearance);
        self.expand(sewn)
    }

    fn push_clear(
        &self,
        sewn: &mut [Vec3],
        collision: &fabelgeist_bvh::TriangleBvh,
        clearance: f32,
    ) {
        for round in 0..=CLEARING_ROUNDS {
            let pushes: Vec<Option<Vec3>> = sewn
                .iter()
                .map(|&point| push_out(point, collision, clearance))
                .collect();
            let spread = round < CLEARING_ROUNDS;
            for (vertex, point) in sewn.iter_mut().enumerate() {
                let push = pushes[vertex].or_else(|| {
                    let pushed: Vec<Vec3> = self.neighbours[vertex]
                        .iter()
                        .filter_map(|&n| pushes[n])
                        .collect();
                    (spread && !pushed.is_empty()).then(|| {
                        pushed.iter().fold(Vec3::default(), |sum, p| sum + *p) * PUSH_SPREAD
                            / self.neighbours[vertex].len() as f32
                    })
                });
                if let Some(push) = push {
                    *point += push;
                }
            }
        }
    }

    /// Keep the wearer and inner garments inside the cloth: push its vertices
    /// clear of them and lift it over features passing between its vertices.
    pub(super) fn keep_out(
        &self,
        positions: &[[f32; 3]],
        collision: &fabelgeist_bvh::TriangleBvh,
        clearance: f32,
    ) -> Vec<[f32; 3]> {
        let mut positions = positions.to_vec();
        for _ in 0..KEEP_OUT_ROUNDS {
            let cleared = self.clear(&positions, collision, clearance);
            positions = self.cover(&cleared, collision, clearance);
        }
        positions
    }

    /// Draw cloth floating over the parts of the body it dresses onto them,
    /// by `fit` of its gap, without stretching it. Cloth hanging past the
    /// body, such as a hem between the legs, follows only as it is pulled.
    pub(super) fn draw_in(
        &self,
        positions: &[[f32; 3]],
        dressing: &Dressing,
        collision: &fabelgeist_bvh::TriangleBvh,
        clearance: f32,
        fit: f32,
    ) -> Vec<[f32; 3]> {
        let mut points = self.sewn(positions);
        let limits = self.limits(&points, DRAWN_STRETCH);
        let dressed: Vec<bool> = points.iter().map(|&p| dressing.dresses(p)).collect();
        // Each round closes the same share of the remaining gap.
        let rate = 1.0 - (1.0 - fit).powf(1.0 / DRAWING_ROUNDS as f32);
        for _ in 0..DRAWING_ROUNDS {
            for (point, _) in points.iter_mut().zip(&dressed).filter(|(_, d)| **d) {
                let Some((triangle, closest, _)) = collision.closest_point(*point, DRAWING_REACH_M)
                else {
                    continue;
                };
                let (a, b, c) = collision.triangle(triangle);
                let Some(normal) = unit((b - a).cross(c - a)) else {
                    continue;
                };
                let gap = (*point - closest).dot(normal) - clearance;
                if gap > 0.0 {
                    *point -= normal * (gap * rate);
                }
            }
            self.hold(&mut points, &limits);
            self.push_clear(&mut points, collision, clearance);
        }
        self.expand(points)
    }

    /// Lift cloth over features of the collision surface that pass through its
    /// triangles between their vertices, such as a thumb through a cuff. The
    /// cloth rises along the surface's outward normal at the feature, which
    /// stays steady where a crumpled hem's triangles face every way.
    fn cover(
        &self,
        positions: &[[f32; 3]],
        collision: &fabelgeist_bvh::TriangleBvh,
        clearance: f32,
    ) -> Vec<[f32; 3]> {
        let mut points = self.sewn(positions);
        let outward: Vec<Vec3> = normals(
            &collision
                .positions
                .iter()
                .map(|p| array(*p))
                .collect::<Vec<_>>(),
            &collision.triangles,
        )
        .into_iter()
        .map(vector)
        .collect();
        for _ in 0..COVERING_ROUNDS {
            let cloth =
                fabelgeist_bvh::TriangleBvh::new(points.clone(), self.surface.faces.clone());
            let mut lifts = vec![(Vec3::default(), 0u32); points.len()];
            for (&feature, &out) in collision.positions.iter().zip(&outward) {
                let Some((triangle, closest, _)) =
                    cloth.closest_point(feature, clearance + PENETRATION_REACH_M)
                else {
                    continue;
                };
                let (a, b, c) = cloth.triangle(triangle);
                let Some(normal) = unit((b - a).cross(c - a)) else {
                    continue;
                };
                let weights = barycentric(a, b, c, closest);
                let depth = (feature - closest).dot(normal) + clearance;
                let facing = out.dot(normal);
                if depth <= 0.0
                    || facing < MINIMUM_FACING
                    || weights.iter().any(|w| *w < INTERIOR_MARGIN)
                {
                    continue;
                }
                // Move the triangle outward so the point over the feature
                // rises `depth` along the cloth's normal.
                let scale = depth / facing / weights.iter().map(|w| w * w).sum::<f32>();
                let corners = self.surface.faces[triangle as usize];
                for (corner, weight) in corners.iter().zip(weights) {
                    let lift = &mut lifts[*corner as usize];
                    *lift = (lift.0 + out * (weight * scale), lift.1 + 1);
                }
            }
            if lifts.iter().all(|(_, count)| *count == 0) {
                break;
            }
            for (point, (lift, count)) in points.iter_mut().zip(lifts) {
                if count > 0 {
                    *point += lift / count as f32;
                }
            }
        }
        self.expand(points)
    }
}

/// How far `point` must move to lie `clearance` outside the nearest surface.
fn push_out(point: Vec3, collision: &fabelgeist_bvh::TriangleBvh, clearance: f32) -> Option<Vec3> {
    let (triangle, closest, _) = collision.closest_point(point, clearance + PENETRATION_REACH_M)?;
    let (a, b, c) = collision.triangle(triangle);
    let normal = unit((b - a).cross(c - a))?;
    let height = (point - closest).dot(normal);
    (height < clearance).then(|| normal * (clearance - height))
}

pub(super) fn unit(v: Vec3) -> Option<Vec3> {
    let length = v.length();
    (length > 1e-10).then(|| v / length)
}

/// Barycentric weights of `p`, a point on triangle `abc`.
pub(super) fn barycentric(a: Vec3, b: Vec3, c: Vec3, p: Vec3) -> [f32; 3] {
    let (u, v, q) = (b - a, c - a, p - a);
    let (uu, uv, vv) = (u.dot(u), u.dot(v), v.dot(v));
    let determinant = uu * vv - uv * uv;
    if determinant.abs() <= 1e-15 {
        return [1.0, 0.0, 0.0];
    }
    let y = (vv * q.dot(u) - uv * q.dot(v)) / determinant;
    let z = (uu * q.dot(v) - uv * q.dot(u)) / determinant;
    [1.0 - y - z, y, z]
}
